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

use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use auth::AuthConfig;
use config::Config;
use object_store::s3::S3ObjectStore;
use repo::{
    attachment::PgAttachmentRepo, edge::PgEdgeRepo, node::PgNodeRepo,
    permission::PgPermissionRepo, tag::PgTagRepo,
};
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    tracing::info!("database connection pool established");

    // Phase 1: OIDC + S3 are stubs; use empty strings until Phase 2/6 wire them up.
    let auth = AuthConfig {
        issuer: config.oidc_issuer.clone().unwrap_or_default(),
        client_id: config.oidc_client_id.clone().unwrap_or_default(),
        client_secret: config.oidc_client_secret.clone().unwrap_or_default(),
    };

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
        object_store,
        auth,
        config: config.clone(),
        pool,
    };

    let app = routes::build_router(state);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("ember-trove-api listening on {addr}");

    axum::serve(listener, app).await?;
    Ok(())
}
