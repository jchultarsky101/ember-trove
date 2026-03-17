use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Claims extracted from a validated JWT access token.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuthClaims {
    /// OIDC subject identifier (unique user ID).
    pub sub: String,
    /// Email address (optional — depends on scope).
    pub email: Option<String>,
    /// Display name.
    pub name: Option<String>,
    /// Realm roles assigned in Keycloak.
    #[serde(default)]
    pub roles: Vec<String>,
    /// Token expiry (Unix timestamp).
    pub exp: i64,
}

/// Public user information returned by `GET /auth/me`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UserInfo {
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub roles: Vec<String>,
}

impl From<AuthClaims> for UserInfo {
    fn from(claims: AuthClaims) -> Self {
        Self {
            sub: claims.sub,
            email: claims.email,
            name: claims.name,
            roles: claims.roles,
        }
    }
}
