pub mod middleware;
pub mod oidc;
pub mod permissions;

/// Configuration for OIDC / JWT validation, loaded at startup.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Keycloak realm issuer URL, e.g. `https://keycloak/realms/ember-trove`.
    pub issuer: String,
    /// OIDC client ID registered in Keycloak.
    pub client_id: String,
    /// OIDC client secret.
    pub client_secret: String,
}
