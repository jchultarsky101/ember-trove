use gloo_net::http::Request;

use super::{api_url, parse_json};
use crate::error::UiError;

pub async fn list_templates() -> Result<Vec<common::template::NodeTemplate>, UiError> {
    let resp = Request::get(&api_url("/templates"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn create_template(
    req: &common::template::CreateTemplateRequest,
) -> Result<common::template::NodeTemplate, UiError> {
    let resp = Request::post(&api_url("/templates"))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn update_template(
    id: uuid::Uuid,
    req: &common::template::UpdateTemplateRequest,
) -> Result<common::template::NodeTemplate, UiError> {
    let resp = Request::put(&api_url(&format!("/templates/{id}")))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn delete_template(id: uuid::Uuid) -> Result<(), UiError> {
    let resp = Request::delete(&api_url(&format!("/templates/{id}")))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    if resp.ok() {
        Ok(())
    } else {
        let status = resp.status();
        let text = resp
            .text()
            .await
            .unwrap_or_else(|_| "unknown error".to_string());
        Err(UiError::api(status, text))
    }
}

/// Toggle the `is_default` flag for the given template.
///
/// Returns the updated `NodeTemplate` (with `is_default` reflecting the new
/// state).  Only the template's creator may call this successfully.
pub async fn set_template_default(id: uuid::Uuid) -> Result<common::template::NodeTemplate, UiError> {
    let resp = Request::put(&api_url(&format!("/templates/{id}/set-default")))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}
