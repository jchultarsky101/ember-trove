// Phase 1 skeleton — stub items will be used as later phases are implemented.
#![allow(dead_code)]

mod admin;
mod auth;
mod backup;
mod config;
mod error;
mod notify;
mod object_store;
mod repo;
mod routes;
mod state;
mod wikilink;

#[cfg(test)]
mod tests;

use std::{net::SocketAddr, sync::Arc};

use axum_extra::extract::cookie::Key;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use admin::CognitoAdminClient;
use auth::{AuthConfig, oidc::OidcClient};
use notify::SesNotifier;
use config::Config;
use object_store::s3::S3ObjectStore;
use repo::{
    activity::PgActivityRepo, attachment::PgAttachmentRepo, backup::PgBackupRepo,
    edge::PgEdgeRepo, favorite::PgFavoriteRepo, graph::PgGraphRepo, node::PgNodeRepo,
    node_version::PgNodeVersionRepo, note::PgNoteRepo, permission::PgPermissionRepo,
    search::PgSearchRepo, share_token::PgShareTokenRepo, tag::PgTagRepo, task::PgTaskRepo,
};
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    tracing::info!("database connection pool established");

    // Run pending migrations on startup.
    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .map_err(|e| anyhow::anyhow!("database migration failed: {e}"))?;
    tracing::info!("database migrations complete");

    // OIDC discovery — fetch endpoints and JWKS from Keycloak (optional for Phase 1 dev).
    let oidc = if let (Some(issuer), Some(client_id), Some(client_secret)) =
        (&config.oidc_issuer, &config.oidc_client_id, &config.oidc_client_secret)
    {
        let client =
            OidcClient::discover(issuer, client_id.clone(), client_secret.clone()).await?;

        Some(Arc::new(client))
    } else {
        tracing::warn!("OIDC not configured — auth endpoints will be disabled");
        None
    };

    let auth = AuthConfig {
        issuer: config.oidc_issuer.clone().unwrap_or_default(),
        client_id: config.oidc_client_id.clone().unwrap_or_default(),
        client_secret: config.oidc_client_secret.clone().unwrap_or_default(),
        frontend_url: config.frontend_url.clone(),
        api_external_url: config.api_external_url.clone(),
        cookie_secure: config.cookie_secure,
    };

    // Derive cookie encryption key from hex-encoded COOKIE_KEY.
    let key_bytes = hex::decode(&config.cookie_key)
        .map_err(|e| anyhow::anyhow!("COOKIE_KEY is not valid hex: {e}"))?;
    let cookie_key = Key::from(&key_bytes);

    let object_store: Arc<dyn object_store::ObjectStore> =
        if let (Some(bucket), Some(access_key), Some(secret_key)) = (
            config.s3_bucket.as_deref(),
            config.s3_access_key.as_deref(),
            config.s3_secret_key.as_deref(),
        ) {
            Arc::new(
                S3ObjectStore::new(
                    bucket,
                    &config.s3_region,
                    access_key,
                    secret_key,
                    config.s3_endpoint.as_deref(),
                )
                .map_err(|e| anyhow::anyhow!("S3 init failed: {e}"))?,
            )
        } else {
            tracing::warn!("S3 not configured — attachment upload/download will be unavailable");
            Arc::new(object_store::NullObjectStore)
        };

    // Cognito Admin client — optional; enabled when COGNITO_USER_POOL_ID is set.
    let cognito_admin = if let Some(pool_id) = config.cognito_user_pool_id.clone() {
        tracing::info!("Cognito admin client enabled (pool: {pool_id})");
        Some(Arc::new(
            CognitoAdminClient::new(&config.cognito_region, pool_id).await,
        ))
    } else {
        tracing::info!("COGNITO_USER_POOL_ID not set — /admin/* endpoints disabled");
        None
    };

    // SES notifier — optional; enabled when SES_FROM_EMAIL is set.
    let notifier = if let Some(from_email) = config.ses_from_email.clone() {
        tracing::info!("SES notifier enabled (from: {from_email})");
        Some(Arc::new(
            SesNotifier::new(from_email, config.frontend_url.clone()).await,
        ))
    } else {
        tracing::info!("SES_FROM_EMAIL not set — invite email notifications disabled");
        None
    };

    let state = AppState {
        started_at: std::time::Instant::now(),
        nodes: Arc::new(PgNodeRepo::new(pool.clone())),
        edges: Arc::new(PgEdgeRepo::new(pool.clone())),
        tags: Arc::new(PgTagRepo::new(pool.clone())),
        tasks: Arc::new(PgTaskRepo::new(pool.clone())),
        notes: Arc::new(PgNoteRepo::new(pool.clone())),
        attachments: Arc::new(PgAttachmentRepo::new(pool.clone())),
        permissions: Arc::new(PgPermissionRepo::new(pool.clone())),
        search: Arc::new(PgSearchRepo::new(pool.clone())),
        graph: Arc::new(PgGraphRepo::new(pool.clone())),
        backup: Arc::new(PgBackupRepo::new(pool.clone())),
        favorites: Arc::new(PgFavoriteRepo::new(pool.clone())),
        share_tokens: Arc::new(PgShareTokenRepo::new(pool.clone())),
        activity: Arc::new(PgActivityRepo::new(pool.clone())),
        node_versions: Arc::new(PgNodeVersionRepo::new(pool.clone())),
        object_store,
        oidc,
        cognito_admin,
        notifier,
        cookie_key,
        auth,
        config: config.clone(),
        pool,
    };

    let app = axum::Router::new().nest("/api", routes::build_router(state)?);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("ember-trove-api listening on {addr}");

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}
