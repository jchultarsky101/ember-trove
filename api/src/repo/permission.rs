use async_trait::async_trait;
use common::{
    permission::{GrantPermissionRequest, Permission},
    id::{NodeId, PermissionId},
    EmberTroveError,
};
use sqlx::PgPool;

#[async_trait]
pub trait PermissionRepo: Send + Sync {
    async fn grant(
        &self,
        node_id: NodeId,
        granted_by: &str,
        req: GrantPermissionRequest,
    ) -> Result<Permission, EmberTroveError>;

    async fn revoke(&self, id: PermissionId) -> Result<(), EmberTroveError>;

    async fn list(&self, node_id: NodeId) -> Result<Vec<Permission>, EmberTroveError>;

    async fn find(
        &self,
        node_id: NodeId,
        subject_id: &str,
    ) -> Result<Option<Permission>, EmberTroveError>;
}

pub struct PgPermissionRepo {
    pool: PgPool,
}

impl PgPermissionRepo {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PermissionRepo for PgPermissionRepo {
    async fn grant(
        &self,
        _node_id: NodeId,
        _granted_by: &str,
        _req: GrantPermissionRequest,
    ) -> Result<Permission, EmberTroveError> {
        let _ = &self.pool;
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }

    async fn revoke(&self, _id: PermissionId) -> Result<(), EmberTroveError> {
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }

    async fn list(&self, _node_id: NodeId) -> Result<Vec<Permission>, EmberTroveError> {
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }

    async fn find(
        &self,
        _node_id: NodeId,
        _subject_id: &str,
    ) -> Result<Option<Permission>, EmberTroveError> {
        Err(EmberTroveError::Internal("not yet implemented".to_string()))
    }
}
