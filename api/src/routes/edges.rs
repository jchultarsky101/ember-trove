use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get},
};
use common::{
    edge::{CreateEdgeRequest, Edge},
    id::EdgeId,
};
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_edges).post(create_edge))
        .route("/{id}", delete(delete_edge))
}

async fn list_edges(
    State(state): State<AppState>,
) -> Result<Json<Vec<Edge>>, ApiError> {
    let edges = state.edges.list_all().await?;
    Ok(Json(edges))
}

async fn create_edge(
    State(state): State<AppState>,
    Json(req): Json<CreateEdgeRequest>,
) -> Result<(StatusCode, Json<Edge>), ApiError> {
    let edge = state.edges.create(req).await?;
    Ok((StatusCode::CREATED, Json(edge)))
}

async fn delete_edge(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.edges.delete(EdgeId(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}
