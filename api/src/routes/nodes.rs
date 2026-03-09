use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use common::{
    auth::AuthClaims,
    edge::Edge,
    id::{NodeId, TagId},
    node::{CreateNodeRequest, Node, NodeListParams, UpdateNodeRequest},
    tag::Tag,
};
use garde::Validate;
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_nodes).post(create_node))
        .route("/slug/{slug}", get(get_node_by_slug))
        .route("/{id}", get(get_node).put(update_node).delete(delete_node))
        .route("/{id}/neighbors", get(neighbors))
        .route("/{id}/backlinks", get(backlinks))
        .route("/{id}/edges", get(list_edges_for_node))
        .route("/{id}/tags", get(list_tags_for_node))
        .route("/{id}/tags/{tag_id}", post(attach_tag).delete(detach_tag))
        .route(
            "/{id}/attachments",
            get(list_attachments).post(upload_attachment),
        )
        .route(
            "/{id}/permissions",
            get(list_permissions).post(grant_permission),
        )
        .route("/{id}/permissions/{perm_id}", delete(revoke_permission))
}

async fn list_nodes(
    State(state): State<AppState>,
    Query(params): Query<NodeListParams>,
) -> Result<Json<Vec<Node>>, ApiError> {
    let nodes = state.nodes.list(params).await?;
    Ok(Json(nodes))
}

async fn create_node(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Json(req): Json<CreateNodeRequest>,
) -> Result<(StatusCode, Json<Node>), ApiError> {
    req.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;
    let node = state.nodes.create(&claims.sub, req).await?;
    Ok((StatusCode::CREATED, Json(node)))
}

async fn get_node(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Node>, ApiError> {
    let node = state.nodes.get(NodeId(id)).await?;
    Ok(Json(node))
}

async fn get_node_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Node>, ApiError> {
    let node = state.nodes.get_by_slug(&slug).await?;
    Ok(Json(node))
}

async fn update_node(
    State(state): State<AppState>,
    Extension(_claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateNodeRequest>,
) -> Result<Json<Node>, ApiError> {
    let node = state.nodes.update(NodeId(id), req).await?;
    Ok(Json(node))
}

async fn delete_node(
    State(state): State<AppState>,
    Extension(_claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.nodes.delete(NodeId(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn neighbors(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Node>>, ApiError> {
    let nodes = state.nodes.neighbors(NodeId(id)).await?;
    Ok(Json(nodes))
}

async fn backlinks(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Node>>, ApiError> {
    let nodes = state.nodes.backlinks(NodeId(id)).await?;
    Ok(Json(nodes))
}

// ── Phase 4: Edges & Tags ────────────────────────────────────────────────

async fn list_edges_for_node(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Edge>>, ApiError> {
    let edges = state.edges.list_for_node(NodeId(id)).await?;
    Ok(Json(edges))
}

async fn list_tags_for_node(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Tag>>, ApiError> {
    let tags = state.tags.list_for_node(NodeId(id)).await?;
    Ok(Json(tags))
}

async fn attach_tag(
    State(state): State<AppState>,
    Extension(_claims): Extension<AuthClaims>,
    Path((id, tag_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    state.tags.attach(NodeId(id), TagId(tag_id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn detach_tag(
    State(state): State<AppState>,
    Extension(_claims): Extension<AuthClaims>,
    Path((id, tag_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    state.tags.detach(NodeId(id), TagId(tag_id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Phase 6+ stubs ──────────────────────────────────────────────────────

async fn list_attachments() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}
async fn upload_attachment() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}

// ── Phase 7+ stubs ──────────────────────────────────────────────────────

async fn list_permissions() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}
async fn grant_permission() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}
async fn revoke_permission() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}
