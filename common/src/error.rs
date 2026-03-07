use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmberTroveError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}
