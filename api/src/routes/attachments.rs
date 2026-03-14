/// Attachment download route.
///
/// Phase 1 stub — full implementation in Phase 6.
use axum::{Router, http::StatusCode, routing::get};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{id}/download", get(download))
        .route("/{id}", axum::routing::delete(delete_attachment))
}

async fn download() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}
async fn delete_attachment() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}
