//! Keycloak Admin REST API client.
//!
//! Authenticates against the `master` realm using the `admin-cli` client
//! (resource-owner password grant).  The resulting token is cached with a
//! 30-second safety buffer before its expiry so concurrent requests always
//! get a fresh token without hammering Keycloak.

use std::time::{Duration, Instant};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::error::ApiError;

// ── Internal Keycloak JSON shapes ────────────────────────────────────────────

/// Keycloak user representation returned by the Admin API.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KcUser {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub enabled: bool,
}

/// Keycloak role representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KcRole {
    pub id: String,
    pub name: String,
}

/// Body sent when creating a new Keycloak user.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct KcCreateUserBody {
    username: String,
    email: String,
    first_name: String,
    last_name: String,
    enabled: bool,
    credentials: Vec<serde_json::Value>,
}

/// Token response from the OIDC token endpoint.
#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

// ── Token cache ───────────────────────────────────────────────────────────────

struct CachedToken {
    token: String,
    expires_at: Instant,
}

// ── Client ───────────────────────────────────────────────────────────────────

/// Thin async client for the Keycloak Admin REST API.
pub struct KeycloakAdminClient {
    http: Client,
    base_url: String,
    realm: String,
    admin_user: String,
    admin_password: String,
    token_cache: RwLock<Option<CachedToken>>,
}

impl KeycloakAdminClient {
    pub fn new(
        base_url: String,
        realm: String,
        admin_user: String,
        admin_password: String,
    ) -> Self {
        Self {
            http: Client::new(),
            base_url,
            realm,
            admin_user,
            admin_password,
            token_cache: RwLock::new(None),
        }
    }

    // ── Token management ─────────────────────────────────────────────────────

    /// Returns a valid admin token, refreshing from Keycloak if necessary.
    async fn get_token(&self) -> Result<String, ApiError> {
        // Fast path: read lock — token still valid.
        {
            let guard = self.token_cache.read().await;
            if let Some(cached) = guard.as_ref() {
                if cached.expires_at > Instant::now() {
                    return Ok(cached.token.clone());
                }
            }
        }

        // Slow path: write lock — re-authenticate.
        let mut guard = self.token_cache.write().await;
        // Double-check after acquiring write lock.
        if let Some(cached) = guard.as_ref() {
            if cached.expires_at > Instant::now() {
                return Ok(cached.token.clone());
            }
        }

        let token_url = format!(
            "{}/realms/master/protocol/openid-connect/token",
            self.base_url
        );

        let resp = self
            .http
            .post(&token_url)
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", &self.admin_user),
                ("password", &self.admin_password),
            ])
            .send()
            .await
            .map_err(|e| ApiError::Internal(format!("KC admin token request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(ApiError::Internal(format!(
                "KC admin token error {status}: {body}"
            )));
        }

        let tr: TokenResponse = resp
            .json()
            .await
            .map_err(|e| ApiError::Internal(format!("KC token parse error: {e}")))?;

        // Cache with a 30-second safety buffer.
        let buffer = 30u64;
        let ttl = tr.expires_in.saturating_sub(buffer);
        *guard = Some(CachedToken {
            token: tr.access_token.clone(),
            expires_at: Instant::now() + Duration::from_secs(ttl),
        });

        Ok(tr.access_token)
    }

    // ── User operations ───────────────────────────────────────────────────────

    /// List all users in the realm (up to 200).
    pub async fn list_users(&self) -> Result<Vec<KcUser>, ApiError> {
        let token = self.get_token().await?;
        let url = format!(
            "{}/admin/realms/{}/users?max=200&briefRepresentation=false",
            self.base_url, self.realm
        );
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| ApiError::Internal(format!("KC list_users failed: {e}")))?;
        self.expect_success(&resp, "list_users").await?;
        resp.json::<Vec<KcUser>>()
            .await
            .map_err(|e| ApiError::Internal(format!("KC list_users parse error: {e}")))
    }

    /// Create a user.  Returns the new user's Keycloak UUID (from `Location` header).
    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
        first_name: &str,
        last_name: &str,
    ) -> Result<String, ApiError> {
        let token = self.get_token().await?;
        let url = format!("{}/admin/realms/{}/users", self.base_url, self.realm);

        let body = KcCreateUserBody {
            username: username.to_string(),
            email: email.to_string(),
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            enabled: true,
            credentials: vec![],
        };

        let resp = self
            .http
            .post(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| ApiError::Internal(format!("KC create_user request failed: {e}")))?;

        if resp.status() != reqwest::StatusCode::CREATED {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ApiError::Internal(format!(
                "KC create_user error {status}: {text}"
            )));
        }

        // Keycloak returns the new user URL in the Location header.
        let location = resp
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                ApiError::Internal("KC create_user: missing Location header".to_string())
            })?;

        // Last path segment is the user UUID.
        location
            .rsplit('/')
            .next()
            .map(|s| s.to_string())
            .ok_or_else(|| ApiError::Internal("KC create_user: bad Location header".to_string()))
    }

    /// Hard-delete a user by their Keycloak UUID.
    pub async fn delete_user(&self, id: &str) -> Result<(), ApiError> {
        let token = self.get_token().await?;
        let url = format!("{}/admin/realms/{}/users/{id}", self.base_url, self.realm);
        let resp = self
            .http
            .delete(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| ApiError::Internal(format!("KC delete_user request failed: {e}")))?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(ApiError::NotFound(format!("user {id} not found")));
        }
        self.expect_success(&resp, "delete_user").await
    }

    // ── Role operations ───────────────────────────────────────────────────────

    /// List all realm-level roles.
    pub async fn list_roles(&self) -> Result<Vec<KcRole>, ApiError> {
        let token = self.get_token().await?;
        let url = format!("{}/admin/realms/{}/roles", self.base_url, self.realm);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| ApiError::Internal(format!("KC list_roles request failed: {e}")))?;
        self.expect_success(&resp, "list_roles").await?;
        resp.json::<Vec<KcRole>>()
            .await
            .map_err(|e| ApiError::Internal(format!("KC list_roles parse error: {e}")))
    }

    /// Get the realm roles currently assigned to a user.
    pub async fn get_user_roles(&self, user_id: &str) -> Result<Vec<KcRole>, ApiError> {
        let token = self.get_token().await?;
        let url = format!(
            "{}/admin/realms/{}/users/{user_id}/role-mappings/realm",
            self.base_url, self.realm
        );
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| ApiError::Internal(format!("KC get_user_roles request failed: {e}")))?;
        self.expect_success(&resp, "get_user_roles").await?;
        resp.json::<Vec<KcRole>>()
            .await
            .map_err(|e| ApiError::Internal(format!("KC get_user_roles parse error: {e}")))
    }

    /// Add realm roles to a user.
    pub async fn assign_roles(&self, user_id: &str, roles: &[KcRole]) -> Result<(), ApiError> {
        if roles.is_empty() {
            return Ok(());
        }
        let token = self.get_token().await?;
        let url = format!(
            "{}/admin/realms/{}/users/{user_id}/role-mappings/realm",
            self.base_url, self.realm
        );
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&token)
            .json(roles)
            .send()
            .await
            .map_err(|e| ApiError::Internal(format!("KC assign_roles request failed: {e}")))?;
        self.expect_success(&resp, "assign_roles").await
    }

    /// Remove realm roles from a user.
    pub async fn remove_roles(&self, user_id: &str, roles: &[KcRole]) -> Result<(), ApiError> {
        if roles.is_empty() {
            return Ok(());
        }
        let token = self.get_token().await?;
        let url = format!(
            "{}/admin/realms/{}/users/{user_id}/role-mappings/realm",
            self.base_url, self.realm
        );
        let resp = self
            .http
            .delete(&url)
            .bearer_auth(&token)
            .json(roles)
            .send()
            .await
            .map_err(|e| ApiError::Internal(format!("KC remove_roles request failed: {e}")))?;
        self.expect_success(&resp, "remove_roles").await
    }

    /// Send a "set initial password" email to the user.
    pub async fn send_required_actions_email(&self, user_id: &str) -> Result<(), ApiError> {
        let token = self.get_token().await?;
        let url = format!(
            "{}/admin/realms/{}/users/{user_id}/execute-actions-email",
            self.base_url, self.realm
        );
        let actions = ["UPDATE_PASSWORD"];
        let resp = self
            .http
            .put(&url)
            .bearer_auth(&token)
            .json(&actions)
            .send()
            .await
            .map_err(|e| ApiError::Internal(format!("KC send_email request failed: {e}")))?;
        self.expect_success(&resp, "send_required_actions_email").await
    }

    // ── Helper ────────────────────────────────────────────────────────────────

    /// Returns `Ok(())` if the response is 2xx, otherwise an `ApiError::Internal`
    /// with the status code and body text.
    ///
    /// Note: consumes the response body only on error.
    async fn expect_success(
        &self,
        resp: &reqwest::Response,
        op: &'static str,
    ) -> Result<(), ApiError> {
        if resp.status().is_success() || resp.status() == reqwest::StatusCode::NO_CONTENT {
            Ok(())
        } else {
            Err(ApiError::Internal(format!(
                "KC {op} returned {}",
                resp.status()
            )))
        }
    }
}
