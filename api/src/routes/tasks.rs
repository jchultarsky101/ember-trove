use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, patch, post, put},
};
use chrono::{Datelike, NaiveDate};
use common::{
    auth::AuthClaims,
    id::{NodeId, TaskId},
    node::{NodeListParams, NodeType},
    task::{
        CreateTaskRequest, MyDayTask, ProjectDashboardEntry, RecurrenceRule, ReorderTasksRequest,
        Task, TaskCounts, TaskStatus, UpdateTaskRequest,
    },
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
        .route("/", post(create_standalone_task))
        .route("/inbox", get(list_inbox))
        .route("/{id}", patch(update_task).delete(delete_task))
        .route("/reorder", put(reorder_tasks))
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

// ── Recurrence helpers ────────────────────────────────────────────────────────

/// Advance `d` by one recurrence interval. Falls back to `d` on edge cases
/// (e.g. Feb 29 in non-leap year advances to Feb 28).
fn advance_date(d: NaiveDate, rule: &RecurrenceRule) -> NaiveDate {
    match rule {
        RecurrenceRule::Daily    => d + chrono::Duration::days(1),
        RecurrenceRule::Weekly   => d + chrono::Duration::weeks(1),
        RecurrenceRule::Biweekly => d + chrono::Duration::weeks(2),
        RecurrenceRule::Monthly  => {
            let (y, m) = if d.month() == 12 {
                (d.year() + 1, 1u32)
            } else {
                (d.year(), d.month() + 1)
            };
            NaiveDate::from_ymd_opt(y, m, d.day())
                .or_else(|| NaiveDate::from_ymd_opt(y, m, 28))
                .unwrap_or(d)
        }
        RecurrenceRule::Yearly => {
            NaiveDate::from_ymd_opt(d.year() + 1, d.month(), d.day())
                .or_else(|| NaiveDate::from_ymd_opt(d.year() + 1, d.month(), 28))
                .unwrap_or(d)
        }
    }
}

fn status_is_done(s: &TaskStatus) -> bool {
    matches!(s, TaskStatus::Done | TaskStatus::Cancelled)
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
        .create(Some(NodeId(node_id)), &claims.sub, req)
        .await?;
    Ok((StatusCode::CREATED, Json(task)))
}

/// POST /tasks — create a standalone (inbox) task with no parent node.
async fn create_standalone_task(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<Task>), ApiError> {
    req.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;
    // `node_id` from the body (if provided) is used; otherwise task is standalone.
    let node_id = req.node_id;
    let task = state.tasks.create(node_id, &claims.sub, req).await?;
    Ok((StatusCode::CREATED, Json(task)))
}

/// GET /tasks/inbox — standalone tasks (node_id IS NULL) for the authenticated user.
async fn list_inbox(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
) -> Result<Json<Vec<Task>>, ApiError> {
    let tasks = state.tasks.list_inbox(&claims.sub).await?;
    Ok(Json(tasks))
}

async fn update_task(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Json<Task>, ApiError> {
    // Check if this update transitions to Done — needed for recurrence.
    let becoming_done = req
        .status
        .as_ref()
        .is_some_and(|s| matches!(s, TaskStatus::Done));

    // Fetch pre-update state only when we might need to create a recurrence.
    let pre = if becoming_done {
        state.tasks.get(TaskId(id)).await.ok()
    } else {
        None
    };

    let updated = state.tasks.update(TaskId(id), req).await?;

    // If the task had recurrence and was not already done, schedule next occurrence.
    if let Some(pre_task) = pre
        && let Some(rule) = &pre_task.recurrence
        && !status_is_done(&pre_task.status)
    {
        let today = chrono::Utc::now().date_naive();
        let base_focus = pre_task.focus_date.unwrap_or(today);
        let next_focus = advance_date(base_focus, rule);
        let next_due = pre_task.due_date.map(|d| advance_date(d, rule));
        let next_req = CreateTaskRequest {
            title: pre_task.title.clone(),
            node_id: pre_task.node_id,
            status: Some(TaskStatus::Open),
            priority: Some(pre_task.priority.clone()),
            focus_date: Some(next_focus),
            due_date: next_due,
            recurrence: Some(rule.clone()),
        };
        // Best-effort — ignore errors so the Done update still succeeds.
        let next_node_id = next_req.node_id;
        let _ = state
            .tasks
            .create(next_node_id, &claims.sub, next_req)
            .await;
    }

    Ok(Json(updated))
}

async fn delete_task(
    State(state): State<AppState>,
    Extension(_claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.tasks.delete(TaskId(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// PUT /tasks/reorder — bulk update sort_order for drag-to-reorder in My Day.
async fn reorder_tasks(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Json(req): Json<ReorderTasksRequest>,
) -> Result<StatusCode, ApiError> {
    let updates: Vec<(TaskId, i32)> = req
        .tasks
        .iter()
        .map(|e| (e.id, e.sort_order))
        .collect();
    state.tasks.reorder(&updates, &claims.sub).await?;
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
