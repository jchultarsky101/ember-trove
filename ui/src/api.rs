#![allow(dead_code)]
use common::{
    attachment::Attachment,
    auth::UserInfo,
    edge::{CreateEdgeRequest, Edge},
    id::{AttachmentId, EdgeId, NodeId, TagId},
    node::{CreateNodeRequest, Node, UpdateNodeRequest},
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
    parse_json(resp).await
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
    let resp = Request::get(&api_url("/nodes"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
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

pub async fn fetch_edges_for_node(node_id: NodeId) -> Result<Vec<Edge>, UiError> {
    let resp = Request::get(&api_url(&format!("/nodes/{node_id}/edges")))
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

// ── Search ──────────────────────────────────────────────────────────────

pub async fn search_nodes(
    q: &str,
    fuzzy: bool,
    page: u32,
    per_page: u32,
) -> Result<SearchResponse, UiError> {
    let encoded_q: String = js_sys::encode_uri_component(q).into();
    let url = api_url(&format!(
        "/search?q={encoded_q}&fuzzy={fuzzy}&page={page}&per_page={per_page}"
    ));
    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}
