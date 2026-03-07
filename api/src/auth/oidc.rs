/// OIDC discovery and token exchange helpers.
///
/// Phase 1 stub — full implementation arrives in Phase 2.
use crate::auth::AuthConfig;
use crate::error::ApiError;

/// Build the Keycloak authorize URL for the login redirect.
pub fn authorize_url(config: &AuthConfig, redirect_uri: &str) -> String {
    format!(
        "{}/protocol/openid-connect/auth?client_id={}&redirect_uri={}&response_type=code&scope=openid+email+profile",
        config.issuer,
        config.client_id,
        redirect_uri,
    )
}

/// Placeholder for the OIDC callback / code exchange.
///
/// Returns `ApiError::Internal` until Phase 2 implements the full flow.
pub async fn exchange_code(_config: &AuthConfig, _code: &str) -> Result<String, ApiError> {
    Err(ApiError::Internal(
        "OIDC code exchange not yet implemented".to_string(),
    ))
}
