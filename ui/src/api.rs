#![allow(dead_code)]
use common::{
    admin::{AdminUser, CreateAdminUserRequest, UpdateUserRolesRequest},
    attachment::Attachment,
    auth::UserInfo,
    edge::{CreateEdgeRequest, Edge, EdgeWithTitles},
    favorite::{CreateFavoriteRequest, Favorite, ReorderFavoritesRequest},
    id::{AttachmentId, EdgeId, FavoriteId, NodeId, NodeLinkId, NoteId, TagId, TaskId},
    node::{CreateNodeRequest, Node, NodeListResponse, NodeTitleEntry, SetPinnedRequest, UpdateNodeRequest},
    node_link::{CreateNodeLinkRequest, NodeLink, UpdateNodeLinkRequest},
    id::SearchPresetId,
    search::{CreateSearchPresetRequest, SearchPreset, SearchResponse},
    tag::{CreateTagRequest, Tag, UpdateTagRequest},
    note::{CreateNoteRequest, FeedNote, Note, UpdateNoteRequest},
    task::{CreateTaskRequest, MyDayTask, ProjectDashboardEntry, ReorderTaskEntry, ReorderTasksRequest, Task, UpdateTaskRequest},
};
use gloo_net::http::Request;
use serde::Deserialize;

use crate::error::UiError;

const API_BASE: &str = "/api";

#[must_use]
pub fn api_url(path: &str) -> String {
    format!("{API_BASE}{path}")
}

// ── Health ────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct HealthResponse {
    version: String,
}

/// Fetch the API version string from `/api/health`.
pub async fn fetch_api_version() -> String {
    let Ok(resp) = Request::get(&api_url("/health")).send().await else {
        return String::new();
    };
    resp.json::<HealthResponse>()
        .await
        .map(|h| h.version)
        .unwrap_or_default()
}

/// Change the current user's password.
pub async fn change_password(current: &str, proposed: &str) -> Result<(), UiError> {
    let resp = Request::post(&api_url("/auth/change-password"))
        .json(&serde_json::json!({
            "current_password": current,
            "new_password": proposed,
        }))
        .map_err(|e| UiError::Network(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;

    if resp.ok() {
        Ok(())
    } else {
        let status = resp.status();
        let msg = resp.text().await.unwrap_or_else(|_| "password change failed".to_string());
        Err(UiError::api(status, msg))
    }
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
            // Try a silent token refresh first. If it succeeds, a full-page
            // reload picks up the new access token and retries all pending
            // API calls without the user noticing.
            if refresh_session().await.is_ok() {
                if let Some(win) = web_sys::window() {
                    let _ = win.location().reload();
                }
            } else {
                // Refresh token also expired (long idle, server restart, etc.).
                // Redirect to login rather than leaving the user with a blank
                // screen or a confusing "server error 401" message.
                wasm_bindgen_futures::spawn_local(async {
                    if let Ok(url) = fetch_login_url().await
                        && let Some(win) = web_sys::window()
                    {
                        let _ = win.location().set_href(&url);
                    }
                });
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

pub async fn duplicate_node(id: NodeId) -> Result<Node, UiError> {
    let resp = Request::post(&api_url(&format!("/nodes/{id}/duplicate")))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn set_node_pinned(id: NodeId, pinned: bool) -> Result<Node, UiError> {
    let req = SetPinnedRequest { pinned };
    let resp = Request::put(&api_url(&format!("/nodes/{id}/pin")))
        .json(&req)
        .map_err(|e| UiError::Network(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn fetch_node_titles() -> Result<Vec<NodeTitleEntry>, UiError> {
    let resp = Request::get(&api_url("/nodes/titles"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
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

// ── Public share ─────────────────────────────────────────────────────────

pub async fn fetch_shared_node(token: uuid::Uuid) -> Result<common::node::Node, UiError> {
    let resp = Request::get(&api_url(&format!("/share/{token}")))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

// ── Share tokens ─────────────────────────────────────────────────────────

pub async fn list_share_tokens(
    node_id: NodeId,
) -> Result<Vec<common::share_token::ShareToken>, UiError> {
    let resp = Request::get(&api_url(&format!("/nodes/{node_id}/share")))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn create_share_token(
    node_id: NodeId,
    req: &common::share_token::CreateShareTokenRequest,
) -> Result<common::share_token::ShareToken, UiError> {
    let resp = Request::post(&api_url(&format!("/nodes/{node_id}/share")))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn revoke_share_token(
    node_id: NodeId,
    token_id: common::id::ShareTokenId,
) -> Result<(), UiError> {
    let resp = Request::delete(&api_url(&format!("/nodes/{node_id}/share/{token_id}")))
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

// ── Node versions ────────────────────────────────────────────────────────

pub async fn fetch_versions(
    node_id: NodeId,
    limit: Option<u32>,
) -> Result<Vec<common::node_version::NodeVersion>, UiError> {
    let url = match limit {
        Some(n) => api_url(&format!("/nodes/{node_id}/versions?limit={n}")),
        None => api_url(&format!("/nodes/{node_id}/versions")),
    };
    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn restore_version(
    node_id: NodeId,
    version_id: uuid::Uuid,
) -> Result<common::node::Node, UiError> {
    let resp =
        Request::post(&api_url(&format!("/nodes/{node_id}/versions/{version_id}/restore")))
            .send()
            .await
            .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

// ── Activity log ─────────────────────────────────────────────────────────

pub async fn fetch_activity(
    node_id: NodeId,
    limit: Option<u32>,
) -> Result<Vec<common::activity::ActivityEntry>, UiError> {
    let url = match limit {
        Some(n) => api_url(&format!("/nodes/{node_id}/activity?limit={n}")),
        None => api_url(&format!("/nodes/{node_id}/activity")),
    };
    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
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

/// Batch-save all node positions at once (used by auto-arrange).
pub async fn save_positions(
    positions: &[(common::id::NodeId, f64, f64)],
) -> Result<(), UiError> {
    let req = common::graph::SavePositionsRequest {
        positions: positions.to_vec(),
    };
    let resp = Request::put(&api_url("/graph/positions"))
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
/// - `status`: optional status filter (`Some("published")` etc.)
/// - `node_type`: optional node-type filter (`Some("article")` etc.)
/// - `tag_ids`: zero or more tag UUIDs to filter by
/// - `tag_op`: `"or"` (default) or `"and"` — how to combine multiple tags
/// - `sort`: optional sort order (`"relevance"`, `"updated_desc"`, `"updated_asc"`,
///   `"title_asc"`, `"title_desc"`)
/// - `updated_after` / `updated_before`: optional date bounds in `YYYY-MM-DD` format
#[allow(clippy::too_many_arguments)]
pub async fn search_nodes(
    q: &str,
    fuzzy: bool,
    status: Option<&str>,
    node_type: Option<&str>,
    tag_ids: &[uuid::Uuid],
    tag_op: &str,
    sort: Option<&str>,
    updated_after: Option<&str>,
    updated_before: Option<&str>,
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
    if let Some(nt) = node_type {
        url.push_str(&format!("&node_type={nt}"));
    }
    if !tag_ids.is_empty() {
        let ids_str = tag_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        url.push_str(&format!("&tag_ids={ids_str}&tag_op={tag_op}"));
    }
    if let Some(s) = sort {
        url.push_str(&format!("&sort={s}"));
    }
    if let Some(d) = updated_after {
        url.push_str(&format!("&updated_after={d}"));
    }
    if let Some(d) = updated_before {
        url.push_str(&format!("&updated_before={d}"));
    }
    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

// ── Admin ────────────────────────────────────────────────────────────────────

pub async fn list_admin_users() -> Result<Vec<AdminUser>, UiError> {
    let resp = Request::get(&api_url("/admin/users"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn create_admin_user(req: &CreateAdminUserRequest) -> Result<AdminUser, UiError> {
    let resp = Request::post(&api_url("/admin/users"))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn delete_admin_user(id: &str) -> Result<(), UiError> {
    let resp = Request::delete(&api_url(&format!("/admin/users/{id}")))
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

pub async fn list_realm_roles() -> Result<Vec<String>, UiError> {
    let resp = Request::get(&api_url("/admin/users/roles"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn set_user_roles(id: &str, req: &UpdateUserRolesRequest) -> Result<(), UiError> {
    let resp = Request::put(&api_url(&format!("/admin/users/{id}/roles")))
        .json(req)
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

// ── Task endpoints ─────────────────────────────────────────────────────────────

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

// ── Notes ──────────────────────────────────────────────────────────────────────

pub async fn fetch_notes(node_id: NodeId) -> Result<Vec<Note>, UiError> {
    let resp = Request::get(&api_url(&format!("/nodes/{node_id}/notes")))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn create_note(node_id: NodeId, req: &CreateNoteRequest) -> Result<Note, UiError> {
    let resp = Request::post(&api_url(&format!("/nodes/{node_id}/notes")))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn update_note(note_id: NoteId, req: &UpdateNoteRequest) -> Result<Note, UiError> {
    let resp = Request::patch(&api_url(&format!("/notes/{note_id}")))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn fetch_notes_feed() -> Result<Vec<FeedNote>, UiError> {
    let resp = Request::get(&api_url("/notes/feed"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

#[allow(dead_code)]
pub fn _use_note_id(_: NoteId) {}

// ── Backup endpoints ──────────────────────────────────────────────────────────

pub async fn list_backups() -> Result<Vec<common::backup::BackupJob>, UiError> {
    let resp = Request::get(&api_url("/admin/backups"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn create_backup_api() -> Result<common::backup::BackupJob, UiError> {
    let resp = Request::post(&api_url("/admin/backups"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
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

// ── Favorites ─────────────────────────────────────────────────────────────────

pub async fn fetch_favorites() -> Result<Vec<Favorite>, UiError> {
    let resp = Request::get(&api_url("/favorites"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn create_favorite(req: &CreateFavoriteRequest) -> Result<Favorite, UiError> {
    let resp = Request::post(&api_url("/favorites"))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn delete_favorite(id: FavoriteId) -> Result<(), UiError> {
    let resp = Request::delete(&api_url(&format!("/favorites/{id}")))
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

pub async fn reorder_favorites(req: &ReorderFavoritesRequest) -> Result<Vec<Favorite>, UiError> {
    let resp = Request::patch(&api_url("/favorites/reorder"))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

// ── Templates ─────────────────────────────────────────────────────────────────

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

// ── Search presets ─────────────────────────────────────────────────────────────

pub async fn fetch_search_presets() -> Result<Vec<SearchPreset>, UiError> {
    let resp = Request::get(&api_url("/search-presets"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn create_search_preset(req: &CreateSearchPresetRequest) -> Result<SearchPreset, UiError> {
    let resp = Request::post(&api_url("/search-presets"))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn delete_search_preset(id: SearchPresetId) -> Result<(), UiError> {
    let resp = Request::delete(&api_url(&format!("/search-presets/{id}")))
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

// ── Node Links ─────────────────────────────────────────────────────────────────

pub async fn fetch_node_links(node_id: NodeId) -> Result<Vec<NodeLink>, UiError> {
    let resp = Request::get(&api_url(&format!("/nodes/{}/links", node_id)))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn create_node_link(
    node_id: NodeId,
    req: &CreateNodeLinkRequest,
) -> Result<NodeLink, UiError> {
    let resp = Request::post(&api_url(&format!("/nodes/{}/links", node_id)))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn update_node_link(
    node_id: NodeId,
    link_id: NodeLinkId,
    req: &UpdateNodeLinkRequest,
) -> Result<NodeLink, UiError> {
    let resp = Request::put(&api_url(&format!("/nodes/{}/links/{}", node_id, link_id)))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn delete_node_link(node_id: NodeId, link_id: NodeLinkId) -> Result<(), UiError> {
    let resp = Request::delete(&api_url(&format!("/nodes/{}/links/{}", node_id, link_id)))
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
