use common::{
    edge::{CreateEdgeRequest, Edge, EdgeWithTitles},
    id::{EdgeId, NodeId},
};
use gloo_net::http::Request;

use super::{api_url, parse_json};
use crate::error::UiError;

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
