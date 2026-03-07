pub mod attachments;
pub mod auth;
pub mod edges;
pub mod nodes;
pub mod permissions;
pub mod search;
pub mod tags;

use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .nest("/auth", auth::router())
        .nest("/nodes", nodes::router())
        .nest("/edges", edges::router())
        .nest("/tags", tags::router())
        .nest("/attachments", attachments::router())
        .nest("/search", search::router())
        .with_state(state)
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "ember-trove-api" }))
}
