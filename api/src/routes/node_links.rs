use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use common::{
    auth::AuthClaims,
    id::{NodeId, NodeLinkId},
    node_link::{CreateNodeLinkRequest, NodeLink, UpdateNodeLinkRequest},
};
use garde::Validate;
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_links).post(create_link))
        .route("/{link_id}", axum::routing::put(update_link).delete(delete_link))
}

async fn list_links(
    State(state): State<AppState>,
    Extension(_claims): Extension<AuthClaims>,
    Path(node_id): Path<Uuid>,
) -> Result<Json<Vec<NodeLink>>, ApiError> {
    let links = state.node_links.list(NodeId(node_id)).await?;
    Ok(Json(links))
}

async fn create_link(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(node_id): Path<Uuid>,
    Json(req): Json<CreateNodeLinkRequest>,
) -> Result<(StatusCode, Json<NodeLink>), ApiError> {
    req.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    // Require at least editor-level access.
    state
        .permissions
        .find(NodeId(node_id), &claims.sub)
        .await?
        .filter(|p| {
            use common::permission::PermissionRole;
            claims.roles.contains(&"admin".to_string())
                || matches!(p.role, PermissionRole::Editor | PermissionRole::Owner)
        })
        .ok_or_else(|| ApiError::Forbidden("editor access required".to_string()))?;

    let link = state.node_links.create(NodeId(node_id), req).await?;
    Ok((StatusCode::CREATED, Json(link)))
}

async fn update_link(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path((node_id, link_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateNodeLinkRequest>,
) -> Result<Json<NodeLink>, ApiError> {
    req.validate().map_err(|e| ApiError::Validation(e.to_string()))?;
    state
        .permissions
        .find(NodeId(node_id), &claims.sub)
        .await?
        .filter(|p| {
            use common::permission::PermissionRole;
            claims.roles.contains(&"admin".to_string())
                || matches!(p.role, PermissionRole::Editor | PermissionRole::Owner)
        })
        .ok_or_else(|| ApiError::Forbidden("editor access required".to_string()))?;

    let link = state.node_links.update(NodeLinkId(link_id), req).await?;
    Ok(Json(link))
}

async fn delete_link(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path((node_id, link_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    state
        .permissions
        .find(NodeId(node_id), &claims.sub)
        .await?
        .filter(|p| {
            use common::permission::PermissionRole;
            claims.roles.contains(&"admin".to_string())
                || matches!(p.role, PermissionRole::Editor | PermissionRole::Owner)
        })
        .ok_or_else(|| ApiError::Forbidden("editor access required".to_string()))?;

    state.node_links.delete(NodeLinkId(link_id)).await?;
    Ok(StatusCode::NO_CONTENT)
}
