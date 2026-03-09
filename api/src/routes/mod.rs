pub mod attachments;
pub mod auth;
pub mod edges;
pub mod nodes;
pub mod permissions;
pub mod search;
pub mod tags;

use axum::http::{Method, header};
use axum::{Json, Router, middleware, routing::get};
use serde_json::{Value, json};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::{auth::middleware::require_auth, state::AppState};

pub fn build_router(state: AppState) -> Router {
    // Fall back to permissive origin if parse somehow fails (shouldn't — env validated at startup).
    let origin = state
        .auth
        .frontend_url
        .parse()
        .map(AllowOrigin::exact)
        .unwrap_or_else(|_| AllowOrigin::any());

    // With allow_credentials(true), headers and methods must be explicit (not Any).
    let cors = CorsLayer::new()
        .allow_origin(origin)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
            header::COOKIE,
        ])
        .allow_credentials(true);

    // Protected routes — require a valid JWT.
    let protected = Router::new()
        .merge(auth::protected_router())
        .nest("/nodes", nodes::router())
        .nest("/edges", edges::router())
        .nest("/tags", tags::router())
        .nest("/attachments", attachments::router())
        .nest("/search", search::router())
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));

    // Public routes — no auth required.
    Router::new()
        .route("/health", get(health))
        .merge(auth::public_router())
        .merge(protected)
        .layer(cors)
        .with_state(state)
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "ember-trove-api" }))
}
