pub mod admin;
pub mod backup;
pub mod favorites;
pub mod metrics;
pub mod notes;
pub mod attachments;
pub mod auth;
pub mod edges;
pub mod graph;
pub mod nodes;
pub mod permissions;
pub mod search;
pub mod tags;
pub mod tasks;

use axum::{extract::State, middleware, routing::get, Json, Router};
use axum::http::{header, Method, StatusCode};
use chrono::Utc;
use serde_json::{json, Value};
use std::time::Duration;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::timeout::TimeoutLayer;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor};

use crate::{auth::middleware::require_auth, state::AppState};

pub fn build_router(state: AppState) -> anyhow::Result<Router> {
    // --- Rate limiting -------------------------------------------------------
    // 10 req/s sustained per peer IP; burst cap of 100 tokens.
    // Only applies to mutating methods (POST/PUT/DELETE/PATCH) to keep read
    // traffic snappy for a single-user PKM.
    // SmartIpKeyExtractor reads X-Real-IP / X-Forwarded-For (set by nginx) and
    // falls back to ConnectInfo<SocketAddr> for direct connections.
    // const_* methods are only on PeerIpKeyExtractor builders, so we set period
    // and burst first, then switch extractor via key_extractor().
    let mut base = GovernorConfigBuilder::default()
        .const_per_millisecond(100) // replenish 1 token per 100 ms = 10 req/s
        .const_burst_size(100);
    let mut smart = base.key_extractor(SmartIpKeyExtractor);
    let governor_conf = smart
        .finish()
        .ok_or_else(|| anyhow::anyhow!("invalid rate-limit governor config"))?;
    let governor_limiter = governor_conf.limiter().clone();
    // Spawn background task to evict stale IP entries from the limiter's map.
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            governor_limiter.retain_recent();
        }
    });
    let rate_limit = GovernorLayer::new(std::sync::Arc::new(governor_conf));

    // --- CORS ----------------------------------------------------------------
    // Fall back to permissive origin if parse somehow fails (shouldn't — env validated at startup).
    let origin = state
        .auth
        .frontend_url
        .parse()
        .map(AllowOrigin::exact)
        .unwrap_or_else(|_| AllowOrigin::any());

    // With allow_credentials(true), headers and methods must be explicit (not Any).
    let cors = CorsLayer::new()
        .allow_origin(origin)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
            header::COOKIE,
        ])
        // Expose X-Request-Id so clients can correlate logs with server traces.
        .expose_headers([header::HeaderName::from_static("x-request-id")])
        .allow_credentials(true);

    // Protected routes — require a valid JWT.
    let protected = Router::new()
        .merge(auth::protected_router())
        .nest("/nodes", nodes::router())
        .nest("/nodes/{node_id}/tasks", tasks::node_task_router())
        .nest("/tasks", tasks::task_router())
        .nest("/my-day", tasks::my_day_router())
        .nest("/dashboard/projects", tasks::dashboard_router())
        .nest("/nodes/{node_id}/notes", notes::node_note_router())
        .nest("/notes", notes::note_feed_router())
        .nest("/edges", edges::router())
        .nest("/tags", tags::router())
        .nest("/attachments", attachments::router())
        .nest("/search", search::router())
        .nest("/graph", graph::router())
        .nest("/permissions", permissions::router())
        .nest("/favorites", favorites::router())
        .nest("/admin", admin::router())
        .nest("/admin/backups", backup::router())
        .nest("/metrics", metrics::router())
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));

    // Rate-limited public routes (auth callbacks, etc.) + protected routes.
    // Health is intentionally excluded — monitoring tools must never be rate-limited.
    let rate_limited = Router::new()
        .merge(auth::public_router())
        .merge(protected)
        .layer(rate_limit);

    // Public routes — no auth required.
    let router = Router::new()
        .route("/health", get(health))
        .merge(rate_limited)
        .layer(cors)
        // 30-second request timeout — returns 408 Request Timeout.
        .layer(TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(30)))
        // Propagate X-Request-Id from requests; generate one if absent.
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .with_state(state);
    Ok(router)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use tower_governor::governor::GovernorConfigBuilder;

    /// Verify that the rate-limit configuration we use at startup is valid
    /// (non-zero period and burst size) and therefore `finish()` returns `Some`.
    #[test]
    fn governor_config_is_valid() {
        use tower_governor::key_extractor::SmartIpKeyExtractor;
        let mut base = GovernorConfigBuilder::default()
            .const_per_millisecond(100)
            .const_burst_size(100);
        let conf = base.key_extractor(SmartIpKeyExtractor).finish();
        assert!(conf.is_some(), "governor config with SmartIpKeyExtractor must be valid");
    }
}

async fn health(State(state): State<AppState>) -> Json<Value> {
    let db_status = sqlx::query("SELECT 1")
        .fetch_optional(&state.pool)
        .await
        .map(|r| if r.is_some() { "ok" } else { "error" })
        .unwrap_or("error");

    Json(json!({
        "status": if db_status == "ok" { "ok" } else { "degraded" },
        "service": "ember-trove-api",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": Utc::now().to_rfc3339(),
        "database": db_status
    }))
}

