use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, put},
};
use common::{
    auth::AuthClaims,
    graph::{NodePosition, SavePositionRequest, SavePositionsRequest},
    id::NodeId,
};
use uuid::Uuid;

use crate::{
    auth::permissions::require_viewer,
    error::ApiError,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/positions", get(list_positions))
        .route("/positions", put(upsert_positions_batch))
        .route("/positions/{node_id}", put(upsert_position))
}

async fn list_positions(
    State(state): State<AppState>,
    Extension(_claims): Extension<AuthClaims>,
) -> Result<Json<Vec<NodePosition>>, ApiError> {
    // Positions are keyed by node_id; the graph view client-side already
    // filters to only render nodes the caller owns. Returning all positions
    // is safe (just x/y coordinates, no sensitive data).
    let positions = state.graph.list_positions().await?;
    Ok(Json(positions))
}

async fn upsert_positions_batch(
    State(state): State<AppState>,
    Extension(_claims): Extension<AuthClaims>,
    Json(req): Json<SavePositionsRequest>,
) -> Result<StatusCode, ApiError> {
    // The graph view only sends positions for nodes the caller owns
    // (node list is already ownership-scoped). Positions are just x/y
    // coordinates with no sensitive data.
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
    Extension(claims): Extension<AuthClaims>,
    Path(node_id): Path<Uuid>,
    Json(req): Json<SavePositionRequest>,
) -> Result<StatusCode, ApiError> {
    require_viewer(state.permissions.as_ref(), &claims, NodeId(node_id)).await?;
    state.graph.upsert_position(node_id, req.x, req.y).await?;
    Ok(StatusCode::NO_CONTENT)
}
