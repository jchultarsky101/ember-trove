use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, put},
};
use common::graph::{NodePosition, SavePositionRequest, SavePositionsRequest};
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/positions", get(list_positions))
        .route("/positions", put(upsert_positions_batch))
        .route("/positions/{node_id}", put(upsert_position))
}

async fn list_positions(
    State(state): State<AppState>,
) -> Result<Json<Vec<NodePosition>>, ApiError> {
    let positions = state.graph.list_positions().await?;
    Ok(Json(positions))
}

async fn upsert_positions_batch(
    State(state): State<AppState>,
    Json(req): Json<SavePositionsRequest>,
) -> Result<StatusCode, ApiError> {
    let tuples: Vec<(Uuid, f64, f64)> = req
        .positions
        .into_iter()
        .map(|(node_id, x, y)| (node_id.0, x, y))
        .collect();
    state.graph.save_positions(&tuples).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn upsert_position(
    State(state): State<AppState>,
    Path(node_id): Path<Uuid>,
    Json(req): Json<SavePositionRequest>,
) -> Result<StatusCode, ApiError> {
    state.graph.upsert_position(node_id, req.x, req.y).await?;
    Ok(StatusCode::NO_CONTENT)
}
