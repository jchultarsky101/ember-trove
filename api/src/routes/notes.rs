use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use common::{
    auth::AuthClaims,
    id::NodeId,
    note::{CreateNoteRequest, FeedNote, Note},
};
use garde::Validate;
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

/// Mounts under `/nodes/:node_id/notes`
pub fn node_note_router() -> Router<AppState> {
    Router::new().route("/", get(list_notes).post(create_note))
}

/// Mounts under `/notes`
pub fn note_feed_router() -> Router<AppState> {
    Router::new().route("/feed", get(note_feed))
}

// ── Handlers ───────────────────────────────────────────────────────────────────

/// GET /nodes/:id/notes — list all notes for a node, newest first.
async fn list_notes(
    State(state): State<AppState>,
    Extension(_claims): Extension<AuthClaims>,
    Path(node_id): Path<Uuid>,
) -> Result<Json<Vec<Note>>, ApiError> {
    let notes = state.notes.list_for_node(NodeId(node_id)).await?;
    Ok(Json(notes))
}

/// POST /nodes/:id/notes — create a note. Only the node owner may do this.
async fn create_note(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(node_id): Path<Uuid>,
    Json(req): Json<CreateNoteRequest>,
) -> Result<(StatusCode, Json<Note>), ApiError> {
    req.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // Verify the caller owns this node.
    let node = state
        .nodes
        .get(NodeId(node_id))
        .await
        .map_err(|_| ApiError::Forbidden("node not found".into()))?;
    if node.owner_id != claims.sub {
        return Err(ApiError::Forbidden("only the node owner may add notes".into()));
    }

    let note = state.notes.create(NodeId(node_id), &claims.sub, req).await?;
    Ok((StatusCode::CREATED, Json(note)))
}

/// GET /notes/feed — all notes the caller has written, newest first, with node titles.
async fn note_feed(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
) -> Result<Json<Vec<FeedNote>>, ApiError> {
    let feed = state.notes.feed_for_owner(&claims.sub).await?;
    Ok(Json(feed))
}
