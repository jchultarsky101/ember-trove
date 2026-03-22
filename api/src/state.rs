use std::{sync::Arc, time::Instant};

use axum_extra::extract::cookie::Key;
use sqlx::PgPool;

use crate::{
    admin::CognitoAdminClient,
    auth::{AuthConfig, oidc::OidcClient},
    config::Config,
    object_store::ObjectStore,
    repo::{
        attachment::AttachmentRepo, backup::BackupRepo, edge::EdgeRepo, favorite::FavoriteRepo,
        graph::GraphRepo, node::NodeRepo, note::NoteRepo, permission::PermissionRepo,
        search::SearchRepo, tag::TagRepo, task::TaskRepo,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub nodes: Arc<dyn NodeRepo>,
    pub edges: Arc<dyn EdgeRepo>,
    pub tags: Arc<dyn TagRepo>,
    pub tasks: Arc<dyn TaskRepo>,
    pub notes: Arc<dyn NoteRepo>,
    pub attachments: Arc<dyn AttachmentRepo>,
    pub permissions: Arc<dyn PermissionRepo>,
    pub search: Arc<dyn SearchRepo>,
    pub graph: Arc<dyn GraphRepo>,
    pub backup: Arc<dyn BackupRepo>,
    pub favorites: Arc<dyn FavoriteRepo>,
    pub object_store: Arc<dyn ObjectStore>,
    pub oidc: Option<Arc<OidcClient>>,
    /// Cognito admin client — `None` when `COGNITO_USER_POOL_ID` is not set.
    pub cognito_admin: Option<Arc<CognitoAdminClient>>,
    pub cookie_key: Key,
    pub auth: AuthConfig,
    pub config: Config,
    /// Process start time — used to compute uptime in the metrics endpoint.
    pub started_at: Instant,
}

/// `PrivateCookieJar` needs `FromRef<AppState>` for `Key` to derive
/// the encryption key from shared state.
impl axum::extract::FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.cookie_key.clone()
    }
}
