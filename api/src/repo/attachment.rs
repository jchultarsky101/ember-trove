use async_trait::async_trait;
use common::{
    EmberTroveError,
    attachment::Attachment,
    id::{AttachmentId, NodeId},
};
use sqlx::PgPool;

#[async_trait]
pub trait AttachmentRepo: Send + Sync {
    async fn create(
        &self,
        node_id: NodeId,
        filename: &str,
        content_type: &str,
        size_bytes: i64,
        s3_key: &str,
    ) -> Result<Attachment, EmberTroveError>;

    async fn list(&self, node_id: NodeId) -> Result<Vec<Attachment>, EmberTroveError>;

    async fn get(&self, id: AttachmentId) -> Result<Attachment, EmberTroveError>;

    async fn delete(&self, id: AttachmentId) -> Result<String, EmberTroveError>;
}

pub struct PgAttachmentRepo {
    pool: PgPool,
}

impl PgAttachmentRepo {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AttachmentRepo for PgAttachmentRepo {
    async fn create(
        &self,
        _node_id: NodeId,
        _filename: &str,
        _content_type: &str,
        _size_bytes: i64,
        _s3_key: &str,
    ) -> Result<Attachment, EmberTroveError> {
        let _ = &self.pool;
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }

    async fn list(&self, _node_id: NodeId) -> Result<Vec<Attachment>, EmberTroveError> {
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }

    async fn get(&self, _id: AttachmentId) -> Result<Attachment, EmberTroveError> {
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }

    async fn delete(&self, _id: AttachmentId) -> Result<String, EmberTroveError> {
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }
}
