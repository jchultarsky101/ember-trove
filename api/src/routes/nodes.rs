/// Node CRUD routes.
///
/// Phase 1 stubs — full implementation in Phase 3.
use axum::{http::StatusCode, routing::{delete, get, post}, Router};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_nodes).post(create_node))
        .route("/slug/{slug}", get(get_node_by_slug))
        .route("/{id}", get(get_node).put(update_node).delete(delete_node))
        .route("/{id}/neighbors", get(neighbors))
        .route("/{id}/backlinks", get(backlinks))
        .route("/{id}/edges", get(list_edges_for_node))
        .route("/{id}/tags/{tag_id}", post(attach_tag).delete(detach_tag))
        .route("/{id}/attachments", get(list_attachments).post(upload_attachment))
        .route("/{id}/permissions", get(list_permissions).post(grant_permission))
        .route("/{id}/permissions/{perm_id}", delete(revoke_permission))
}

async fn list_nodes() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn create_node() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn get_node() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn get_node_by_slug() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn update_node() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn delete_node() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn neighbors() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn backlinks() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn list_edges_for_node() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn attach_tag() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn detach_tag() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn list_attachments() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn upload_attachment() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn list_permissions() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn grant_permission() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
async fn revoke_permission() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
