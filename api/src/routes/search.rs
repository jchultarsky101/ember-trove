/// Search route: GET /search?q=...&fuzzy=bool
///
/// Phase 1 stub — full implementation in Phase 5.
use axum::{http::StatusCode, routing::get, Router};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(search))
}

async fn search() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}
