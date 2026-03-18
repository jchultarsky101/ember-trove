use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, patch},
};
use chrono::NaiveDate;
use common::{
    auth::AuthClaims,
    id::{NodeId, TaskId},
    task::{CreateTaskRequest, Task, UpdateTaskRequest},
};
use garde::Validate;
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

/// Mounts under `/nodes/:node_id/tasks` and `/tasks`.
pub fn node_task_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_tasks).post(create_task))
}

pub fn task_router() -> Router<AppState> {
    Router::new()
        .route("/{id}", patch(update_task).delete(delete_task))
}

#[derive(Deserialize)]
struct MyDayQuery {
    date: Option<NaiveDate>,
}

pub fn my_day_router() -> Router<AppState> {
    Router::new().route("/", get(my_day))
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn list_tasks(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(node_id): Path<Uuid>,
) -> Result<Json<Vec<Task>>, ApiError> {
    let tasks = state
        .tasks
        .list_for_node(NodeId(node_id), &claims.sub)
        .await?;
    Ok(Json(tasks))
}

async fn create_task(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(node_id): Path<Uuid>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<Task>), ApiError> {
    req.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;
    let task = state
        .tasks
        .create(NodeId(node_id), &claims.sub, req)
        .await?;
    Ok((StatusCode::CREATED, Json(task)))
}

async fn update_task(
    State(state): State<AppState>,
    Extension(_claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Json<Task>, ApiError> {
    let task = state.tasks.update(TaskId(id), req).await?;
    Ok(Json(task))
}

async fn delete_task(
    State(state): State<AppState>,
    Extension(_claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.tasks.delete(TaskId(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /my-day?date=YYYY-MM-DD  (defaults to today UTC if omitted)
async fn my_day(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Query(q): Query<MyDayQuery>,
) -> Result<Json<Vec<Task>>, ApiError> {
    let date = q.date.unwrap_or_else(|| chrono::Utc::now().date_naive());
    let tasks = state.tasks.list_my_day(&claims.sub, date).await?;
    Ok(Json(tasks))
}
