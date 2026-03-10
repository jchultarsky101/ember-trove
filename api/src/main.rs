// Phase 1 skeleton — stub items will be used as later phases are implemented.
#![allow(dead_code)]

mod auth;
mod config;
mod error;
mod object_store;
mod repo;
mod routes;
mod state;

use std::sync::Arc;

use axum_extra::extract::cookie::Key;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use auth::{AuthConfig, oidc::OidcClient};
use config::Config;
use object_store::s3::S3ObjectStore;
use repo::{
    attachment::PgAttachmentRepo, edge::PgEdgeRepo, node::PgNodeRepo, permission::PgPermissionRepo,
    search::PgSearchRepo, tag::PgTagRepo,
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

    // OIDC discovery — fetch endpoints and JWKS from Keycloak.
    let oidc = Arc::new(
        OidcClient::discover(
            &config.oidc_issuer,
            config.oidc_client_id.clone(),
            config.oidc_client_secret.clone(),
        )
        .await?,
    );

    let auth = AuthConfig {
        issuer: config.oidc_issuer.clone(),
        client_id: config.oidc_client_id.clone(),
        client_secret: config.oidc_client_secret.clone(),
        frontend_url: config.frontend_url.clone(),
        api_external_url: config.api_external_url.clone(),
    };

    // Derive cookie encryption key from hex-encoded COOKIE_KEY.
    let key_bytes = hex::decode(&config.cookie_key)
        .map_err(|e| anyhow::anyhow!("COOKIE_KEY is not valid hex: {e}"))?;
    let cookie_key = Key::from(&key_bytes);

    let object_store = Arc::new(S3ObjectStore::new(
        config.s3_bucket.clone().unwrap_or_default(),
        config.s3_endpoint.clone().unwrap_or_default(),
    ));

    let state = AppState {
        nodes: Arc::new(PgNodeRepo::new(pool.clone())),
        edges: Arc::new(PgEdgeRepo::new(pool.clone())),
        tags: Arc::new(PgTagRepo::new(pool.clone())),
        attachments: Arc::new(PgAttachmentRepo::new(pool.clone())),
        permissions: Arc::new(PgPermissionRepo::new(pool.clone())),
        search: Arc::new(PgSearchRepo::new(pool.clone())),
        object_store,
        oidc,
        cookie_key,
        auth,
        config: config.clone(),
        pool,
    };

    let app = axum::Router::new().nest("/api", routes::build_router(state));

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("ember-trove-api listening on {addr}");

    axum::serve(listener, app).await?;
    Ok(())
}
