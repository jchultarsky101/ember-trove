use async_trait::async_trait;
use common::{
    edge::{CreateEdgeRequest, Edge},
    id::{EdgeId, NodeId},
    EmberTroveError,
};
use sqlx::PgPool;

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

#[async_trait]
impl EdgeRepo for PgEdgeRepo {
    async fn create(&self, _req: CreateEdgeRequest) -> Result<Edge, EmberTroveError> {
        let _ = &self.pool;
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }

    async fn delete(&self, _id: EdgeId) -> Result<(), EmberTroveError> {
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }

    async fn list_for_node(&self, _node_id: NodeId) -> Result<Vec<Edge>, EmberTroveError> {
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }
}
