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
}

/// Token response from the OIDC token endpoint.
#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub id_token: Option<String>,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub expires_in: Option<i64>,
}

/// JWT claims with Keycloak-specific realm_access field.
#[derive(Debug, Deserialize)]
pub struct KeycloakClaims {
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    #[serde(default)]
    pub realm_access: Option<RealmAccess>,
    pub exp: i64,
    #[serde(default)]
    pub aud: Option<AudClaim>,
}

#[derive(Debug, Deserialize)]
pub struct RealmAccess {
    #[serde(default)]
    pub roles: Vec<String>,
}

/// Keycloak can set `aud` as a single string or an array.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum AudClaim {
    Single(String),
    Multiple(Vec<String>),
}

impl AudClaim {
    pub fn contains(&self, value: &str) -> bool {
        match self {
            Self::Single(s) => s == value,
            Self::Multiple(v) => v.iter().any(|s| s == value),
        }
    }
}

/// OIDC client that handles discovery, code exchange, and JWT validation.
pub struct OidcClient {
    pub authorization_endpoint: String,
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

        tracing::info!(
            authorization_endpoint = %discovery.authorization_endpoint,
            token_endpoint = %discovery.token_endpoint,
            jwks_uri = %discovery.jwks_uri,
            "OIDC discovery complete"
        );

        Ok(Self {
            authorization_endpoint: discovery.authorization_endpoint,
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

    /// Validate an access token JWT, returning the decoded claims.
    pub async fn validate_token(&self, token: &str) -> Result<KeycloakClaims, ApiError> {
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

        // Don't require 'aud' — Keycloak omits it by default without an audience
        // mapper configured. When present, the audience mapper adds the client_id
        // or "account"; we validate it opportunistically via the struct field.
        let mut validation = Validation::new(header.alg);
        validation.validate_aud = false;
        validation.validate_exp = true;

        let token_data = jsonwebtoken::decode::<KeycloakClaims>(token, &decoding_key, &validation)
            .map_err(|e| ApiError::Unauthorized(format!("JWT validation failed: {e}")))?;

        Ok(token_data.claims)
    }

    /// Fetch (and cache) the JWKS from the OIDC provider.
    async fn get_jwks(&self) -> Result<JwkSet, ApiError> {
        {
            let cached = self.jwks.read().await;
            if let Some(ref cached) = *cached {
                // Return cached JWKS if still within TTL window.
                if cached.fetched_at.elapsed() < JWKS_TTL {
                    return Ok(cached.jwks.clone());
                }
            }
        }

        // Fetch fresh JWKS from provider.
        let jwks: JwkSet = self
            .http
            .get(&self.jwks_uri)
            .send()
            .await
            .map_err(|e| ApiError::Internal(format!("JWKS fetch failed: {e}")))?
            .json()
            .await
            .map_err(|e| ApiError::Internal(format!("JWKS parse failed: {e}")))?;

        // Update cache with fresh JWKS and current timestamp.
        let mut cached = self.jwks.write().await;
        *cached = Some(CachedJwks {
            jwks: jwks.clone(),
            fetched_at: Instant::now(),
        });

        Ok(jwks)
    }
}
