use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};

use axum_extra::extract::cookie::Key;
use sqlx::PgPool;

use crate::{
    admin::CognitoAdminClient,
    auth::{AuthConfig, oidc::OidcClient},
    config::Config,
    notify::SesNotifier,
    object_store::ObjectStore,
    repo::{
        activity::ActivityRepo, attachment::AttachmentRepo, backup::BackupRepo, edge::EdgeRepo,
        favorite::FavoriteRepo, graph::GraphRepo, node::NodeRepo, node_version::NodeVersionRepo,
        note::NoteRepo, permission::PermissionRepo, search::SearchRepo,
        node_link::NodeLinkRepo, search_presets::SearchPresetRepo, share_token::ShareTokenRepo,
        tag::TagRepo, task::TaskRepo, template::TemplateRepo,
    },
};

/// In-memory PKCE store: maps OAuth `state` → `(code_verifier, created_at)`.
///
/// Replaces the previous cookie-based approach which failed on iOS Safari due
/// to ITP dropping cookies set during a `fetch()` call after a cross-domain
/// redirect (the Cognito hosted-UI round-trip).  The `state` parameter travels
/// in the redirect URL itself, so it is immune to cookie restrictions.
pub type PkceStore = Arc<Mutex<HashMap<String, (String, Instant)>>>;

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
    pub share_tokens: Arc<dyn ShareTokenRepo>,
    pub activity: Arc<dyn ActivityRepo>,
    pub node_versions: Arc<dyn NodeVersionRepo>,
    pub templates: Arc<dyn TemplateRepo>,
    pub search_presets: Arc<dyn SearchPresetRepo>,
    pub node_links: Arc<dyn NodeLinkRepo>,
    pub object_store: Arc<dyn ObjectStore>,
    pub oidc: Option<Arc<OidcClient>>,
    /// Cognito admin client — `None` when `COGNITO_USER_POOL_ID` is not set.
    pub cognito_admin: Option<Arc<CognitoAdminClient>>,
    /// SES notifier — `None` when `SES_FROM_EMAIL` is not set.
    pub notifier: Option<Arc<SesNotifier>>,
    pub cookie_key: Key,
    pub auth: AuthConfig,
    pub config: Config,
    /// Process start time — used to compute uptime in the metrics endpoint.
    pub started_at: Instant,
    /// Short-lived PKCE verifier store — keyed by OAuth state parameter.
    pub pkce_store: PkceStore,
}

/// `PrivateCookieJar` needs `FromRef<AppState>` for `Key` to derive
/// the encryption key from shared state.
impl axum::extract::FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.cookie_key.clone()
    }
}
