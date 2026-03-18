use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use common::{
    EmberTroveError,
    id::{NodeId, TaskId},
    task::{
        CreateTaskRequest, Task, TaskCounts, TaskPriority, TaskStatus, UpdateTaskRequest,
    },
};
use sqlx::PgPool;
use uuid::Uuid;

// ── Trait ─────────────────────────────────────────────────────────────────────

#[async_trait]
pub trait TaskRepo: Send + Sync {
    async fn create(
        &self,
        node_id: NodeId,
        owner_id: &str,
        req: CreateTaskRequest,
    ) -> Result<Task, EmberTroveError>;

    async fn list_for_node(
        &self,
        node_id: NodeId,
        owner_id: &str,
    ) -> Result<Vec<Task>, EmberTroveError>;

    async fn get(&self, id: TaskId) -> Result<Task, EmberTroveError>;

    async fn update(&self, id: TaskId, req: UpdateTaskRequest) -> Result<Task, EmberTroveError>;

    async fn delete(&self, id: TaskId) -> Result<(), EmberTroveError>;

    /// Tasks the caller has marked for focus on `date` (My Day).
    async fn list_my_day(
        &self,
        owner_id: &str,
        date: NaiveDate,
    ) -> Result<Vec<Task>, EmberTroveError>;

    /// Aggregated task counts per project node for the dashboard.
    async fn counts_for_nodes(
        &self,
        node_ids: &[NodeId],
    ) -> Result<Vec<(NodeId, TaskCounts)>, EmberTroveError>;
}

// ── PgTaskRepo ────────────────────────────────────────────────────────────────

pub struct PgTaskRepo {
    pool: PgPool,
}

impl PgTaskRepo {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct TaskRow {
    id: Uuid,
    node_id: Uuid,
    owner_id: String,
    title: String,
    status: String,
    priority: String,
    focus_date: Option<NaiveDate>,
    due_date: Option<NaiveDate>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

fn parse_status(s: &str) -> Result<TaskStatus, EmberTroveError> {
    match s {
        "open" => Ok(TaskStatus::Open),
        "in_progress" => Ok(TaskStatus::InProgress),
        "done" => Ok(TaskStatus::Done),
        "cancelled" => Ok(TaskStatus::Cancelled),
        other => Err(EmberTroveError::Internal(format!(
            "unknown task_status: {other}"
        ))),
    }
}

fn parse_priority(s: &str) -> Result<TaskPriority, EmberTroveError> {
    match s {
        "low" => Ok(TaskPriority::Low),
        "medium" => Ok(TaskPriority::Medium),
        "high" => Ok(TaskPriority::High),
        other => Err(EmberTroveError::Internal(format!(
            "unknown task_priority: {other}"
        ))),
    }
}

fn status_str(s: &TaskStatus) -> &'static str {
    match s {
        TaskStatus::Open => "open",
        TaskStatus::InProgress => "in_progress",
        TaskStatus::Done => "done",
        TaskStatus::Cancelled => "cancelled",
    }
}

fn priority_str(p: &TaskPriority) -> &'static str {
    match p {
        TaskPriority::Low => "low",
        TaskPriority::Medium => "medium",
        TaskPriority::High => "high",
    }
}

impl TaskRow {
    fn into_task(self) -> Result<Task, EmberTroveError> {
        Ok(Task {
            id: TaskId(self.id),
            node_id: NodeId(self.node_id),
            owner_id: self.owner_id,
            title: self.title,
            status: parse_status(&self.status)?,
            priority: parse_priority(&self.priority)?,
            focus_date: self.focus_date,
            due_date: self.due_date,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

const SELECT_COLS: &str = r#"
    id, node_id, owner_id, title,
    status::text    AS status,
    priority::text  AS priority,
    focus_date, due_date, created_at, updated_at
"#;

#[async_trait]
impl TaskRepo for PgTaskRepo {
    async fn create(
        &self,
        node_id: NodeId,
        owner_id: &str,
        req: CreateTaskRequest,
    ) -> Result<Task, EmberTroveError> {
        let status = req
            .status
            .as_ref()
            .map_or("open", |s| status_str(s));
        let priority = req
            .priority
            .as_ref()
            .map_or("medium", |p| priority_str(p));

        let row = sqlx::query_as::<_, TaskRow>(&format!(
            r#"
            INSERT INTO node_tasks (node_id, owner_id, title, status, priority, focus_date, due_date)
            VALUES ($1, $2, $3, $4::task_status, $5::task_priority, $6, $7)
            RETURNING {SELECT_COLS}
            "#
        ))
        .bind(node_id.0)
        .bind(owner_id)
        .bind(&req.title)
        .bind(status)
        .bind(priority)
        .bind(req.focus_date)
        .bind(req.due_date)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("create task failed: {e}")))?;

        row.into_task()
    }

    async fn list_for_node(
        &self,
        node_id: NodeId,
        _owner_id: &str,
    ) -> Result<Vec<Task>, EmberTroveError> {
        let rows = sqlx::query_as::<_, TaskRow>(&format!(
            r#"
            SELECT {SELECT_COLS}
            FROM node_tasks
            WHERE node_id = $1
            ORDER BY
                CASE status::text
                    WHEN 'open'        THEN 0
                    WHEN 'in_progress' THEN 1
                    WHEN 'done'        THEN 2
                    WHEN 'cancelled'   THEN 3
                END,
                CASE priority::text
                    WHEN 'high'   THEN 0
                    WHEN 'medium' THEN 1
                    WHEN 'low'    THEN 2
                END,
                created_at
            "#
        ))
        .bind(node_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("list tasks failed: {e}")))?;

        rows.into_iter().map(TaskRow::into_task).collect()
    }

    async fn get(&self, id: TaskId) -> Result<Task, EmberTroveError> {
        let row = sqlx::query_as::<_, TaskRow>(&format!(
            r#"SELECT {SELECT_COLS} FROM node_tasks WHERE id = $1"#
        ))
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("get task failed: {e}")))?
        .ok_or_else(|| EmberTroveError::NotFound(format!("task {id} not found")))?;

        row.into_task()
    }

    async fn update(&self, id: TaskId, req: UpdateTaskRequest) -> Result<Task, EmberTroveError> {
        let status_s = req.status.as_ref().map(|s| status_str(s).to_string());
        let priority_s = req.priority.as_ref().map(|p| priority_str(p).to_string());

        let row = sqlx::query_as::<_, TaskRow>(&format!(
            r#"
            UPDATE node_tasks SET
                title      = COALESCE($2, title),
                status     = COALESCE($3::task_status,   status),
                priority   = COALESCE($4::task_priority, priority),
                focus_date = CASE WHEN $5 THEN $6 ELSE focus_date END,
                due_date   = CASE WHEN $7 THEN $8 ELSE due_date END
            WHERE id = $1
            RETURNING {SELECT_COLS}
            "#
        ))
        .bind(id.0)
        .bind(&req.title)
        .bind(status_s)
        .bind(priority_s)
        .bind(req.focus_date.is_some())
        .bind(req.focus_date.and_then(|d| d))
        .bind(req.due_date.is_some())
        .bind(req.due_date.and_then(|d| d))
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("update task failed: {e}")))?
        .ok_or_else(|| EmberTroveError::NotFound(format!("task {id} not found")))?;

        row.into_task()
    }

    async fn delete(&self, id: TaskId) -> Result<(), EmberTroveError> {
        let result = sqlx::query("DELETE FROM node_tasks WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| EmberTroveError::Internal(format!("delete task failed: {e}")))?;

        if result.rows_affected() == 0 {
            return Err(EmberTroveError::NotFound(format!("task {id} not found")));
        }
        Ok(())
    }

    async fn list_my_day(
        &self,
        owner_id: &str,
        date: NaiveDate,
    ) -> Result<Vec<Task>, EmberTroveError> {
        let rows = sqlx::query_as::<_, TaskRow>(&format!(
            r#"
            SELECT {SELECT_COLS}
            FROM node_tasks
            WHERE owner_id = $1 AND focus_date = $2
            ORDER BY
                CASE priority::text
                    WHEN 'high'   THEN 0
                    WHEN 'medium' THEN 1
                    WHEN 'low'    THEN 2
                END,
                node_id, created_at
            "#
        ))
        .bind(owner_id)
        .bind(date)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("list my day failed: {e}")))?;

        rows.into_iter().map(TaskRow::into_task).collect()
    }

    async fn counts_for_nodes(
        &self,
        node_ids: &[NodeId],
    ) -> Result<Vec<(NodeId, TaskCounts)>, EmberTroveError> {
        if node_ids.is_empty() {
            return Ok(vec![]);
        }

        let ids: Vec<Uuid> = node_ids.iter().map(|n| n.0).collect();

        #[derive(sqlx::FromRow)]
        struct CountRow {
            node_id: Uuid,
            open: i64,
            in_progress: i64,
            done: i64,
            cancelled: i64,
        }

        let rows = sqlx::query_as::<_, CountRow>(
            r#"
            SELECT
                node_id,
                COUNT(*) FILTER (WHERE status = 'open')        AS open,
                COUNT(*) FILTER (WHERE status = 'in_progress') AS in_progress,
                COUNT(*) FILTER (WHERE status = 'done')        AS done,
                COUNT(*) FILTER (WHERE status = 'cancelled')   AS cancelled
            FROM node_tasks
            WHERE node_id = ANY($1)
            GROUP BY node_id
            "#,
        )
        .bind(&ids)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("task counts failed: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|r| {
                (
                    NodeId(r.node_id),
                    TaskCounts {
                        open: r.open as u32,
                        in_progress: r.in_progress as u32,
                        done: r.done as u32,
                        cancelled: r.cancelled as u32,
                    },
                )
            })
            .collect())
    }
}
