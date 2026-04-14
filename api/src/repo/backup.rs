use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::{EmberTroveError, backup::BackupJob};
use sqlx::PgPool;
use uuid::Uuid;

// ── Trait ──────────────────────────────────────────────────────────────────────

#[async_trait]
pub trait BackupRepo: Send + Sync {
    /// Persist a backup job record and return the stored row.
    #[allow(clippy::too_many_arguments)]
    async fn create(
        &self,
        created_by: &str,
        s3_key: &str,
        size_bytes: i64,
        node_count: i32,
        edge_count: i32,
        tag_count: i32,
        note_count: i32,
        task_count: i32,
        attachment_count: i32,
        comment: Option<&str>,
    ) -> Result<BackupJob, EmberTroveError>;

    /// List all backup jobs for the given owner, newest first.
    async fn list_for_owner(&self, owner_id: &str) -> Result<Vec<BackupJob>, EmberTroveError>;

    /// Fetch a single backup job by id.
    async fn get(&self, id: Uuid) -> Result<BackupJob, EmberTroveError>;

    /// Delete a backup job record.
    async fn delete(&self, id: Uuid) -> Result<(), EmberTroveError>;
}

// ── PgBackupRepo ───────────────────────────────────────────────────────────────

pub struct PgBackupRepo {
    pool: PgPool,
}

impl PgBackupRepo {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct BackupJobRow {
    id: Uuid,
    created_by: String,
    created_at: DateTime<Utc>,
    size_bytes: i64,
    s3_key: String,
    node_count: i32,
    edge_count: i32,
    tag_count: i32,
    note_count: i32,
    task_count: i32,
    attachment_count: i32,
    comment: Option<String>,
}

impl From<BackupJobRow> for BackupJob {
    fn from(r: BackupJobRow) -> Self {
        Self {
            id: r.id,
            created_by: r.created_by,
            created_at: r.created_at,
            size_bytes: r.size_bytes,
            s3_key: r.s3_key,
            node_count: r.node_count,
            edge_count: r.edge_count,
            tag_count: r.tag_count,
            note_count: r.note_count,
            task_count: r.task_count,
            attachment_count: r.attachment_count,
            comment: r.comment,
        }
    }
}

#[async_trait]
impl BackupRepo for PgBackupRepo {
    #[allow(clippy::too_many_arguments)]
    async fn create(
        &self,
        created_by: &str,
        s3_key: &str,
        size_bytes: i64,
        node_count: i32,
        edge_count: i32,
        tag_count: i32,
        note_count: i32,
        task_count: i32,
        attachment_count: i32,
        comment: Option<&str>,
    ) -> Result<BackupJob, EmberTroveError> {
        let row = sqlx::query_as::<_, BackupJobRow>(
            r#"
            INSERT INTO backup_jobs
                (created_by, s3_key, size_bytes, node_count, edge_count, tag_count,
                 note_count, task_count, attachment_count, comment)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, created_by, created_at, size_bytes, s3_key,
                      node_count, edge_count, tag_count, note_count, task_count,
                      attachment_count, comment
            "#,
        )
        .bind(created_by)
        .bind(s3_key)
        .bind(size_bytes)
        .bind(node_count)
        .bind(edge_count)
        .bind(tag_count)
        .bind(note_count)
        .bind(task_count)
        .bind(attachment_count)
        .bind(comment)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("backup create failed: {e}")))?;

        Ok(row.into())
    }

    async fn list_for_owner(&self, owner_id: &str) -> Result<Vec<BackupJob>, EmberTroveError> {
        let rows = sqlx::query_as::<_, BackupJobRow>(
            r#"
            SELECT id, created_by, created_at, size_bytes, s3_key,
                   node_count, edge_count, tag_count, note_count, task_count,
                   attachment_count, comment
            FROM backup_jobs
            WHERE created_by = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("backup list failed: {e}")))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn get(&self, id: Uuid) -> Result<BackupJob, EmberTroveError> {
        let row = sqlx::query_as::<_, BackupJobRow>(
            r#"
            SELECT id, created_by, created_at, size_bytes, s3_key,
                   node_count, edge_count, tag_count, note_count, task_count,
                   attachment_count, comment
            FROM backup_jobs
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("backup get failed: {e}")))?
        .ok_or_else(|| EmberTroveError::NotFound(format!("backup job {id} not found")))?;

        Ok(row.into())
    }

    async fn delete(&self, id: Uuid) -> Result<(), EmberTroveError> {
        let result = sqlx::query("DELETE FROM backup_jobs WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| EmberTroveError::Internal(format!("backup delete failed: {e}")))?;

        if result.rows_affected() == 0 {
            return Err(EmberTroveError::NotFound(format!(
                "backup job {id} not found"
            )));
        }
        Ok(())
    }
}
