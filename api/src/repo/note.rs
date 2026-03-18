use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::{
    EmberTroveError,
    id::{NodeId, NoteId},
    note::{CreateNoteRequest, FeedNote, Note},
};
use sqlx::PgPool;
use uuid::Uuid;

// ── Trait ──────────────────────────────────────────────────────────────────────

#[async_trait]
pub trait NoteRepo: Send + Sync {
    /// Create a new note on a node. Only the node owner should call this.
    async fn create(
        &self,
        node_id: NodeId,
        owner_id: &str,
        req: CreateNoteRequest,
    ) -> Result<Note, EmberTroveError>;

    /// All notes for a node, newest first.
    async fn list_for_node(&self, node_id: NodeId) -> Result<Vec<Note>, EmberTroveError>;

    /// All notes by a given owner, newest first, with node titles (central feed).
    async fn feed_for_owner(&self, owner_id: &str) -> Result<Vec<FeedNote>, EmberTroveError>;

    /// All notes across all owners, newest first, with node titles.
    /// Used in single-user mode where every authenticated user sees everything.
    async fn feed_all(&self) -> Result<Vec<FeedNote>, EmberTroveError>;

    /// All notes across all owners — used for full backup.
    async fn list_all(&self) -> Result<Vec<Note>, EmberTroveError>;
}

// ── PgNoteRepo ─────────────────────────────────────────────────────────────────

pub struct PgNoteRepo {
    pool: PgPool,
}

impl PgNoteRepo {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct NoteRow {
    id: Uuid,
    node_id: Uuid,
    owner_id: String,
    body: String,
    created_at: DateTime<Utc>,
}

impl NoteRow {
    fn into_note(self) -> Note {
        Note {
            id: NoteId(self.id),
            node_id: NodeId(self.node_id),
            owner_id: self.owner_id,
            body: self.body,
            created_at: self.created_at,
        }
    }
}

#[async_trait]
impl NoteRepo for PgNoteRepo {
    async fn create(
        &self,
        node_id: NodeId,
        owner_id: &str,
        req: CreateNoteRequest,
    ) -> Result<Note, EmberTroveError> {
        let row = sqlx::query_as::<_, NoteRow>(
            r#"
            INSERT INTO node_notes (node_id, owner_id, body)
            VALUES ($1, $2, $3)
            RETURNING id, node_id, owner_id, body, created_at
            "#,
        )
        .bind(node_id.0)
        .bind(owner_id)
        .bind(&req.body)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("create note failed: {e}")))?;

        Ok(row.into_note())
    }

    async fn list_for_node(&self, node_id: NodeId) -> Result<Vec<Note>, EmberTroveError> {
        let rows = sqlx::query_as::<_, NoteRow>(
            r#"
            SELECT id, node_id, owner_id, body, created_at
            FROM node_notes
            WHERE node_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(node_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("list notes failed: {e}")))?;

        Ok(rows.into_iter().map(NoteRow::into_note).collect())
    }

    async fn list_all(&self) -> Result<Vec<Note>, EmberTroveError> {
        let rows = sqlx::query_as::<_, NoteRow>(
            r#"
            SELECT id, node_id, owner_id, body, created_at
            FROM node_notes
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("list_all notes failed: {e}")))?;

        Ok(rows.into_iter().map(NoteRow::into_note).collect())
    }

    async fn feed_for_owner(&self, owner_id: &str) -> Result<Vec<FeedNote>, EmberTroveError> {
        #[derive(sqlx::FromRow)]
        struct FeedRow {
            id: Uuid,
            node_id: Uuid,
            owner_id: String,
            body: String,
            created_at: DateTime<Utc>,
            node_title: String,
        }

        let rows = sqlx::query_as::<_, FeedRow>(
            r#"
            SELECT
                n.id, n.node_id, n.owner_id, n.body, n.created_at,
                nd.title AS node_title
            FROM node_notes n
            JOIN nodes nd ON nd.id = n.node_id
            WHERE n.owner_id = $1
            ORDER BY n.created_at DESC
            "#,
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("feed notes failed: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|r| FeedNote {
                note: Note {
                    id: NoteId(r.id),
                    node_id: NodeId(r.node_id),
                    owner_id: r.owner_id,
                    body: r.body,
                    created_at: r.created_at,
                },
                node_title: r.node_title,
            })
            .collect())
    }

    async fn feed_all(&self) -> Result<Vec<FeedNote>, EmberTroveError> {
        #[derive(sqlx::FromRow)]
        struct FeedRow {
            id: Uuid,
            node_id: Uuid,
            owner_id: String,
            body: String,
            created_at: DateTime<Utc>,
            node_title: String,
        }

        let rows = sqlx::query_as::<_, FeedRow>(
            r#"
            SELECT
                n.id, n.node_id, n.owner_id, n.body, n.created_at,
                nd.title AS node_title
            FROM node_notes n
            JOIN nodes nd ON nd.id = n.node_id
            ORDER BY n.created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("feed_all notes failed: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|r| FeedNote {
                note: Note {
                    id: NoteId(r.id),
                    node_id: NodeId(r.node_id),
                    owner_id: r.owner_id,
                    body: r.body,
                    created_at: r.created_at,
                },
                node_title: r.node_title,
            })
            .collect())
    }
}
