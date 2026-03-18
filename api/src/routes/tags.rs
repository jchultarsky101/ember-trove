use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, put},
};
use common::{
    auth::AuthClaims,
    id::TagId,
    tag::{CreateTagRequest, Tag, UpdateTagRequest},
};
use garde::Validate;
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_tags).post(create_tag))
        .route("/{id}", put(update_tag).delete(delete_tag))
}

async fn list_tags(
    State(state): State<AppState>,
    Extension(_claims): Extension<AuthClaims>,
) -> Result<Json<Vec<Tag>>, ApiError> {
    // Single-user mode: return all tags regardless of who created them.
    let tags = state.tags.list_all().await?;
    Ok(Json(tags))
}

async fn create_tag(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Json(req): Json<CreateTagRequest>,
) -> Result<(StatusCode, Json<Tag>), ApiError> {
    req.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;
    let tag = state.tags.create(&claims.sub, req).await?;
    Ok((StatusCode::CREATED, Json(tag)))
}

async fn update_tag(
    State(state): State<AppState>,
    Extension(_claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTagRequest>,
) -> Result<Json<Tag>, ApiError> {
    let tag = state.tags.update(TagId(id), req).await?;
    Ok(Json(tag))
}

async fn delete_tag(
    State(state): State<AppState>,
    Extension(_claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.tags.delete(TagId(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}
