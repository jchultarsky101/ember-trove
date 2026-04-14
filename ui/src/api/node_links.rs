use common::{
    id::{NodeId, NodeLinkId},
    node_link::{CreateNodeLinkRequest, NodeLink, UpdateNodeLinkRequest},
};
use gloo_net::http::Request;

use super::{api_url, parse_json};
use crate::error::UiError;

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
