use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::{
    EmberTroveError,
    id::TemplateId,
    node::NodeType,
    template::{CreateTemplateRequest, NodeTemplate, UpdateTemplateRequest},
};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[async_trait]
pub trait TemplateRepo: Send + Sync + 'static {
    /// Return all templates, ordered by name.
    async fn list(&self) -> Result<Vec<NodeTemplate>, EmberTroveError>;

    /// Fetch a single template by ID.
    async fn get(&self, id: TemplateId) -> Result<NodeTemplate, EmberTroveError>;

    /// Create a new template.
    async fn create(
        &self,
        created_by: &str,
        req: CreateTemplateRequest,
    ) -> Result<NodeTemplate, EmberTroveError>;

    /// Update an existing template.
    async fn update(
        &self,
        id: TemplateId,
        req: UpdateTemplateRequest,
    ) -> Result<NodeTemplate, EmberTroveError>;

    /// Delete a template.
    async fn delete(&self, id: TemplateId) -> Result<(), EmberTroveError>;
}

// ── Internal row type ─────────────────────────────────────────────────────────

#[derive(FromRow)]
struct TemplateRow {
    id: Uuid,
    name: String,
    description: Option<String>,
    node_type: String,
    body: String,
    created_by: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TemplateRow {
    fn into_template(self) -> Result<NodeTemplate, EmberTroveError> {
        let node_type = parse_node_type(&self.node_type)?;
        Ok(NodeTemplate {
            id: TemplateId(self.id),
            name: self.name,
            description: self.description,
            node_type,
            body: self.body,
            created_by: self.created_by,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

fn parse_node_type(s: &str) -> Result<NodeType, EmberTroveError> {
    match s {
        "article" => Ok(NodeType::Article),
        "project" => Ok(NodeType::Project),
        "area" => Ok(NodeType::Area),
        "resource" => Ok(NodeType::Resource),
        "reference" => Ok(NodeType::Reference),
        other => Err(EmberTroveError::Internal(format!(
            "unknown node_type in template: {other}"
        ))),
    }
}

fn node_type_str(nt: &NodeType) -> &'static str {
    match nt {
        NodeType::Article => "article",
        NodeType::Project => "project",
        NodeType::Area => "area",
        NodeType::Resource => "resource",
        NodeType::Reference => "reference",
    }
}

// ── Postgres implementation ───────────────────────────────────────────────────

pub struct PgTemplateRepo {
    pool: PgPool,
}

impl PgTemplateRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TemplateRepo for PgTemplateRepo {
    async fn list(&self) -> Result<Vec<NodeTemplate>, EmberTroveError> {
        let rows = sqlx::query_as::<_, TemplateRow>(
            "SELECT id, name, description, node_type, body, created_by, created_at, updated_at \
             FROM node_templates ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("list templates failed: {e}")))?;

        rows.into_iter().map(TemplateRow::into_template).collect()
    }

    async fn get(&self, id: TemplateId) -> Result<NodeTemplate, EmberTroveError> {
        sqlx::query_as::<_, TemplateRow>(
            "SELECT id, name, description, node_type, body, created_by, created_at, updated_at \
             FROM node_templates WHERE id = $1",
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("get template failed: {e}")))?
        .ok_or_else(|| EmberTroveError::NotFound("template not found".to_string()))?
        .into_template()
    }

    async fn create(
        &self,
        created_by: &str,
        req: CreateTemplateRequest,
    ) -> Result<NodeTemplate, EmberTroveError> {
        let row = sqlx::query_as::<_, TemplateRow>(
            "INSERT INTO node_templates (name, description, node_type, body, created_by) \
             VALUES ($1, $2, $3, $4, $5) \
             RETURNING id, name, description, node_type, body, created_by, created_at, updated_at",
        )
        .bind(&req.name)
        .bind(req.description.as_deref())
        .bind(node_type_str(&req.node_type))
        .bind(&req.body)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("create template failed: {e}")))?;

        row.into_template()
    }

    async fn update(
        &self,
        id: TemplateId,
        req: UpdateTemplateRequest,
    ) -> Result<NodeTemplate, EmberTroveError> {
        let row = sqlx::query_as::<_, TemplateRow>(
            "UPDATE node_templates \
             SET name = $1, description = $2, node_type = $3, body = $4, updated_at = now() \
             WHERE id = $5 \
             RETURNING id, name, description, node_type, body, created_by, created_at, updated_at",
        )
        .bind(&req.name)
        .bind(req.description.as_deref())
        .bind(node_type_str(&req.node_type))
        .bind(&req.body)
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("update template failed: {e}")))?
        .ok_or_else(|| EmberTroveError::NotFound("template not found".to_string()))?;

        row.into_template()
    }

    async fn delete(&self, id: TemplateId) -> Result<(), EmberTroveError> {
        let result = sqlx::query("DELETE FROM node_templates WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| EmberTroveError::Internal(format!("delete template failed: {e}")))?;

        if result.rows_affected() == 0 {
            return Err(EmberTroveError::NotFound("template not found".to_string()));
        }
        Ok(())
    }
}
