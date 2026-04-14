use gloo_net::http::Request;

use super::{api_url, parse_json};
use crate::error::UiError;

pub async fn list_backups() -> Result<Vec<common::backup::BackupJob>, UiError> {
    let resp = Request::get(&api_url("/admin/backups"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn create_backup_api(comment: Option<String>) -> Result<common::backup::BackupJob, UiError> {
    let builder = Request::post(&api_url("/admin/backups"));
    let resp = if let Some(ref c) = comment {
        builder
            .header("Content-Type", "application/json")
            .body(serde_json::json!({ "comment": c }).to_string())
            .map_err(|e| UiError::Network(e.to_string()))?
            .send()
            .await
            .map_err(|e| UiError::Network(e.to_string()))?
    } else {
        builder
            .send()
            .await
            .map_err(|e| UiError::Network(e.to_string()))?
    };
    parse_json(resp).await
}

pub async fn delete_backup(id: uuid::Uuid) -> Result<(), UiError> {
    let resp = Request::delete(&api_url(&format!("/admin/backups/{id}")))
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

pub async fn preview_backup_restore(id: uuid::Uuid) -> Result<common::backup::BackupPreview, UiError> {
    let resp = Request::get(&api_url(&format!("/admin/backups/{id}/preview")))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn restore_backup(id: uuid::Uuid) -> Result<(), UiError> {
    let resp = Request::post(&api_url(&format!("/admin/backups/{id}/restore")))
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
