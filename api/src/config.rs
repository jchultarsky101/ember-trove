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
    pub s3_endpoint: String,
    pub s3_bucket: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub s3_region: String,
    pub oidc_issuer: String,
    pub oidc_client_id: String,
    pub oidc_client_secret: String,
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
            s3_endpoint: require("S3_ENDPOINT")?,
            s3_bucket: require("S3_BUCKET")?,
            s3_access_key: require("S3_ACCESS_KEY")?,
            s3_secret_key: require("S3_SECRET_KEY")?,
            s3_region: env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            oidc_issuer: require("OIDC_ISSUER")?,
            oidc_client_id: require("OIDC_CLIENT_ID")?,
            oidc_client_secret: require("OIDC_CLIENT_SECRET")?,
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port,
        })
    }
}

fn require(name: &'static str) -> Result<String, ConfigError> {
    env::var(name).map_err(|_| ConfigError::MissingVar(name))
}
