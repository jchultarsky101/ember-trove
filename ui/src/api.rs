#![allow(dead_code)]
use common::{
    auth::UserInfo,
    edge::{CreateEdgeRequest, Edge},
    id::{EdgeId, NodeId, TagId},
    node::{CreateNodeRequest, Node, UpdateNodeRequest},
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
