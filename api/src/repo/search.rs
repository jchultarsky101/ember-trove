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

/// Intermediate row type for all search result queries.
#[derive(sqlx::FromRow)]
struct SearchRow {
    id: Uuid,
    title: String,
    slug: String,
    snippet: Option<String>,
    rank: f32,
    node_type: String,
    status: String,
    match_source: Option<String>,
}

impl SearchRow {
    fn into_result(self) -> SearchResult {
        SearchResult {
            node_id: NodeId(self.id),
            title: self.title,
            slug: self.slug,
            snippet: self.snippet,
            rank: self.rank,
            node_type: self.node_type,
            status: self.status,
            match_source: self.match_source,
        }
    }
}

/// Count-only row for the total-count subquery.
#[derive(sqlx::FromRow)]
struct CountRow {
    total: i64,
}

/// Parse the `tag_ids` comma-separated string from the query into a Vec<Uuid>.
fn parse_tag_ids(tag_ids: &Option<String>) -> Vec<Uuid> {
    tag_ids
        .as_deref()
        .unwrap_or("")
        .split(',')
        .filter(|s| !s.is_empty())
        .filter_map(|s| Uuid::parse_str(s.trim()).ok())
        .collect()
}

#[async_trait]
impl SearchRepo for PgSearchRepo {
    async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, EmberTroveError> {
        let q = query.q.trim();
        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(20).min(100);
        let offset = (page - 1) * per_page;
        let tag_ids = parse_tag_ids(&query.tag_ids);
        let and_mode = query.tag_op.as_deref() == Some("and");

        // Empty query → list all nodes (optionally filtered by tags).
        if q.is_empty() {
            return self
                .list_nodes(&query.node_type, &query.status, &tag_ids, and_mode, page, per_page, offset)
                .await;
        }

        let fuzzy = query.fuzzy.unwrap_or(false);

        if fuzzy {
            self.fuzzy_search(q, &query.node_type, &query.status, &tag_ids, and_mode, page, per_page, offset)
                .await
        } else {
            self.fulltext_search(q, &query.node_type, &query.status, &tag_ids, and_mode, page, per_page, offset)
                .await
        }
    }
}

impl PgSearchRepo {
    /// List nodes with optional tag filter — used for empty-query browsing.
    ///
    /// Tag filter SQL uses `= ANY($n::uuid[])` with a HAVING clause to express
    /// AND/OR without dynamic query construction:
    ///   - OR (`and_mode = false`): `HAVING true OR COUNT(...) = n`  → all rows qualify
    ///   - AND (`and_mode = true`):  `HAVING false OR COUNT(...) = n` → only rows with every tag
    ///
    /// An empty array means no tag filter at all (`array_length = NULL`).
    #[allow(clippy::too_many_arguments)]
    async fn list_nodes(
        &self,
        node_type: &Option<common::node::NodeType>,
        status: &Option<common::node::NodeStatus>,
        tag_ids: &[Uuid],
        and_mode: bool,
        page: u32,
        per_page: u32,
        offset: u32,
    ) -> Result<SearchResponse, EmberTroveError> {
        let type_filter = node_type.as_ref().map(node_type_to_str);
        let status_filter = status.as_ref().map(node_status_to_str);

        let count_row = sqlx::query_as::<_, CountRow>(
            r#"
            SELECT COUNT(*)::bigint AS total
            FROM nodes
            WHERE ($1::text IS NULL OR node_type = $1::node_type)
              AND ($2::text IS NULL OR status = $2::node_status)
              AND (
                array_length($3::uuid[], 1) IS NULL
                OR id IN (
                    SELECT node_id FROM node_tags
                    WHERE tag_id = ANY($3::uuid[])
                    GROUP BY node_id
                    HAVING (NOT $4) OR COUNT(DISTINCT tag_id) = array_length($3::uuid[], 1)
                )
              )
            "#,
        )
        .bind(type_filter)
        .bind(status_filter)
        .bind(tag_ids)
        .bind(and_mode)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("list nodes count failed: {e}")))?;

        let rows = sqlx::query_as::<_, SearchRow>(
            r#"
            SELECT
                id,
                title,
                slug,
                NULL::text AS snippet,
                1.0::float4 AS rank,
                node_type::text AS node_type,
                status::text AS status,
                NULL::text AS match_source
            FROM nodes
            WHERE ($1::text IS NULL OR node_type = $1::node_type)
              AND ($2::text IS NULL OR status = $2::node_status)
              AND (
                array_length($3::uuid[], 1) IS NULL
                OR id IN (
                    SELECT node_id FROM node_tags
                    WHERE tag_id = ANY($3::uuid[])
                    GROUP BY node_id
                    HAVING (NOT $4) OR COUNT(DISTINCT tag_id) = array_length($3::uuid[], 1)
                )
              )
            ORDER BY updated_at DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(type_filter)
        .bind(status_filter)
        .bind(tag_ids)
        .bind(and_mode)
        .bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("list nodes failed: {e}")))?;

        Ok(SearchResponse {
            results: rows.into_iter().map(SearchRow::into_result).collect(),
            total: count_row.total as u64,
            page,
            per_page,
        })
    }

    /// PostgreSQL full-text search across node title/body, note bodies, and task titles.
    ///
    /// Uses a `UNION ALL` of three candidate sets (node / note / task), then
    /// `DISTINCT ON (id)` to deduplicate by node, keeping the highest-ranked
    /// source.  A `valid_nodes` CTE applies type/status/tag filters once so
    /// they are not repeated in every branch.
    #[allow(clippy::too_many_arguments)]
    async fn fulltext_search(
        &self,
        q: &str,
        node_type: &Option<common::node::NodeType>,
        status: &Option<common::node::NodeStatus>,
        tag_ids: &[Uuid],
        and_mode: bool,
        page: u32,
        per_page: u32,
        offset: u32,
    ) -> Result<SearchResponse, EmberTroveError> {
        let type_filter = node_type.as_ref().map(node_type_to_str);
        let status_filter = status.as_ref().map(node_status_to_str);

        let count_row = sqlx::query_as::<_, CountRow>(
            r#"
            WITH valid_nodes AS (
                SELECT id FROM nodes
                WHERE ($2::text IS NULL OR node_type = $2::node_type)
                  AND ($3::text IS NULL OR status = $3::node_status)
                  AND (
                    array_length($4::uuid[], 1) IS NULL
                    OR id IN (
                        SELECT node_id FROM node_tags
                        WHERE tag_id = ANY($4::uuid[])
                        GROUP BY node_id
                        HAVING (NOT $5) OR COUNT(DISTINCT tag_id) = array_length($4::uuid[], 1)
                    )
                  )
            )
            SELECT COUNT(DISTINCT n.id)::bigint AS total
            FROM nodes n
            WHERE n.id IN (SELECT id FROM valid_nodes)
              AND (
                n.search_vec @@ websearch_to_tsquery('english', $1)
                OR EXISTS (
                    SELECT 1 FROM node_notes nn
                    WHERE nn.node_id = n.id
                      AND nn.search_vec @@ websearch_to_tsquery('english', $1)
                )
                OR EXISTS (
                    SELECT 1 FROM node_tasks nt
                    WHERE nt.node_id = n.id
                      AND to_tsvector('english', nt.title) @@ websearch_to_tsquery('english', $1)
                )
              )
            "#,
        )
        .bind(q)
        .bind(type_filter)
        .bind(status_filter)
        .bind(tag_ids)
        .bind(and_mode)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("search count failed: {e}")))?;

        let rows = sqlx::query_as::<_, SearchRow>(
            r#"
            WITH valid_nodes AS (
                SELECT id FROM nodes
                WHERE ($2::text IS NULL OR node_type = $2::node_type)
                  AND ($3::text IS NULL OR status = $3::node_status)
                  AND (
                    array_length($4::uuid[], 1) IS NULL
                    OR id IN (
                        SELECT node_id FROM node_tags
                        WHERE tag_id = ANY($4::uuid[])
                        GROUP BY node_id
                        HAVING (NOT $5) OR COUNT(DISTINCT tag_id) = array_length($4::uuid[], 1)
                    )
                  )
            ),
            candidates AS (
                -- Match from node title / body
                SELECT
                    n.id,
                    n.title,
                    n.slug,
                    ts_headline(
                        'english',
                        coalesce(n.body, ''),
                        websearch_to_tsquery('english', $1),
                        'StartSel=<mark>, StopSel=</mark>, MaxFragments=2, MaxWords=30, MinWords=10'
                    ) AS snippet,
                    ts_rank_cd(n.search_vec, websearch_to_tsquery('english', $1)) AS rank,
                    n.node_type::text AS node_type,
                    n.status::text AS status,
                    'node'::text AS match_source
                FROM nodes n
                WHERE n.id IN (SELECT id FROM valid_nodes)
                  AND n.search_vec @@ websearch_to_tsquery('english', $1)

                UNION ALL

                -- Match from an attached note body
                SELECT
                    n.id,
                    n.title,
                    n.slug,
                    ts_headline(
                        'english',
                        nn.body,
                        websearch_to_tsquery('english', $1),
                        'StartSel=<mark>, StopSel=</mark>, MaxFragments=2, MaxWords=30, MinWords=10'
                    ) AS snippet,
                    ts_rank_cd(nn.search_vec, websearch_to_tsquery('english', $1)) AS rank,
                    n.node_type::text AS node_type,
                    n.status::text AS status,
                    'note'::text AS match_source
                FROM node_notes nn
                JOIN nodes n ON n.id = nn.node_id
                WHERE n.id IN (SELECT id FROM valid_nodes)
                  AND nn.search_vec @@ websearch_to_tsquery('english', $1)

                UNION ALL

                -- Match from an attached task title
                SELECT
                    n.id,
                    n.title,
                    n.slug,
                    substring(nt.title FROM 1 FOR 200) AS snippet,
                    ts_rank_cd(
                        to_tsvector('english', nt.title),
                        websearch_to_tsquery('english', $1)
                    ) AS rank,
                    n.node_type::text AS node_type,
                    n.status::text AS status,
                    'task'::text AS match_source
                FROM node_tasks nt
                JOIN nodes n ON n.id = nt.node_id
                WHERE n.id IN (SELECT id FROM valid_nodes)
                  AND to_tsvector('english', nt.title) @@ websearch_to_tsquery('english', $1)
            ),
            best AS (
                SELECT DISTINCT ON (id)
                    id, title, slug, snippet, rank, node_type, status, match_source
                FROM candidates
                ORDER BY id, rank DESC
            )
            SELECT id, title, slug, snippet, rank, node_type, status, match_source
            FROM best
            ORDER BY rank DESC, title
            LIMIT $6 OFFSET $7
            "#,
        )
        .bind(q)
        .bind(type_filter)
        .bind(status_filter)
        .bind(tag_ids)
        .bind(and_mode)
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

    /// Fuzzy trigram search across node title/body, note bodies, and task titles.
    ///
    /// Uses `similarity()` / `ILIKE` predicates with the same `UNION ALL` +
    /// `DISTINCT ON` deduplication strategy as `fulltext_search`.
    #[allow(clippy::too_many_arguments)]
    async fn fuzzy_search(
        &self,
        q: &str,
        node_type: &Option<common::node::NodeType>,
        status: &Option<common::node::NodeStatus>,
        tag_ids: &[Uuid],
        and_mode: bool,
        page: u32,
        per_page: u32,
        offset: u32,
    ) -> Result<SearchResponse, EmberTroveError> {
        let type_filter = node_type.as_ref().map(node_type_to_str);
        let status_filter = status.as_ref().map(node_status_to_str);
        let like_pattern = format!("%{q}%");

        let count_row = sqlx::query_as::<_, CountRow>(
            r#"
            WITH valid_nodes AS (
                SELECT id FROM nodes
                WHERE ($3::text IS NULL OR node_type = $3::node_type)
                  AND ($4::text IS NULL OR status = $4::node_status)
                  AND (
                    array_length($5::uuid[], 1) IS NULL
                    OR id IN (
                        SELECT node_id FROM node_tags
                        WHERE tag_id = ANY($5::uuid[])
                        GROUP BY node_id
                        HAVING (NOT $6) OR COUNT(DISTINCT tag_id) = array_length($5::uuid[], 1)
                    )
                  )
            )
            SELECT COUNT(DISTINCT n.id)::bigint AS total
            FROM nodes n
            WHERE n.id IN (SELECT id FROM valid_nodes)
              AND (
                similarity(n.title, $1) > 0.1
                OR n.body ILIKE $2
                OR EXISTS (
                    SELECT 1 FROM node_notes nn
                    WHERE nn.node_id = n.id AND nn.body ILIKE $2
                )
                OR EXISTS (
                    SELECT 1 FROM node_tasks nt
                    WHERE nt.node_id = n.id
                      AND (similarity(nt.title, $1) > 0.1 OR nt.title ILIKE $2)
                )
              )
            "#,
        )
        .bind(q)
        .bind(&like_pattern)
        .bind(type_filter)
        .bind(status_filter)
        .bind(tag_ids)
        .bind(and_mode)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("fuzzy count failed: {e}")))?;

        let rows = sqlx::query_as::<_, SearchRow>(
            r#"
            WITH valid_nodes AS (
                SELECT id FROM nodes
                WHERE ($3::text IS NULL OR node_type = $3::node_type)
                  AND ($4::text IS NULL OR status = $4::node_status)
                  AND (
                    array_length($5::uuid[], 1) IS NULL
                    OR id IN (
                        SELECT node_id FROM node_tags
                        WHERE tag_id = ANY($5::uuid[])
                        GROUP BY node_id
                        HAVING (NOT $6) OR COUNT(DISTINCT tag_id) = array_length($5::uuid[], 1)
                    )
                  )
            ),
            candidates AS (
                -- Match from node title / body
                SELECT
                    n.id,
                    n.title,
                    n.slug,
                    CASE WHEN n.body ILIKE $2
                        THEN substring(n.body FROM 1 FOR 200)
                        ELSE NULL
                    END AS snippet,
                    GREATEST(similarity(n.title, $1), 0.0) AS rank,
                    n.node_type::text AS node_type,
                    n.status::text AS status,
                    'node'::text AS match_source
                FROM nodes n
                WHERE n.id IN (SELECT id FROM valid_nodes)
                  AND (similarity(n.title, $1) > 0.1 OR n.body ILIKE $2)

                UNION ALL

                -- Match from an attached note body
                SELECT
                    n.id,
                    n.title,
                    n.slug,
                    substring(nn.body FROM 1 FOR 200) AS snippet,
                    0.1::float4 AS rank,
                    n.node_type::text AS node_type,
                    n.status::text AS status,
                    'note'::text AS match_source
                FROM node_notes nn
                JOIN nodes n ON n.id = nn.node_id
                WHERE n.id IN (SELECT id FROM valid_nodes)
                  AND nn.body ILIKE $2

                UNION ALL

                -- Match from an attached task title
                SELECT
                    n.id,
                    n.title,
                    n.slug,
                    substring(nt.title FROM 1 FOR 200) AS snippet,
                    GREATEST(similarity(nt.title, $1), 0.0) AS rank,
                    n.node_type::text AS node_type,
                    n.status::text AS status,
                    'task'::text AS match_source
                FROM node_tasks nt
                JOIN nodes n ON n.id = nt.node_id
                WHERE n.id IN (SELECT id FROM valid_nodes)
                  AND (similarity(nt.title, $1) > 0.1 OR nt.title ILIKE $2)
            ),
            best AS (
                SELECT DISTINCT ON (id)
                    id, title, slug, snippet, rank, node_type, status, match_source
                FROM candidates
                ORDER BY id, rank DESC
            )
            SELECT id, title, slug, snippet, rank, node_type, status, match_source
            FROM best
            ORDER BY rank DESC, title
            LIMIT $7 OFFSET $8
            "#,
        )
        .bind(q)
        .bind(&like_pattern)
        .bind(type_filter)
        .bind(status_filter)
        .bind(tag_ids)
        .bind(and_mode)
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
