use gloo_net::http::Request;

use super::{api_url, parse_json};
use crate::error::UiError;

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
