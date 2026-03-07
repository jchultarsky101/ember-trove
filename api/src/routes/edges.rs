/// Edge routes.
///
/// Phase 1 stubs — full implementation in Phase 4.
use axum::{http::StatusCode, routing::{delete, post}, Router};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_edge))
        .route("/{id}", delete(delete_edge))
}

async fn create_edge() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn delete_edge() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
