#![allow(dead_code)]
/// HTTP client helpers for communicating with the Ember Trove API.
///
/// Phase 1 stub — individual request functions will be added per phase.
use crate::error::UiError;

const API_BASE: &str = "/api";

/// Build a full API URL from a path segment.
#[must_use]
pub fn api_url(path: &str) -> String {
    format!("{API_BASE}{path}")
}

/// Parse a JSON response body, mapping errors to `UiError`.
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
