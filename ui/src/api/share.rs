use common::id::NodeId;
use gloo_net::http::Request;

use super::{api_url, parse_json};
use crate::error::UiError;

pub async fn fetch_shared_node(token: uuid::Uuid) -> Result<common::node::Node, UiError> {
    let resp = Request::get(&api_url(&format!("/share/{token}")))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

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
