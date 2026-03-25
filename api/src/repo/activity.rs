use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::{
    EmberTroveError,
    activity::{ActivityAction, ActivityEntry},
    id::{ActivityId, NodeId},
};
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait ActivityRepo: Send + Sync {
    /// Append one entry. Fire-and-forget — callers ignore errors.
    async fn record(
        &self,
        node_id: NodeId,
        subject_id: &str,
        action: ActivityAction,
        metadata: serde_json::Value,
    ) -> Result<(), EmberTroveError>;

    /// Return the most recent `limit` entries for a node (newest first).
    async fn list(
        &self,
        node_id: NodeId,
        limit: i64,
    ) -> Result<Vec<ActivityEntry>, EmberTroveError>;
}

pub struct PgActivityRepo {
    pool: PgPool,
}

impl PgActivityRepo {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ActivityRow {
    id: Uuid,
    node_id: Uuid,
    subject_id: String,
    action: String,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
}

impl ActivityRow {
    fn into_entry(self) -> Result<ActivityEntry, EmberTroveError> {
        let action = ActivityAction::from_db_str(&self.action).ok_or_else(|| {
            EmberTroveError::Internal(format!("unknown activity action: {}", self.action))
        })?;
        Ok(ActivityEntry {
            id: ActivityId(self.id),
            node_id: NodeId(self.node_id),
            subject_id: self.subject_id,
            action,
            metadata: self.metadata,
            created_at: self.created_at,
        })
    }
}

#[async_trait]
impl ActivityRepo for PgActivityRepo {
    async fn record(
        &self,
        node_id: NodeId,
        subject_id: &str,
        action: ActivityAction,
        metadata: serde_json::Value,
    ) -> Result<(), EmberTroveError> {
        sqlx::query(
            r#"
            INSERT INTO activity_log (node_id, subject_id, action, metadata)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(node_id.inner())
        .bind(subject_id)
        .bind(action.as_str())
        .bind(metadata)
        .execute(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("record activity failed: {e}")))?;
        Ok(())
    }

    async fn list(
        &self,
        node_id: NodeId,
        limit: i64,
    ) -> Result<Vec<ActivityEntry>, EmberTroveError> {
        let rows = sqlx::query_as::<_, ActivityRow>(
            r#"
            SELECT id, node_id, subject_id, action, metadata, created_at
            FROM activity_log
            WHERE node_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(node_id.inner())
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("list activity failed: {e}")))?;

        rows.into_iter().map(|r| r.into_entry()).collect()
    }
}
