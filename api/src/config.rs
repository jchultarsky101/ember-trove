use std::env;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required environment variable: {0}")]
    MissingVar(&'static str),

    #[error("invalid value for {var}: {reason}")]
    InvalidValue { var: &'static str, reason: String },
}

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    // S3 — optional until Phase 6
    pub s3_endpoint: Option<String>,
    pub s3_bucket: Option<String>,
    pub s3_access_key: Option<String>,
    pub s3_secret_key: Option<String>,
    pub s3_region: String,
    // OIDC — optional for Phase 1 dev, required for auth
    pub oidc_issuer: Option<String>,
    pub oidc_client_id: Option<String>,
    pub oidc_client_secret: Option<String>,
    // Cognito Admin credentials — optional; enables /admin/* endpoints when set.
    pub cognito_user_pool_id: Option<String>,
    pub cognito_region: String,
    pub aws_access_key_id: Option<String>,
    pub aws_secret_access_key: Option<String>,
    // Cookie encryption key (128 hex chars → 64 bytes, required by cookie::Key)
    pub cookie_key: String,
    /// Set `Secure` on session cookies. `true` in production (HTTPS), `false` in dev.
    pub cookie_secure: bool,
    // URLs
    pub frontend_url: String,
    pub api_external_url: String,
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3003".to_string())
            .parse::<u16>()
            .map_err(|e| ConfigError::InvalidValue {
                var: "PORT",
                reason: e.to_string(),
            })?;

        let cookie_key = require("COOKIE_KEY")?;
        if cookie_key.len() != 128 {
            return Err(ConfigError::InvalidValue {
                var: "COOKIE_KEY",
                reason: "must be exactly 128 hex characters (64 bytes)".to_string(),
            });
        }

        Ok(Self {
            database_url: require("DATABASE_URL")?,
            s3_endpoint: env::var("S3_ENDPOINT").ok(),
            s3_bucket: env::var("S3_BUCKET").ok(),
            s3_access_key: env::var("S3_ACCESS_KEY").ok(),
            s3_secret_key: env::var("S3_SECRET_KEY").ok(),
            s3_region: env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            oidc_issuer: env::var("OIDC_ISSUER").ok(),
            oidc_client_id: env::var("OIDC_CLIENT_ID").ok(),
            oidc_client_secret: env::var("OIDC_CLIENT_SECRET").ok(),
            cognito_user_pool_id: env::var("COGNITO_USER_POOL_ID").ok(),
            cognito_region: env::var("COGNITO_REGION")
                .unwrap_or_else(|_| "us-east-2".to_string()),
            aws_access_key_id: env::var("AWS_ACCESS_KEY_ID").ok(),
            aws_secret_access_key: env::var("AWS_SECRET_ACCESS_KEY").ok(),
            cookie_key,
            cookie_secure: env::var("COOKIE_SECURE")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            frontend_url: require("FRONTEND_URL")?,
            api_external_url: require("API_EXTERNAL_URL")?,
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port,
        })
    }
}

fn require(name: &'static str) -> Result<String, ConfigError> {
    env::var(name).map_err(|_| ConfigError::MissingVar(name))
}
