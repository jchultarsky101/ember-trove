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
    node::{NodeListParams, NodeType},
    task::{CreateTaskRequest, MyDayTask, ProjectDashboardEntry, Task, TaskCounts, UpdateTaskRequest},
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

pub fn calendar_router() -> Router<AppState> {
    Router::new().route("/", get(calendar_handler))
}

pub fn dashboard_router() -> Router<AppState> {
    Router::new().route("/", get(project_dashboard))
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
) -> Result<Json<Vec<MyDayTask>>, ApiError> {
    let date = q.date.unwrap_or_else(|| chrono::Utc::now().date_naive());
    let tasks = state.tasks.list_my_day(&claims.sub, date).await?;
    Ok(Json(tasks))
}

#[derive(Deserialize)]
struct CalendarParams {
    year: i32,
    month: u32,
}

/// GET /calendar?year=YYYY&month=M — tasks with due_date in the given month.
async fn calendar_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Query(params): Query<CalendarParams>,
) -> Result<Json<Vec<MyDayTask>>, ApiError> {
    use chrono::NaiveDate;
    let from = NaiveDate::from_ymd_opt(params.year, params.month, 1)
        .ok_or_else(|| ApiError::Validation("invalid year/month".into()))?;
    let to = if params.month == 12 {
        NaiveDate::from_ymd_opt(params.year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(params.year, params.month + 1, 1)
    }
    .ok_or_else(|| ApiError::Validation("invalid year/month".into()))?
    .pred_opt()
    .ok_or_else(|| ApiError::Validation("date underflow".into()))?;

    let tasks = state.tasks.list_by_due_range(&claims.sub, from, to).await?;
    Ok(Json(tasks))
}

/// GET /dashboard/projects — all project nodes with aggregated task counts.
async fn project_dashboard(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
) -> Result<Json<Vec<ProjectDashboardEntry>>, ApiError> {
    // Fetch all project nodes for this owner.
    let params = NodeListParams {
        node_type: Some(NodeType::Project),
        status: None,
        tag_id: None,
        owner_id: Some(claims.sub.clone()),
        subject_id: None,
        page: Some(1),
        per_page: Some(500),
    };
    let (projects, _) = state.nodes.list(params).await?;

    if projects.is_empty() {
        return Ok(Json(vec![]));
    }

    let node_ids: Vec<NodeId> = projects.iter().map(|n| n.id).collect();
    let counts_map: std::collections::HashMap<NodeId, TaskCounts> = state
        .tasks
        .counts_for_nodes(&node_ids)
        .await?
        .into_iter()
        .collect();

    let empty = TaskCounts { open: 0, in_progress: 0, done: 0, cancelled: 0 };
    let entries = projects
        .into_iter()
        .map(|n| {
            let node_status = format!("{:?}", n.status).to_lowercase();
            let task_counts = counts_map.get(&n.id).cloned().unwrap_or_else(|| empty.clone());
            ProjectDashboardEntry {
                node_id: n.id,
                title: n.title,
                node_status,
                task_counts,
            }
        })
        .collect();

    Ok(Json(entries))
}
