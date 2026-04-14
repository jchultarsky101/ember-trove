use common::{
    id::{NodeId, TaskId},
    task::{CreateTaskRequest, MyDayTask, ProjectDashboardEntry, ReorderTaskEntry, ReorderTasksRequest, Task, UpdateTaskRequest},
};
use gloo_net::http::Request;

use super::{api_url, parse_json};
use crate::error::UiError;

pub async fn fetch_tasks(node_id: NodeId) -> Result<Vec<Task>, UiError> {
    let resp = Request::get(&api_url(&format!("/nodes/{node_id}/tasks")))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn list_inbox() -> Result<Vec<Task>, UiError> {
    let resp = Request::get(&api_url("/tasks/inbox"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn create_standalone_task(req: &CreateTaskRequest) -> Result<Task, UiError> {
    let resp = Request::post(&api_url("/tasks"))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn create_task(node_id: NodeId, req: &CreateTaskRequest) -> Result<Task, UiError> {
    let resp = Request::post(&api_url(&format!("/nodes/{node_id}/tasks")))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn update_task(task_id: TaskId, req: &UpdateTaskRequest) -> Result<Task, UiError> {
    let resp = Request::patch(&api_url(&format!("/tasks/{task_id}")))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn delete_task(task_id: TaskId) -> Result<(), UiError> {
    let resp = Request::delete(&api_url(&format!("/tasks/{task_id}")))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    if resp.ok() {
        Ok(())
    } else {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Err(UiError::api(status, text))
    }
}

pub async fn reorder_tasks(entries: &[(TaskId, i32)]) -> Result<(), UiError> {
    let req = ReorderTasksRequest {
        tasks: entries
            .iter()
            .map(|(id, order)| ReorderTaskEntry { id: *id, sort_order: *order })
            .collect(),
    };
    let resp = Request::put(&api_url("/tasks/reorder"))
        .json(&req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    if resp.ok() {
        Ok(())
    } else {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Err(UiError::api(status, text))
    }
}

pub async fn fetch_project_dashboard() -> Result<Vec<ProjectDashboardEntry>, UiError> {
    let resp = Request::get(&api_url("/dashboard/projects"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn fetch_my_day() -> Result<Vec<MyDayTask>, UiError> {
    let resp = Request::get(&api_url("/my-day"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn fetch_calendar_tasks(year: i32, month: u32) -> Result<Vec<MyDayTask>, UiError> {
    let url = api_url(&format!("/calendar?year={year}&month={month}"));
    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}
