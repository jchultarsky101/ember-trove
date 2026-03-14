use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::{
    EmberTroveError,
    edge::{CreateEdgeRequest, Edge, EdgeType},
    id::{EdgeId, NodeId},
};
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait EdgeRepo: Send + Sync {
    async fn create(&self, req: CreateEdgeRequest) -> Result<Edge, EmberTroveError>;
    async fn delete(&self, id: EdgeId) -> Result<(), EmberTroveError>;
    async fn list_for_node(&self, node_id: NodeId) -> Result<Vec<Edge>, EmberTroveError>;
}

pub struct PgEdgeRepo {
    pool: PgPool,
}

impl PgEdgeRepo {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct EdgeRow {
    id: Uuid,
    source_id: Uuid,
    target_id: Uuid,
    edge_type: String,
    label: Option<String>,
    created_at: DateTime<Utc>,
}

impl EdgeRow {
    fn into_edge(self) -> Result<Edge, EmberTroveError> {
        Ok(Edge {
            id: EdgeId(self.id),
            source_id: NodeId(self.source_id),
            target_id: NodeId(self.target_id),
            edge_type: parse_edge_type(&self.edge_type)?,
            label: self.label,
            created_at: self.created_at,
        })
    }
}

fn parse_edge_type(s: &str) -> Result<EdgeType, EmberTroveError> {
    match s {
        "references" => Ok(EdgeType::References),
        "contains" => Ok(EdgeType::Contains),
        "related_to" => Ok(EdgeType::RelatedTo),
        "depends_on" => Ok(EdgeType::DependsOn),
        "derived_from" => Ok(EdgeType::DerivedFrom),
        other => Err(EmberTroveError::Internal(format!(
            "unknown edge_type: {other}"
        ))),
    }
}

fn edge_type_to_str(t: &EdgeType) -> &'static str {
    match t {
        EdgeType::References => "references",
        EdgeType::Contains => "contains",
        EdgeType::RelatedTo => "related_to",
        EdgeType::DependsOn => "depends_on",
        EdgeType::DerivedFrom => "derived_from",
    }
}

#[async_trait]
impl EdgeRepo for PgEdgeRepo {
    async fn create(&self, req: CreateEdgeRequest) -> Result<Edge, EmberTroveError> {
        if req.source_id == req.target_id {
            return Err(EmberTroveError::Validation(
                "self-loops are not allowed".to_string(),
            ));
        }

        let edge_type_str = edge_type_to_str(&req.edge_type);

        let row = sqlx::query_as::<_, EdgeRow>(
            r#"
            INSERT INTO edges (source_id, target_id, edge_type, label)
            VALUES ($1, $2, $3::edge_type, $4)
            RETURNING id, source_id, target_id, edge_type::text, label, created_at
            "#,
        )
        .bind(req.source_id.0)
        .bind(req.target_id.0)
        .bind(edge_type_str)
        .bind(&req.label)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db_err) => {
                if db_err.constraint() == Some("edges_no_self_loop") {
                    EmberTroveError::Validation("self-loops are not allowed".to_string())
                } else if db_err.is_foreign_key_violation() {
                    EmberTroveError::NotFound("source or target node not found".to_string())
                } else {
                    EmberTroveError::Internal(format!("create edge failed: {e}"))
                }
            }
            _ => EmberTroveError::Internal(format!("create edge failed: {e}")),
        })?;

        row.into_edge()
    }

    async fn delete(&self, id: EdgeId) -> Result<(), EmberTroveError> {
        let result = sqlx::query("DELETE FROM edges WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| EmberTroveError::Internal(format!("delete edge failed: {e}")))?;

        if result.rows_affected() == 0 {
            return Err(EmberTroveError::NotFound(format!("edge {id} not found")));
        }

        Ok(())
    }

    async fn list_for_node(&self, node_id: NodeId) -> Result<Vec<Edge>, EmberTroveError> {
        let rows = sqlx::query_as::<_, EdgeRow>(
            r#"
            SELECT id, source_id, target_id, edge_type::text, label, created_at
            FROM edges
            WHERE source_id = $1 OR target_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(node_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("list edges failed: {e}")))?;

        rows.into_iter().map(EdgeRow::into_edge).collect()
    }
}
