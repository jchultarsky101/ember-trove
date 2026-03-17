#![allow(dead_code)]
use common::{
    attachment::Attachment,
    auth::UserInfo,
    edge::{CreateEdgeRequest, Edge, EdgeWithTitles},
    id::{AttachmentId, EdgeId, NodeId, TagId},
    node::{CreateNodeRequest, Node, NodeListResponse, UpdateNodeRequest},
    search::SearchResponse,
    tag::{CreateTagRequest, Tag, UpdateTagRequest},
};
use gloo_net::http::Request;
use serde::Deserialize;

use crate::error::UiError;

const API_BASE: &str = "/api";

#[must_use]
pub fn api_url(path: &str) -> String {
    format!("{API_BASE}{path}")
}

pub async fn parse_json<T: serde::de::DeserializeOwned>(
    response: gloo_net::http::Response,
) -> Result<T, UiError> {
    if response.ok() {
        response
            .json::<T>()
            .await
            .map_err(|e| UiError::Parse(e.to_string()))
    } else {
        let status = response.status();
        if status == 401 {
            // Attempt a silent token refresh. On success reload the page so
            // all resources re-fetch with the new session cookie. On failure
            // the reload still happens, which causes init_auth to detect an
            // unauthenticated state and show the login screen.
            let _ = refresh_session().await;
            if let Some(win) = web_sys::window() {
                let _ = win.location().reload();
            }
        }
        let text = response
            .text()
            .await
            .unwrap_or_else(|_| "unknown error".to_string());
        Err(UiError::api(status, text))
    }
}

// ── Auth ─────────────────────────────────────────────────────────────────

pub async fn fetch_me() -> Result<UserInfo, UiError> {
    let resp = Request::get(&api_url("/auth/me"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    // Do NOT use parse_json here — parse_json retries via refresh+reload on
    // 401, which would loop infinitely because fetch_me IS the auth check.
    // A plain 401 here just means "not logged in"; AuthGate handles the redirect.
    if resp.ok() {
        resp.json::<UserInfo>()
            .await
            .map_err(|e| UiError::Parse(e.to_string()))
    } else {
        let status = resp.status();
        let text = resp
            .text()
            .await
            .unwrap_or_else(|_| "unknown error".to_string());
        Err(UiError::api(status, text))
    }
}

#[derive(Deserialize)]
pub struct RedirectResponse {
    pub redirect_url: String,
}

pub async fn fetch_login_url() -> Result<String, UiError> {
    let resp = Request::get(&api_url("/auth/login"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    let data: RedirectResponse = parse_json(resp).await?;
    Ok(data.redirect_url)
}

/// Call the backend refresh endpoint. On success the server sets a new
/// session cookie. The UI then reloads to pick it up.
pub async fn refresh_session() -> Result<(), UiError> {
    let resp = Request::post(&api_url("/auth/refresh"))
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

pub async fn fetch_logout_url() -> Result<String, UiError> {
    let resp = Request::post(&api_url("/auth/logout"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    let data: RedirectResponse = parse_json(resp).await?;
    Ok(data.redirect_url)
}

// ── Nodes ────────────────────────────────────────────────────────────────

pub async fn fetch_nodes() -> Result<Vec<Node>, UiError> {
    fetch_nodes_filtered(None, None).await
}

/// Fetch nodes with optional status and tag_id filters.
/// `status`: one of "draft", "published", "archived" or None for all.
/// `tag_id`: UUID string of a tag to filter by, or None for all.
pub async fn fetch_nodes_filtered(
    status: Option<&str>,
    tag_id: Option<uuid::Uuid>,
) -> Result<Vec<Node>, UiError> {
    let mut params: Vec<String> = Vec::new();
    if let Some(s) = status {
        params.push(format!("status={}", js_sys::encode_uri_component(s)));
    }
    if let Some(tid) = tag_id {
        params.push(format!("tag_id={tid}"));
    }
    let url = if params.is_empty() {
        api_url("/nodes")
    } else {
        format!("{}?{}", api_url("/nodes"), params.join("&"))
    };
    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    let list: NodeListResponse = parse_json(resp).await?;
    Ok(list.nodes)
}

pub async fn fetch_node(id: NodeId) -> Result<Node, UiError> {
    let resp = Request::get(&api_url(&format!("/nodes/{id}")))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn create_node(req: &CreateNodeRequest) -> Result<Node, UiError> {
    let resp = Request::post(&api_url("/nodes"))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn update_node(id: NodeId, req: &UpdateNodeRequest) -> Result<Node, UiError> {
    let resp = Request::put(&api_url(&format!("/nodes/{id}")))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn delete_node(id: NodeId) -> Result<(), UiError> {
    let resp = Request::delete(&api_url(&format!("/nodes/{id}")))
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

// ── Edges ───────────────────────────────────────────────────────────────

pub async fn fetch_all_edges() -> Result<Vec<Edge>, UiError> {
    let resp = Request::get(&api_url("/edges"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn fetch_edges_for_node(node_id: NodeId) -> Result<Vec<EdgeWithTitles>, UiError> {
    let resp = Request::get(&api_url(&format!("/nodes/{node_id}/edges")))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn fetch_backlinks(node_id: NodeId) -> Result<Vec<common::node::Node>, UiError> {
    let resp = Request::get(&api_url(&format!("/nodes/{node_id}/backlinks")))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn create_edge(req: &CreateEdgeRequest) -> Result<Edge, UiError> {
    let resp = Request::post(&api_url("/edges"))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn delete_edge(id: EdgeId) -> Result<(), UiError> {
    let resp = Request::delete(&api_url(&format!("/edges/{id}")))
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

// ── Tags ────────────────────────────────────────────────────────────────

pub async fn fetch_tags() -> Result<Vec<Tag>, UiError> {
    let resp = Request::get(&api_url("/tags"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn create_tag(req: &CreateTagRequest) -> Result<Tag, UiError> {
    let resp = Request::post(&api_url("/tags"))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn update_tag(id: TagId, req: &UpdateTagRequest) -> Result<Tag, UiError> {
    let resp = Request::put(&api_url(&format!("/tags/{id}")))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn delete_tag(id: TagId) -> Result<(), UiError> {
    let resp = Request::delete(&api_url(&format!("/tags/{id}")))
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

pub async fn fetch_tags_for_node(node_id: NodeId) -> Result<Vec<Tag>, UiError> {
    let resp = Request::get(&api_url(&format!("/nodes/{node_id}/tags")))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn attach_tag(node_id: NodeId, tag_id: TagId) -> Result<(), UiError> {
    let resp = Request::post(&api_url(&format!("/nodes/{node_id}/tags/{tag_id}")))
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

pub async fn detach_tag(node_id: NodeId, tag_id: TagId) -> Result<(), UiError> {
    let resp = Request::delete(&api_url(&format!("/nodes/{node_id}/tags/{tag_id}")))
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

// ── Attachments ─────────────────────────────────────────────────────────

pub async fn fetch_attachments(node_id: NodeId) -> Result<Vec<Attachment>, UiError> {
    let resp = Request::get(&api_url(&format!("/nodes/{node_id}/attachments")))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

/// Upload a file using multipart/form-data.
/// The `form_data` must have a `file` field containing the File object.
pub async fn upload_attachment(
    node_id: NodeId,
    form_data: web_sys::FormData,
) -> Result<Attachment, UiError> {
    use wasm_bindgen::JsValue;
    let resp = Request::post(&api_url(&format!("/nodes/{node_id}/attachments")))
        .body(JsValue::from(form_data))
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn delete_attachment(id: AttachmentId) -> Result<(), UiError> {
    let resp = Request::delete(&api_url(&format!("/attachments/{id}")))
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

#[must_use]
pub fn attachment_download_url(id: AttachmentId) -> String {
    api_url(&format!("/attachments/{id}/download"))
}

// ── Permissions ─────────────────────────────────────────────────────────

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

// ── Graph positions ──────────────────────────────────────────────────────

pub async fn fetch_positions() -> Result<Vec<common::graph::NodePosition>, UiError> {
    let resp = Request::get(&api_url("/graph/positions"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn save_position(node_id: uuid::Uuid, x: f64, y: f64) -> Result<(), UiError> {
    let req = common::graph::SavePositionRequest { x, y };
    let resp = Request::put(&api_url(&format!("/graph/positions/{node_id}")))
        .json(&req)
        .map_err(|e| UiError::Parse(e.to_string()))?
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

// ── Search ──────────────────────────────────────────────────────────────

/// Search nodes.
///
/// - `status`: optional filter (`Some("published")` etc.)
/// - `tag_ids`: zero or more tag UUIDs to filter by
/// - `tag_op`: `"or"` (default) or `"and"` — how to combine multiple tags
pub async fn search_nodes(
    q: &str,
    fuzzy: bool,
    status: Option<&str>,
    tag_ids: &[uuid::Uuid],
    tag_op: &str,
    page: u32,
    per_page: u32,
) -> Result<SearchResponse, UiError> {
    let encoded_q: String = js_sys::encode_uri_component(q).into();
    let mut url = format!(
        "{}?q={encoded_q}&fuzzy={fuzzy}&page={page}&per_page={per_page}",
        api_url("/search")
    );
    if let Some(s) = status {
        url.push_str(&format!("&status={}", js_sys::encode_uri_component(s)));
    }
    if !tag_ids.is_empty() {
        let ids_str = tag_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        url.push_str(&format!("&tag_ids={ids_str}&tag_op={tag_op}"));
    }
    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}
