use std::sync::Arc;

use axum_extra::extract::cookie::Key;
use sqlx::PgPool;

use crate::{
    auth::{AuthConfig, oidc::OidcClient},
    config::Config,
    object_store::ObjectStore,
    repo::{
        attachment::AttachmentRepo, edge::EdgeRepo, node::NodeRepo, permission::PermissionRepo,
        search::SearchRepo, tag::TagRepo,
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
    pub search: Arc<dyn SearchRepo>,
    pub object_store: Arc<dyn ObjectStore>,
    pub oidc: Option<Arc<OidcClient>>,
    pub cookie_key: Key,
    pub auth: AuthConfig,
    pub config: Config,
}

/// PrivateCookieJar needs `FromRef<AppState>` for `Key` to derive
/// the encryption key from shared state.
impl axum::extract::FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.cookie_key.clone()
    }
}
