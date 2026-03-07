use async_trait::async_trait;
use common::{
    tag::{CreateTagRequest, Tag, UpdateTagRequest},
    id::{NodeId, TagId},
    EmberTroveError,
};
use sqlx::PgPool;

#[async_trait]
pub trait TagRepo: Send + Sync {
    async fn create(&self, owner_id: &str, req: CreateTagRequest)
        -> Result<Tag, EmberTroveError>;
    async fn list(&self, owner_id: &str) -> Result<Vec<Tag>, EmberTroveError>;
    async fn update(&self, id: TagId, req: UpdateTagRequest) -> Result<Tag, EmberTroveError>;
    async fn delete(&self, id: TagId) -> Result<(), EmberTroveError>;
    async fn attach(&self, node_id: NodeId, tag_id: TagId) -> Result<(), EmberTroveError>;
    async fn detach(&self, node_id: NodeId, tag_id: TagId) -> Result<(), EmberTroveError>;
}

pub struct PgTagRepo {
    pool: PgPool,
}

impl PgTagRepo {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TagRepo for PgTagRepo {
    async fn create(
        &self,
        _owner_id: &str,
        _req: CreateTagRequest,
    ) -> Result<Tag, EmberTroveError> {
        let _ = &self.pool;
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }

    async fn list(&self, _owner_id: &str) -> Result<Vec<Tag>, EmberTroveError> {
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }

    async fn update(&self, _id: TagId, _req: UpdateTagRequest) -> Result<Tag, EmberTroveError> {
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }

    async fn delete(&self, _id: TagId) -> Result<(), EmberTroveError> {
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }

    async fn attach(&self, _node_id: NodeId, _tag_id: TagId) -> Result<(), EmberTroveError> {
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }

    async fn detach(&self, _node_id: NodeId, _tag_id: TagId) -> Result<(), EmberTroveError> {
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }
}
