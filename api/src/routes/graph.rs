use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, put},
};
use common::graph::{NodePosition, SavePositionRequest};
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/positions", get(list_positions))
        .route("/positions/{node_id}", put(upsert_position))
}

async fn list_positions(
    State(state): State<AppState>,
) -> Result<Json<Vec<NodePosition>>, ApiError> {
    let positions = state.graph.list_positions().await?;
    Ok(Json(positions))
}

async fn upsert_position(
    State(state): State<AppState>,
    Path(node_id): Path<Uuid>,
    Json(req): Json<SavePositionRequest>,
) -> Result<StatusCode, ApiError> {
    state.graph.upsert_position(node_id, req.x, req.y).await?;
    Ok(StatusCode::NO_CONTENT)
}
