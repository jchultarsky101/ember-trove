use common::id::NodeId;
use gloo_net::http::Request;

use super::{api_url, parse_json};
use crate::error::UiError;

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
