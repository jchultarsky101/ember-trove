use async_trait::async_trait;
use common::{EmberTroveError, graph::NodePosition, id::NodeId};
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait GraphRepo: Send + Sync {
    async fn list_positions(&self) -> Result<Vec<NodePosition>, EmberTroveError>;
    async fn upsert_position(&self, node_id: Uuid, x: f64, y: f64) -> Result<(), EmberTroveError>;
    async fn save_positions(
        &self,
        positions: &[(Uuid, f64, f64)],
    ) -> Result<(), EmberTroveError>;
}

pub struct PgGraphRepo {
    pool: PgPool,
}

impl PgGraphRepo {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct NodePositionRow {
    node_id: Uuid,
    x: f64,
    y: f64,
}

#[async_trait]
impl GraphRepo for PgGraphRepo {
    async fn list_positions(&self) -> Result<Vec<NodePosition>, EmberTroveError> {
        let rows = sqlx::query_as::<_, NodePositionRow>(
            "SELECT node_id, x, y FROM node_positions",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("list_positions failed: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|r| NodePosition {
                node_id: NodeId(r.node_id),
                x: r.x,
                y: r.y,
            })
            .collect())
    }

    async fn upsert_position(&self, node_id: Uuid, x: f64, y: f64) -> Result<(), EmberTroveError> {
        sqlx::query(
            "INSERT INTO node_positions (node_id, x, y)
             VALUES ($1, $2, $3)
             ON CONFLICT (node_id) DO UPDATE
             SET x = EXCLUDED.x, y = EXCLUDED.y, updated_at = now()",
        )
        .bind(node_id)
        .bind(x)
        .bind(y)
        .execute(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("upsert_position failed: {e}")))?;

        Ok(())
    }

    async fn save_positions(
        &self,
        positions: &[(Uuid, f64, f64)],
    ) -> Result<(), EmberTroveError> {
        if positions.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await
            .map_err(|e| EmberTroveError::Internal(format!("save_positions tx begin: {e}")))?;

        sqlx::query("DELETE FROM node_positions")
            .execute(&mut *tx)
            .await
            .map_err(|e| EmberTroveError::Internal(format!("save_positions clear: {e}")))?;

        for (node_id, x, y) in positions {
            sqlx::query(
                "INSERT INTO node_positions (node_id, x, y) VALUES ($1, $2, $3)
                 ON CONFLICT (node_id) DO UPDATE SET x = EXCLUDED.x, y = EXCLUDED.y",
            )
            .bind(node_id)
            .bind(*x)
            .bind(*y)
            .execute(&mut *tx)
            .await
            .map_err(|e| EmberTroveError::Internal(format!("save_positions insert: {e}")))?;
        }

        tx.commit().await
            .map_err(|e| EmberTroveError::Internal(format!("save_positions commit: {e}")))?;

        Ok(())
    }
}
