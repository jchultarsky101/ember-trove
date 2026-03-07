use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    auth::AuthConfig,
    config::Config,
    object_store::ObjectStore,
    repo::{
        attachment::AttachmentRepo, edge::EdgeRepo, node::NodeRepo, permission::PermissionRepo,
        tag::TagRepo,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub nodes: Arc<dyn NodeRepo>,
    pub edges: Arc<dyn EdgeRepo>,
    pub tags: Arc<dyn TagRepo>,
    pub attachments: Arc<dyn AttachmentRepo>,
    pub permissions: Arc<dyn PermissionRepo>,
    pub object_store: Arc<dyn ObjectStore>,
    pub auth: AuthConfig,
    pub config: Config,
}
