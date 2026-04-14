use common::{
    id::{NodeId, TagId},
    tag::{CreateTagRequest, Tag, UpdateTagRequest},
};
use gloo_net::http::Request;

use super::{api_url, parse_json};
use crate::error::UiError;

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
