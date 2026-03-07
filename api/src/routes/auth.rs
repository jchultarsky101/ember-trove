/// Auth routes: login redirect, OIDC callback, token refresh, logout, /me.
///
/// Phase 1 stubs — full implementation in Phase 2.
use axum::{http::StatusCode, routing::get, Router};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(login))
        .route("/callback", get(callback))
        .route("/me", get(me))
}

async fn login() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}

async fn callback() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}

async fn me() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}
