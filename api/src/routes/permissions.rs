/// Standalone permission routes (mirrors nested routes on nodes).
///
/// Phase 1 stub — full implementation in Phase 7.
use axum::{http::StatusCode, Router};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
}

#[allow(dead_code)]
async fn not_implemented() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}
