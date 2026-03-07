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
    // Phase 2+: required when S3ObjectStore is wired up
    pub s3_endpoint: Option<String>,
    pub s3_bucket: Option<String>,
    pub s3_access_key: Option<String>,
    pub s3_secret_key: Option<String>,
    pub s3_region: String,
    // Phase 2+: required when OIDC middleware is wired up
    pub oidc_issuer: Option<String>,
    pub oidc_client_id: Option<String>,
    pub oidc_client_secret: Option<String>,
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
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port,
        })
    }
}

fn require(name: &'static str) -> Result<String, ConfigError> {
    env::var(name).map_err(|_| ConfigError::MissingVar(name))
}
