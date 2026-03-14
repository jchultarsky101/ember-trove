use axum::{Json, Router, extract::State, routing::get};
use common::search::{SearchQuery, SearchResponse};

use crate::{error::ApiError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(search))
}

async fn search(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<SearchQuery>,
) -> Result<Json<SearchResponse>, ApiError> {
    let response = state.search.search(&query).await?;
    Ok(Json(response))
}
