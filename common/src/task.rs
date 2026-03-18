use chrono::{DateTime, NaiveDate, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::id::{NodeId, TaskId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Open,
    InProgress,
    Done,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Task {
    pub id: TaskId,
    pub node_id: NodeId,
    pub owner_id: String,
    pub title: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    /// Non-null when this task is in the user's My Day for that date.
    pub focus_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Lightweight task summary attached to a project in the dashboard.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TaskCounts {
    pub open: u32,
    pub in_progress: u32,
    pub done: u32,
    pub cancelled: u32,
}

/// A task enriched with its parent node's title, returned by the My Day endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MyDayTask {
    #[serde(flatten)]
    pub task: Task,
    pub node_title: String,
}

/// One row in the Project Dashboard — a project node plus its task counts.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProjectDashboardEntry {
    pub node_id: crate::id::NodeId,
    pub title: String,
    pub node_status: String,
    pub task_counts: TaskCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateTaskRequest {
    #[garde(length(min = 1, max = 500))]
    pub title: String,
    #[garde(skip)]
    pub status: Option<TaskStatus>,
    #[garde(skip)]
    pub priority: Option<TaskPriority>,
    #[garde(skip)]
    pub focus_date: Option<NaiveDate>,
    #[garde(skip)]
    pub due_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    /// Set to `Some(date)` to add to My Day, `Some(null)` / `None` to remove.
    pub focus_date: Option<Option<NaiveDate>>,
    pub due_date: Option<Option<NaiveDate>>,
}
