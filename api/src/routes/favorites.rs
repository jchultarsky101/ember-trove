use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, patch},
};
use common::{
    auth::AuthClaims,
    favorite::{CreateFavoriteRequest, Favorite, ReorderFavoritesRequest},
    id::FavoriteId,
};
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_favorites).post(create_favorite))
        .route("/{id}", delete(delete_favorite))
        .route("/reorder", patch(reorder_favorites))
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /favorites — list the caller's favorites ordered by position.
async fn list_favorites(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
) -> Result<Json<Vec<Favorite>>, ApiError> {
    let favs = state.favorites.list(&claims.sub).await?;
    Ok(Json(favs))
}

/// POST /favorites — add a new favorite (node or URL).
async fn create_favorite(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Json(req): Json<CreateFavoriteRequest>,
) -> Result<(StatusCode, Json<Favorite>), ApiError> {
    let fav = state.favorites.create(&claims.sub, req).await?;
    Ok((StatusCode::CREATED, Json(fav)))
}

/// DELETE /favorites/:id — remove a favorite.
async fn delete_favorite(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.favorites.delete(FavoriteId(id), &claims.sub).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// PATCH /favorites/reorder — reorder by sending the full ordered ID list.
async fn reorder_favorites(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Json(req): Json<ReorderFavoritesRequest>,
) -> Result<Json<Vec<Favorite>>, ApiError> {
    let ids: Vec<FavoriteId> = req.ids.into_iter().map(FavoriteId).collect();
    let favs = state.favorites.reorder(&claims.sub, &ids).await?;
    Ok(Json(favs))
}
