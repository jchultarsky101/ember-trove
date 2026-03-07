/// JWT validation middleware.
///
/// Phase 1 stub — passes all requests through without validation.
/// Phase 2 will extract and validate the Bearer JWT, injecting
/// `AuthClaims` into request extensions.
use axum::{extract::Request, middleware::Next, response::Response};

pub async fn require_auth(request: Request, next: Next) -> Response {
    next.run(request).await
}
