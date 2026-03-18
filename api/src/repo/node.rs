use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::{
    EmberTroveError,
    id::{NodeId, TagId},
    node::{CreateNodeRequest, Node, NodeListParams, NodeStatus, NodeType, NodeTitleEntry, UpdateNodeRequest},
    slug::slugify,
    tag::Tag,
};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

#[async_trait]
pub trait NodeRepo: Send + Sync {
    async fn create(&self, owner_id: &str, req: CreateNodeRequest)
    -> Result<Node, EmberTroveError>;

    async fn get(&self, id: NodeId) -> Result<Node, EmberTroveError>;

    async fn get_by_slug(&self, slug: &str) -> Result<Node, EmberTroveError>;

    async fn list(&self, params: NodeListParams) -> Result<(Vec<Node>, u64), EmberTroveError>;

    async fn update(&self, id: NodeId, req: UpdateNodeRequest) -> Result<Node, EmberTroveError>;

    async fn delete(&self, id: NodeId) -> Result<(), EmberTroveError>;

    async fn neighbors(&self, id: NodeId) -> Result<Vec<Node>, EmberTroveError>;

    async fn backlinks(&self, id: NodeId) -> Result<Vec<Node>, EmberTroveError>;

    /// Return all nodes as lightweight title entries, ordered by title.
    /// Used for wiki-link autocomplete and resolution.
    async fn list_titles(&self) -> Result<Vec<NodeTitleEntry>, EmberTroveError>;

    /// Find a node's ID by exact title (case-insensitive). Returns `None` if no
    /// node with that title exists.
    async fn find_id_by_title(&self, title: &str) -> Result<Option<NodeId>, EmberTroveError>;

    /// Fetch every node owned by `owner_id` with tags populated — used for backup.
    async fn list_all_for_owner(&self, owner_id: &str) -> Result<Vec<Node>, EmberTroveError>;
}

pub struct PgNodeRepo {
    pool: PgPool,
}

impl PgNodeRepo {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Intermediate row type for sqlx — maps DB columns with text-cast enums.
#[derive(sqlx::FromRow)]
struct NodeRow {
    id: Uuid,
    owner_id: String,
    node_type: String,
    title: String,
    slug: String,
    body: Option<String>,
    metadata: serde_json::Value,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl NodeRow {
    fn into_node(self) -> Result<Node, EmberTroveError> {
        Ok(Node {
            id: NodeId(self.id),
            owner_id: self.owner_id,
            node_type: parse_node_type(&self.node_type)?,
            title: self.title,
            slug: self.slug,
            body: self.body,
            metadata: self.metadata,
            status: parse_node_status(&self.status)?,
            tags: vec![], // populated separately if needed
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

/// Row type for batch tag fetch — includes the owning node_id.
#[derive(sqlx::FromRow)]
struct NodeTagRow {
    node_id: Uuid,
    id: Uuid,
    owner_id: String,
    name: String,
    color: String,
    created_at: DateTime<Utc>,
}

/// Fetch all tags for a slice of node IDs in one query.
/// Returns a map of node_id → Vec<Tag>.
async fn fetch_tags_for_nodes(
    pool: &PgPool,
    node_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<Tag>>, EmberTroveError> {
    if node_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = sqlx::query_as::<_, NodeTagRow>(
        r#"
        SELECT nt.node_id, t.id, t.owner_id, t.name, t.color, t.created_at
        FROM tags t
        JOIN node_tags nt ON nt.tag_id = t.id
        WHERE nt.node_id = ANY($1)
        ORDER BY t.name
        "#,
    )
    .bind(node_ids)
    .fetch_all(pool)
    .await
    .map_err(|e| EmberTroveError::Internal(format!("fetch tags for nodes failed: {e}")))?;

    let mut map: HashMap<Uuid, Vec<Tag>> = HashMap::new();
    for row in rows {
        map.entry(row.node_id).or_default().push(Tag {
            id: TagId(row.id),
            owner_id: row.owner_id,
            name: row.name,
            color: row.color,
            created_at: row.created_at,
        });
    }
    Ok(map)
}

fn parse_node_type(s: &str) -> Result<NodeType, EmberTroveError> {
    match s {
        "article" => Ok(NodeType::Article),
        "project" => Ok(NodeType::Project),
        "area" => Ok(NodeType::Area),
        "resource" => Ok(NodeType::Resource),
        "reference" => Ok(NodeType::Reference),
        other => Err(EmberTroveError::Internal(format!(
            "unknown node_type: {other}"
        ))),
    }
}

fn parse_node_status(s: &str) -> Result<NodeStatus, EmberTroveError> {
    match s {
        "draft" => Ok(NodeStatus::Draft),
        "published" => Ok(NodeStatus::Published),
        "archived" => Ok(NodeStatus::Archived),
        other => Err(EmberTroveError::Internal(format!(
            "unknown node_status: {other}"
        ))),
    }
}

fn node_type_to_str(t: &NodeType) -> &'static str {
    match t {
        NodeType::Article => "article",
        NodeType::Project => "project",
        NodeType::Area => "area",
        NodeType::Resource => "resource",
        NodeType::Reference => "reference",
    }
}

fn node_status_to_str(s: &NodeStatus) -> &'static str {
    match s {
        NodeStatus::Draft => "draft",
        NodeStatus::Published => "published",
        NodeStatus::Archived => "archived",
    }
}

#[async_trait]
impl NodeRepo for PgNodeRepo {
    async fn create(
        &self,
        owner_id: &str,
        req: CreateNodeRequest,
    ) -> Result<Node, EmberTroveError> {
        let slug = slugify(&req.title);
        let node_type_str = node_type_to_str(&req.node_type);
        let status_str = node_status_to_str(req.status.as_ref().unwrap_or(&NodeStatus::Draft));

        let row = sqlx::query_as::<_, NodeRow>(
            r#"
            INSERT INTO nodes (owner_id, node_type, title, slug, body, metadata, status)
            VALUES ($1, $2::node_type, $3, $4, $5, $6, $7::node_status)
            RETURNING id, owner_id, node_type::text, title, slug, body, metadata,
                      status::text, created_at, updated_at
            "#,
        )
        .bind(owner_id)
        .bind(node_type_str)
        .bind(&req.title)
        .bind(&slug)
        .bind(&req.body)
        .bind(&req.metadata)
        .bind(status_str)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db_err) if db_err.constraint() == Some("nodes_slug_key") => {
                EmberTroveError::AlreadyExists(format!("slug already exists: {slug}"))
            }
            _ => EmberTroveError::Internal(format!("create node failed: {e}")),
        })?;

        row.into_node()
    }

    async fn get(&self, id: NodeId) -> Result<Node, EmberTroveError> {
        let row = sqlx::query_as::<_, NodeRow>(
            r#"
            SELECT id, owner_id, node_type::text, title, slug, body, metadata,
                   status::text, created_at, updated_at
            FROM nodes WHERE id = $1
            "#,
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("get node failed: {e}")))?
        .ok_or_else(|| EmberTroveError::NotFound(format!("node {id} not found")))?;

        let mut node = row.into_node()?;
        let mut tag_map = fetch_tags_for_nodes(&self.pool, &[id.0]).await?;
        node.tags = tag_map.remove(&id.0).unwrap_or_default();
        Ok(node)
    }

    async fn get_by_slug(&self, slug: &str) -> Result<Node, EmberTroveError> {
        let row = sqlx::query_as::<_, NodeRow>(
            r#"
            SELECT id, owner_id, node_type::text, title, slug, body, metadata,
                   status::text, created_at, updated_at
            FROM nodes WHERE slug = $1
            "#,
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("get node by slug failed: {e}")))?
        .ok_or_else(|| EmberTroveError::NotFound(format!("node with slug '{slug}' not found")))?;

        row.into_node()
    }

    async fn list(&self, params: NodeListParams) -> Result<(Vec<Node>, u64), EmberTroveError> {
        let page = params.page.unwrap_or(1).max(1);
        let per_page = params.per_page.unwrap_or(50).min(200);
        let offset = (page - 1) * per_page;

        // Build dynamic count query with optional filters.
        let mut count_sql = String::from("SELECT COUNT(*) FROM nodes n");
        let mut data_sql = String::from(
            "SELECT n.id, n.owner_id, n.node_type::text, n.title, n.slug, n.body, \
             n.metadata, n.status::text, n.created_at, n.updated_at FROM nodes n",
        );

        let mut conditions: Vec<String> = Vec::new();
        let mut param_idx = 1i32;

        if params.tag_id.is_some() {
            count_sql.push_str(" JOIN node_tags nt ON nt.node_id = n.id");
            data_sql.push_str(" JOIN node_tags nt ON nt.node_id = n.id");
            conditions.push(format!("nt.tag_id = ${param_idx}"));
            param_idx += 1;
        }

        if params.node_type.is_some() {
            conditions.push(format!("n.node_type = ${param_idx}::node_type"));
            param_idx += 1;
        }

        if params.status.is_some() {
            conditions.push(format!("n.status = ${param_idx}::node_status"));
            param_idx += 1;
        }

        if params.owner_id.is_some() {
            conditions.push(format!("n.owner_id = ${param_idx}"));
            param_idx += 1;
        }

        if !conditions.is_empty() {
            let where_clause = " WHERE ".to_owned() + &conditions.join(" AND ");
            count_sql.push_str(&where_clause);
            data_sql.push_str(&where_clause);
        }

        // Build count query with bindings.
        let mut count_query = sqlx::query_scalar(&count_sql);
        if let Some(ref tag_id) = params.tag_id {
            count_query = count_query.bind(tag_id.0);
        }
        if let Some(ref node_type) = params.node_type {
            count_query = count_query.bind(node_type_to_str(node_type));
        }
        if let Some(ref status) = params.status {
            count_query = count_query.bind(node_status_to_str(status));
        }
        if let Some(ref owner_id) = params.owner_id {
            count_query = count_query.bind(owner_id.as_str());
        }

        let total: i64 = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| EmberTroveError::Internal(format!("count query failed: {e}")))?;

        // Get paginated data.
        data_sql.push_str(&format!(
            " ORDER BY n.updated_at DESC LIMIT ${param_idx} OFFSET ${}",
            param_idx + 1
        ));

        let mut query = sqlx::query_as::<_, NodeRow>(&data_sql);

        if let Some(ref tag_id) = params.tag_id {
            query = query.bind(tag_id.0);
        }
        if let Some(ref node_type) = params.node_type {
            query = query.bind(node_type_to_str(node_type));
        }
        if let Some(ref status) = params.status {
            query = query.bind(node_status_to_str(status));
        }
        if let Some(ref owner_id) = params.owner_id {
            query = query.bind(owner_id.as_str());
        }

        query = query.bind(per_page as i64).bind(offset as i64);

        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| EmberTroveError::Internal(format!("list nodes failed: {e}")))?;

        let mut nodes = rows.into_iter().map(NodeRow::into_node).collect::<Result<Vec<_>, _>>()?;

        // Batch-fetch tags for all returned nodes in a single query.
        let node_ids: Vec<Uuid> = nodes.iter().map(|n| n.id.0).collect();
        let mut tag_map = fetch_tags_for_nodes(&self.pool, &node_ids).await?;
        for node in &mut nodes {
            node.tags = tag_map.remove(&node.id.0).unwrap_or_default();
        }

        Ok((nodes, total as u64))
    }

    async fn update(&self, id: NodeId, req: UpdateNodeRequest) -> Result<Node, EmberTroveError> {
        let status_str = req.status.as_ref().map(node_status_to_str);

        let row = sqlx::query_as::<_, NodeRow>(
            r#"
            UPDATE nodes SET
                title    = COALESCE($2, title),
                body     = COALESCE($3, body),
                metadata = COALESCE($4, metadata),
                status   = COALESCE($5::node_status, status)
            WHERE id = $1
            RETURNING id, owner_id, node_type::text, title, slug, body, metadata,
                      status::text, created_at, updated_at
            "#,
        )
        .bind(id.0)
        .bind(&req.title)
        .bind(&req.body)
        .bind(&req.metadata)
        .bind(status_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("update node failed: {e}")))?
        .ok_or_else(|| EmberTroveError::NotFound(format!("node {id} not found")))?;

        row.into_node()
    }

    async fn delete(&self, id: NodeId) -> Result<(), EmberTroveError> {
        let result = sqlx::query("DELETE FROM nodes WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| EmberTroveError::Internal(format!("delete node failed: {e}")))?;

        if result.rows_affected() == 0 {
            return Err(EmberTroveError::NotFound(format!("node {id} not found")));
        }

        Ok(())
    }

    async fn neighbors(&self, id: NodeId) -> Result<Vec<Node>, EmberTroveError> {
        let rows = sqlx::query_as::<_, NodeRow>(
            r#"
            SELECT n.id, n.owner_id, n.node_type::text, n.title, n.slug, n.body,
                   n.metadata, n.status::text, n.created_at, n.updated_at
            FROM nodes n
            JOIN edges e ON e.target_id = n.id
            WHERE e.source_id = $1
            ORDER BY n.title
            "#,
        )
        .bind(id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("neighbors query failed: {e}")))?;

        rows.into_iter().map(NodeRow::into_node).collect()
    }

    async fn backlinks(&self, id: NodeId) -> Result<Vec<Node>, EmberTroveError> {
        let rows = sqlx::query_as::<_, NodeRow>(
            r#"
            SELECT n.id, n.owner_id, n.node_type::text, n.title, n.slug, n.body,
                   n.metadata, n.status::text, n.created_at, n.updated_at
            FROM nodes n
            JOIN edges e ON e.source_id = n.id
            WHERE e.target_id = $1
            ORDER BY n.title
            "#,
        )
        .bind(id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("backlinks query failed: {e}")))?;

        rows.into_iter().map(NodeRow::into_node).collect()
    }

    async fn list_titles(&self) -> Result<Vec<NodeTitleEntry>, EmberTroveError> {
        #[derive(sqlx::FromRow)]
        struct TitleRow {
            id: Uuid,
            title: String,
            slug: String,
        }

        let rows = sqlx::query_as::<_, TitleRow>(
            "SELECT id, title, slug FROM nodes ORDER BY title",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("list_titles failed: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|r| NodeTitleEntry {
                id: NodeId(r.id),
                title: r.title,
                slug: r.slug,
            })
            .collect())
    }

    async fn find_id_by_title(&self, title: &str) -> Result<Option<NodeId>, EmberTroveError> {
        #[derive(sqlx::FromRow)]
        struct IdRow {
            id: Uuid,
        }

        let row = sqlx::query_as::<_, IdRow>(
            "SELECT id FROM nodes WHERE LOWER(title) = LOWER($1) LIMIT 1",
        )
        .bind(title)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("find_id_by_title failed: {e}")))?;

        Ok(row.map(|r| NodeId(r.id)))
    }

    async fn list_all_for_owner(&self, owner_id: &str) -> Result<Vec<Node>, EmberTroveError> {
        let rows = sqlx::query_as::<_, NodeRow>(
            r#"
            SELECT id, owner_id, node_type::text, title, slug, body, metadata,
                   status::text, created_at, updated_at
            FROM nodes
            WHERE owner_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("list_all_for_owner failed: {e}")))?;

        let mut nodes = rows
            .into_iter()
            .map(NodeRow::into_node)
            .collect::<Result<Vec<_>, _>>()?;

        let node_ids: Vec<Uuid> = nodes.iter().map(|n| n.id.0).collect();
        let mut tag_map = fetch_tags_for_nodes(&self.pool, &node_ids).await?;
        for node in &mut nodes {
            node.tags = tag_map.remove(&node.id.0).unwrap_or_default();
        }

        Ok(nodes)
    }
}
