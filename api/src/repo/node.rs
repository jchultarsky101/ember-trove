use async_trait::async_trait;
use common::{
    node::{CreateNodeRequest, Node, NodeListParams, UpdateNodeRequest},
    id::NodeId,
    EmberTroveError,
};
use sqlx::PgPool;

#[async_trait]
pub trait NodeRepo: Send + Sync {
    async fn create(
        &self,
        owner_id: &str,
        req: CreateNodeRequest,
    ) -> Result<Node, EmberTroveError>;

    async fn get(&self, id: NodeId) -> Result<Node, EmberTroveError>;

    async fn get_by_slug(&self, slug: &str) -> Result<Node, EmberTroveError>;

    async fn list(&self, params: NodeListParams) -> Result<Vec<Node>, EmberTroveError>;

    async fn update(
        &self,
        id: NodeId,
        req: UpdateNodeRequest,
    ) -> Result<Node, EmberTroveError>;

    async fn delete(&self, id: NodeId) -> Result<(), EmberTroveError>;

    async fn neighbors(&self, id: NodeId) -> Result<Vec<Node>, EmberTroveError>;

    async fn backlinks(&self, id: NodeId) -> Result<Vec<Node>, EmberTroveError>;
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

#[async_trait]
impl NodeRepo for PgNodeRepo {
    async fn create(
        &self,
        _owner_id: &str,
        _req: CreateNodeRequest,
    ) -> Result<Node, EmberTroveError> {
        let _ = &self.pool;
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }

    async fn get(&self, _id: NodeId) -> Result<Node, EmberTroveError> {
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }

    async fn get_by_slug(&self, _slug: &str) -> Result<Node, EmberTroveError> {
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }

    async fn list(&self, _params: NodeListParams) -> Result<Vec<Node>, EmberTroveError> {
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }

    async fn update(
        &self,
        _id: NodeId,
        _req: UpdateNodeRequest,
    ) -> Result<Node, EmberTroveError> {
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }

    async fn delete(&self, _id: NodeId) -> Result<(), EmberTroveError> {
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }

    async fn neighbors(&self, _id: NodeId) -> Result<Vec<Node>, EmberTroveError> {
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }

    async fn backlinks(&self, _id: NodeId) -> Result<Vec<Node>, EmberTroveError> {
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }
}
