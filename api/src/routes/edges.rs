use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, post},
};
use common::{
    edge::{CreateEdgeRequest, Edge},
    id::EdgeId,
};
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_edge))
        .route("/{id}", delete(delete_edge))
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
