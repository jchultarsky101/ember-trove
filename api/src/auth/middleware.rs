use std::sync::Arc;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::PrivateCookieJar;
use common::auth::AuthClaims;

use crate::{error::ApiError, state::AppState};

use super::oidc::OidcClient;

pub const SESSION_COOKIE: &str = "ember_trove_session";

/// Middleware that extracts and validates a JWT from the session cookie or
/// Authorization header, injecting `AuthClaims` into request extensions.
pub async fn require_auth(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = extract_token(&jar, &request)?;
    let claims = validate_and_map(&state.oidc, &token).await?;

    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}

/// Try to get the JWT from the session cookie first, then the Authorization header.
fn extract_token(jar: &PrivateCookieJar, request: &Request) -> Result<String, ApiError> {
    // 1. Session cookie
    if let Some(cookie) = jar.get(SESSION_COOKIE) {
        let value = cookie.value().to_string();
        if !value.is_empty() {
            return Ok(value);
        }
    }

    // 2. Authorization: Bearer <token>
    if let Some(auth_header) = request.headers().get("authorization") {
        let header_str = auth_header
            .to_str()
            .map_err(|_| ApiError::Unauthorized("invalid authorization header".to_string()))?;
        if let Some(token) = header_str.strip_prefix("Bearer ") {
            let token = token.trim();
            if !token.is_empty() {
                return Ok(token.to_string());
            }
        }
    }

    Err(ApiError::Unauthorized("missing authentication".to_string()))
}

/// Validate the JWT and map Keycloak claims to our AuthClaims.
async fn validate_and_map(
    oidc: &Arc<OidcClient>,
    token: &str,
) -> Result<AuthClaims, ApiError> {
    let kc_claims = oidc.validate_token(token).await?;

    let roles = kc_claims
        .realm_access
        .map(|ra| ra.roles)
        .unwrap_or_default();

    Ok(AuthClaims {
        sub: kc_claims.sub,
        email: kc_claims.email,
        name: kc_claims.name,
        roles,
        exp: kc_claims.exp,
    })
}
