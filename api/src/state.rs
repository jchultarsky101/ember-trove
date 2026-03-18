use std::sync::Arc;

use axum_extra::extract::cookie::Key;
use sqlx::PgPool;

use crate::{
    admin::KeycloakAdminClient,
    auth::{AuthConfig, oidc::OidcClient},
    config::Config,
    object_store::ObjectStore,
    repo::{
        attachment::AttachmentRepo, edge::EdgeRepo, graph::GraphRepo, node::NodeRepo,
        permission::PermissionRepo, search::SearchRepo, tag::TagRepo, task::TaskRepo,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub nodes: Arc<dyn NodeRepo>,
    pub edges: Arc<dyn EdgeRepo>,
    pub tags: Arc<dyn TagRepo>,
    pub tasks: Arc<dyn TaskRepo>,
    pub attachments: Arc<dyn AttachmentRepo>,
    pub permissions: Arc<dyn PermissionRepo>,
    pub search: Arc<dyn SearchRepo>,
    pub graph: Arc<dyn GraphRepo>,
    pub object_store: Arc<dyn ObjectStore>,
    pub oidc: Option<Arc<OidcClient>>,
    /// Keycloak Admin client — `None` when `KEYCLOAK_ADMIN_USER` / `KEYCLOAK_ADMIN_PASSWORD`
    /// are not set in the environment.
    pub keycloak_admin: Option<Arc<KeycloakAdminClient>>,
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
