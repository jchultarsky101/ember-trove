use common::id::NodeId;
use gloo_net::http::Request;

use super::{api_url, parse_json};
use crate::error::UiError;

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
