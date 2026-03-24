/// Standalone permission routes.
///
/// These mirror the nested `nodes/{id}/permissions` routes but operate directly
/// on permission IDs, making it easy to list or update grants across nodes.
use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, put},
    Extension, Json,
};
use common::{
    auth::AuthClaims,
    id::PermissionId,
    permission::{Permission, PermissionListParams, UpdatePermissionRequest},
};
use garde::Validate;
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_permissions))
        .route("/{id}", put(update_permission).delete(delete_permission))
}

/// `GET /permissions[?node_id=<uuid>]`
///
/// Returns all permissions visible to the caller, optionally filtered to a
/// single node.
async fn list_permissions(
    State(state): State<AppState>,
    Query(params): Query<PermissionListParams>,
) -> Result<Json<Vec<Permission>>, ApiError> {
    let node_id = params
        .node_id
        .map(common::id::NodeId);
    let perms = state.permissions.list_all(node_id).await?;
    Ok(Json(perms))
}

/// `PUT /permissions/{id}`
///
/// Updates the role of an existing permission grant.
/// Only the caller who originally granted the permission (or an owner) should
/// be able to do this; for now the system is single-user so we just record
/// the updater in `granted_by`.
async fn update_permission(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePermissionRequest>,
) -> Result<Json<Permission>, ApiError> {
    req.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let perm = state
        .permissions
        .update(PermissionId(id), req.role, &claims.sub)
        .await?;
    Ok(Json(perm))
}

/// `DELETE /permissions/{id}`
///
/// Convenience alias to the nested revoke route — operates on the permission
/// ID directly without needing to know the parent node.
async fn delete_permission(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.permissions.revoke(PermissionId(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}
