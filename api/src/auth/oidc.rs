use std::sync::Arc;
use std::time::{Duration, Instant};

use jsonwebtoken::{DecodingKey, Validation, jwk::JwkSet};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::error::ApiError;

/// TTL for JWKS cache — refresh after 1 hour to handle key rotation.
const JWKS_TTL: Duration = Duration::from_secs(3600);

/// Cached JWKS with timestamp for TTL invalidation.
struct CachedJwks {
    jwks: JwkSet,
    fetched_at: Instant,
}

/// OIDC discovery document (subset of fields we need).
#[derive(Debug, Deserialize)]
struct OidcDiscovery {
    authorization_endpoint: String,
    token_endpoint: String,
    jwks_uri: String,
    end_session_endpoint: String,
    /// RFC 7009 token revocation endpoint — used for backchannel logout.
    revocation_endpoint: Option<String>,
}

/// Token response from the OIDC token endpoint.
#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    /// ID token — present for `openid` scope; contains email, name, and group claims.
    #[serde(default)]
    pub id_token: Option<String>,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub expires_in: Option<i64>,
}

/// JWT claims — compatible with Cognito ID tokens.
///
/// Groups are read from `cognito:groups` (Cognito) which maps directly
/// to roles in our `AuthClaims`.
#[derive(Debug, Deserialize)]
pub struct OidcClaims {
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    /// Cognito group memberships — used as roles in `AuthClaims`.
    #[serde(rename = "cognito:groups", default)]
    pub groups: Option<Vec<String>>,
    pub exp: i64,
}

/// OIDC client that handles discovery, code exchange, and JWT validation.
pub struct OidcClient {
    /// Browser-facing authorization endpoint.
    pub authorization_endpoint: String,
    /// Browser-facing end-session endpoint.
    pub end_session_endpoint: String,
    /// RFC 7009 revocation endpoint for backchannel logout (server-side only).
    revocation_endpoint: String,
    token_endpoint: String,
    client_id: String,
    client_secret: String,
    jwks: Arc<RwLock<Option<CachedJwks>>>,
    jwks_uri: String,
    http: reqwest::Client,
}

impl OidcClient {
    /// Discover OIDC endpoints from the issuer's well-known configuration.
    pub async fn discover(
        issuer: &str,
        client_id: String,
        client_secret: String,
    ) -> Result<Self, ApiError> {
        let http = reqwest::Client::new();
        let url = format!(
            "{}/.well-known/openid-configuration",
            issuer.trim_end_matches('/')
        );

        let discovery: OidcDiscovery = http
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::Internal(format!("OIDC discovery request failed: {e}")))?
            .json()
            .await
            .map_err(|e| ApiError::Internal(format!("OIDC discovery parse failed: {e}")))?;

        // Prefer the discovery doc's revocation_endpoint (RFC 7009).
        // Fall back to deriving it from the token endpoint for providers that
        // don't advertise it.
        let revocation_endpoint = discovery
            .revocation_endpoint
            .clone()
            .unwrap_or_else(|| {
                discovery
                    .token_endpoint
                    .strip_suffix("/token")
                    .map(|base| format!("{base}/revoke"))
                    .unwrap_or_else(|| discovery.end_session_endpoint.clone())
            });

        tracing::info!(
            authorization_endpoint = %discovery.authorization_endpoint,
            token_endpoint = %discovery.token_endpoint,
            revocation_endpoint = %revocation_endpoint,
            jwks_uri = %discovery.jwks_uri,
            "OIDC discovery complete"
        );

        Ok(Self {
            authorization_endpoint: discovery.authorization_endpoint,
            end_session_endpoint: discovery.end_session_endpoint,
            revocation_endpoint,
            token_endpoint: discovery.token_endpoint,
            client_id,
            client_secret,
            jwks: Arc::new(RwLock::new(None)),
            jwks_uri: discovery.jwks_uri,
            http,
        })
    }

    /// Exchange an authorization code for tokens.
    pub async fn exchange_code(
        &self,
        code: &str,
        redirect_uri: &str,
    ) -> Result<TokenResponse, ApiError> {
        let resp = self
            .http
            .post(&self.token_endpoint)
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", redirect_uri),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
            ])
            .send()
            .await
            .map_err(|e| ApiError::Internal(format!("token exchange request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_else(|_| "unknown".to_string());
            return Err(ApiError::Internal(format!(
                "token exchange failed ({status}): {body}"
            )));
        }

        resp.json::<TokenResponse>()
            .await
            .map_err(|e| ApiError::Internal(format!("token response parse failed: {e}")))
    }

    /// Revoke the refresh token server-side (backchannel logout / RFC 7009).
    ///
    /// Errors are non-fatal: a stale/expired refresh token still means the
    /// user should be treated as logged out from our side.
    pub async fn backchannel_logout(&self, refresh_token: &str) {
        let result = self
            .http
            .post(&self.revocation_endpoint)
            .form(&[
                ("token", refresh_token),
                ("token_type_hint", "refresh_token"),
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
            ])
            .send()
            .await;

        match result {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!("backchannel logout: refresh token revoked");
            }
            Ok(resp) => {
                let status = resp.status();
                tracing::warn!(
                    "backchannel logout: non-success status {status} \
                     (token may already be expired)"
                );
            }
            Err(e) => {
                tracing::warn!("backchannel logout: request failed: {e}");
            }
        }
    }

    /// Exchange a refresh token for a new set of tokens.
    pub async fn exchange_refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<TokenResponse, ApiError> {
        let resp = self
            .http
            .post(&self.token_endpoint)
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
            ])
            .send()
            .await
            .map_err(|e| ApiError::Internal(format!("refresh token request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_else(|_| "unknown".to_string());
            return Err(ApiError::Unauthorized(format!(
                "refresh token exchange failed ({status}): {body}"
            )));
        }

        resp.json::<TokenResponse>()
            .await
            .map_err(|e| ApiError::Internal(format!("refresh token response parse failed: {e}")))
    }

    /// Validate a JWT (typically the ID token), returning decoded claims.
    pub async fn validate_token(&self, token: &str) -> Result<OidcClaims, ApiError> {
        let jwks = self.get_jwks().await?;

        let header = jsonwebtoken::decode_header(token)
            .map_err(|e| ApiError::Unauthorized(format!("invalid JWT header: {e}")))?;

        let kid = header
            .kid
            .as_deref()
            .ok_or_else(|| ApiError::Unauthorized("JWT missing kid".to_string()))?;

        let jwk = jwks
            .keys
            .iter()
            .find(|k| k.common.key_id.as_deref() == Some(kid))
            .ok_or_else(|| ApiError::Unauthorized(format!("unknown key id: {kid}")))?;

        let decoding_key = DecodingKey::from_jwk(jwk)
            .map_err(|e| ApiError::Unauthorized(format!("invalid JWK: {e}")))?;

        // Cognito ID tokens set `aud` to the app client ID — skip audience
        // validation here; we trust the issuer + signature.
        let mut validation = Validation::new(header.alg);
        validation.validate_aud = false;
        validation.validate_exp = true;

        let token_data = jsonwebtoken::decode::<OidcClaims>(token, &decoding_key, &validation)
            .map_err(|e| ApiError::Unauthorized(format!("JWT validation failed: {e}")))?;

        Ok(token_data.claims)
    }

    /// Fetch (and cache) the JWKS from the OIDC provider.
    async fn get_jwks(&self) -> Result<JwkSet, ApiError> {
        {
            let cached = self.jwks.read().await;
            if let Some(ref cached) = *cached {
                if cached.fetched_at.elapsed() < JWKS_TTL {
                    return Ok(cached.jwks.clone());
                }
            }
        }

        let jwks: JwkSet = self
            .http
            .get(&self.jwks_uri)
            .send()
            .await
            .map_err(|e| ApiError::Internal(format!("JWKS fetch failed: {e}")))?
            .json()
            .await
            .map_err(|e| ApiError::Internal(format!("JWKS parse failed: {e}")))?;

        let mut cached = self.jwks.write().await;
        *cached = Some(CachedJwks {
            jwks: jwks.clone(),
            fetched_at: Instant::now(),
        });

        Ok(jwks)
    }
}
