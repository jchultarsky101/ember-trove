/// Tag routes.
///
/// Phase 1 stubs — full implementation in Phase 4.
use axum::{http::StatusCode, routing::{get, put}, Router};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_tags).post(create_tag))
        .route("/{id}", put(update_tag).delete(delete_tag))
}

async fn list_tags() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn create_tag() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn update_tag() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn delete_tag() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
