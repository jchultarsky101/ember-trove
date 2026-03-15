use async_trait::async_trait;
use common::{
    EmberTroveError,
    id::NodeId,
    search::{SearchQuery, SearchResponse, SearchResult},
};
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait SearchRepo: Send + Sync {
    async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, EmberTroveError>;
}

pub struct PgSearchRepo {
    pool: PgPool,
}

impl PgSearchRepo {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Intermediate row type for full-text search results.
#[derive(sqlx::FromRow)]
struct SearchRow {
    id: Uuid,
    title: String,
    slug: String,
    snippet: Option<String>,
    rank: f32,
}

impl SearchRow {
    fn into_result(self) -> SearchResult {
        SearchResult {
            node_id: NodeId(self.id),
            title: self.title,
            slug: self.slug,
            snippet: self.snippet,
            rank: self.rank,
        }
    }
}

/// Count-only row for the total-count subquery.
#[derive(sqlx::FromRow)]
struct CountRow {
    total: i64,
}

#[async_trait]
impl SearchRepo for PgSearchRepo {
    async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, EmberTroveError> {
        let q = query.q.trim();
        if q.is_empty() {
            return Ok(SearchResponse {
                results: vec![],
                total: 0,
                page: 1,
                per_page: query.per_page.unwrap_or(20),
            });
        }

        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(20).min(100);
        let offset = (page - 1) * per_page;
        let fuzzy = query.fuzzy.unwrap_or(false);

        if fuzzy {
            self.fuzzy_search(q, &query.node_type, &query.status, page, per_page, offset)
                .await
        } else {
            self.fulltext_search(q, &query.node_type, &query.status, page, per_page, offset)
                .await
        }
    }
}

impl PgSearchRepo {
    /// PostgreSQL full-text search using `search_vec` tsvector column with
    /// `websearch_to_tsquery` for natural-language queries.
    async fn fulltext_search(
        &self,
        q: &str,
        node_type: &Option<common::node::NodeType>,
        status: &Option<common::node::NodeStatus>,
        page: u32,
        per_page: u32,
        offset: u32,
    ) -> Result<SearchResponse, EmberTroveError> {
        let type_filter = node_type.as_ref().map(node_type_to_str);
        let status_filter = status.as_ref().map(node_status_to_str);

        // Count query
        let count_row = sqlx::query_as::<_, CountRow>(
            r#"
            SELECT COUNT(*)::bigint AS total
            FROM nodes
            WHERE search_vec @@ websearch_to_tsquery('english', $1)
              AND ($2::text IS NULL OR node_type = $2::node_type)
              AND ($3::text IS NULL OR status = $3::node_status)
            "#,
        )
        .bind(q)
        .bind(type_filter)
        .bind(status_filter)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("search count failed: {e}")))?;

        // Results with ts_headline snippet and ts_rank_cd scoring
        let rows = sqlx::query_as::<_, SearchRow>(
            r#"
            SELECT
                id,
                title,
                slug,
                ts_headline(
                    'english',
                    coalesce(body, ''),
                    websearch_to_tsquery('english', $1),
                    'StartSel=<mark>, StopSel=</mark>, MaxFragments=2, MaxWords=30, MinWords=10'
                ) AS snippet,
                ts_rank_cd(search_vec, websearch_to_tsquery('english', $1)) AS rank
            FROM nodes
            WHERE search_vec @@ websearch_to_tsquery('english', $1)
              AND ($2::text IS NULL OR node_type = $2::node_type)
              AND ($3::text IS NULL OR status = $3::node_status)
            ORDER BY rank DESC, updated_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(q)
        .bind(type_filter)
        .bind(status_filter)
        .bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("fulltext search failed: {e}")))?;

        Ok(SearchResponse {
            results: rows.into_iter().map(SearchRow::into_result).collect(),
            total: count_row.total as u64,
            page,
            per_page,
        })
    }

    /// Fuzzy trigram search using pg_trgm's `similarity()` function on the
    /// title column, falling back to `ILIKE` on the body.
    async fn fuzzy_search(
        &self,
        q: &str,
        node_type: &Option<common::node::NodeType>,
        status: &Option<common::node::NodeStatus>,
        page: u32,
        per_page: u32,
        offset: u32,
    ) -> Result<SearchResponse, EmberTroveError> {
        let type_filter = node_type.as_ref().map(node_type_to_str);
        let status_filter = status.as_ref().map(node_status_to_str);
        let like_pattern = format!("%{q}%");

        // Count query
        let count_row = sqlx::query_as::<_, CountRow>(
            r#"
            SELECT COUNT(*)::bigint AS total
            FROM nodes
            WHERE (similarity(title, $1) > 0.1 OR body ILIKE $2)
              AND ($3::text IS NULL OR node_type = $3::node_type)
              AND ($4::text IS NULL OR status = $4::node_status)
            "#,
        )
        .bind(q)
        .bind(&like_pattern)
        .bind(type_filter)
        .bind(status_filter)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("fuzzy count failed: {e}")))?;

        // Results ranked by trigram similarity on title
        let rows = sqlx::query_as::<_, SearchRow>(
            r#"
            SELECT
                id,
                title,
                slug,
                CASE
                    WHEN body ILIKE $2
                    THEN substring(body FROM 1 FOR 200)
                    ELSE NULL
                END AS snippet,
                GREATEST(similarity(title, $1), 0.0) AS rank
            FROM nodes
            WHERE (similarity(title, $1) > 0.1 OR body ILIKE $2)
              AND ($3::text IS NULL OR node_type = $3::node_type)
              AND ($4::text IS NULL OR status = $4::node_status)
            ORDER BY rank DESC, updated_at DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(q)
        .bind(&like_pattern)
        .bind(type_filter)
        .bind(status_filter)
        .bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("fuzzy search failed: {e}")))?;

        Ok(SearchResponse {
            results: rows.into_iter().map(SearchRow::into_result).collect(),
            total: count_row.total as u64,
            page,
            per_page,
        })
    }
}

fn node_type_to_str(t: &common::node::NodeType) -> &'static str {
    match t {
        common::node::NodeType::Article => "article",
        common::node::NodeType::Project => "project",
        common::node::NodeType::Area => "area",
        common::node::NodeType::Resource => "resource",
        common::node::NodeType::Reference => "reference",
    }
}

fn node_status_to_str(s: &common::node::NodeStatus) -> &'static str {
    match s {
        common::node::NodeStatus::Draft => "draft",
        common::node::NodeStatus::Published => "published",
        common::node::NodeStatus::Archived => "archived",
    }
}
