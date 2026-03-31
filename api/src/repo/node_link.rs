use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::{
    EmberTroveError,
    id::{NodeId, NodeLinkId},
    node_link::{CreateNodeLinkRequest, NodeLink, UpdateNodeLinkRequest},
};
use sqlx::PgPool;
use uuid::Uuid;

// ── Trait ──────────────────────────────────────────────────────────────────────

#[async_trait]
pub trait NodeLinkRepo: Send + Sync {
    async fn list(&self, node_id: NodeId) -> Result<Vec<NodeLink>, EmberTroveError>;
    async fn create(
        &self,
        node_id: NodeId,
        req: CreateNodeLinkRequest,
    ) -> Result<NodeLink, EmberTroveError>;
    async fn update(
        &self,
        id: NodeLinkId,
        req: UpdateNodeLinkRequest,
    ) -> Result<NodeLink, EmberTroveError>;
    async fn delete(&self, id: NodeLinkId) -> Result<(), EmberTroveError>;
}

// ── PgNodeLinkRepo ─────────────────────────────────────────────────────────────

pub struct PgNodeLinkRepo {
    pool: PgPool,
}

impl PgNodeLinkRepo {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct NodeLinkRow {
    id: Uuid,
    node_id: Uuid,
    name: String,
    url: String,
    created_at: DateTime<Utc>,
}

impl NodeLinkRow {
    fn into_link(self) -> NodeLink {
        NodeLink {
            id: NodeLinkId(self.id),
            node_id: NodeId(self.node_id),
            name: self.name,
            url: self.url,
            created_at: self.created_at,
        }
    }
}

#[async_trait]
impl NodeLinkRepo for PgNodeLinkRepo {
    async fn list(&self, node_id: NodeId) -> Result<Vec<NodeLink>, EmberTroveError> {
        let rows = sqlx::query_as::<_, NodeLinkRow>(
            "SELECT id, node_id, name, url, created_at
             FROM node_links
             WHERE node_id = $1
             ORDER BY created_at ASC",
        )
        .bind(node_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("list node_links failed: {e}")))?;

        Ok(rows.into_iter().map(NodeLinkRow::into_link).collect())
    }

    async fn create(
        &self,
        node_id: NodeId,
        req: CreateNodeLinkRequest,
    ) -> Result<NodeLink, EmberTroveError> {
        let row = sqlx::query_as::<_, NodeLinkRow>(
            "INSERT INTO node_links (node_id, name, url)
             VALUES ($1, $2, $3)
             RETURNING id, node_id, name, url, created_at",
        )
        .bind(node_id.0)
        .bind(&req.name)
        .bind(&req.url)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("create node_link failed: {e}")))?;

        Ok(row.into_link())
    }

    async fn update(
        &self,
        id: NodeLinkId,
        req: UpdateNodeLinkRequest,
    ) -> Result<NodeLink, EmberTroveError> {
        let row = sqlx::query_as::<_, NodeLinkRow>(
            "UPDATE node_links
             SET name = COALESCE($1, name),
                 url  = COALESCE($2, url)
             WHERE id = $3
             RETURNING id, node_id, name, url, created_at",
        )
        .bind(req.name.as_deref())
        .bind(req.url.as_deref())
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("update node_link failed: {e}")))?
        .ok_or_else(|| EmberTroveError::NotFound("node link not found".to_string()))?;

        Ok(row.into_link())
    }

    async fn delete(&self, id: NodeLinkId) -> Result<(), EmberTroveError> {
        sqlx::query("DELETE FROM node_links WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| EmberTroveError::Internal(format!("delete node_link failed: {e}")))?;
        Ok(())
    }
}
