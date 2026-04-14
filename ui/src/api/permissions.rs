use common::id::NodeId;
use gloo_net::http::Request;

use super::{api_url, parse_json};
use crate::error::UiError;

/// List every permission row in the system — no node filter (admin view).
pub async fn list_all_permissions() -> Result<Vec<common::permission::Permission>, UiError> {
    let resp = Request::get(&api_url("/permissions"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn list_permissions(
    node_id: NodeId,
) -> Result<Vec<common::permission::Permission>, UiError> {
    let resp = Request::get(&api_url(&format!("/nodes/{node_id}/permissions")))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn grant_permission(
    node_id: NodeId,
    req: &common::permission::GrantPermissionRequest,
) -> Result<common::permission::Permission, UiError> {
    let resp = Request::post(&api_url(&format!("/nodes/{node_id}/permissions")))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn revoke_permission(
    node_id: NodeId,
    perm_id: common::id::PermissionId,
) -> Result<(), UiError> {
    let resp = Request::delete(&api_url(&format!(
        "/nodes/{node_id}/permissions/{perm_id}"
    )))
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

pub async fn invite_to_node(
    node_id: NodeId,
    req: &common::permission::InviteRequest,
) -> Result<common::permission::Permission, UiError> {
    let resp = Request::post(&api_url(&format!("/nodes/{node_id}/invite")))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn update_permission(
    perm_id: common::id::PermissionId,
    req: &common::permission::UpdatePermissionRequest,
) -> Result<common::permission::Permission, UiError> {
    let resp = Request::put(&api_url(&format!("/permissions/{perm_id}")))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}
