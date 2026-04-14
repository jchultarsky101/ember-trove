//! Operational metrics endpoint — admin-only, no auth JWT required beyond role check.
//!
//! `GET /api/metrics` returns a JSON snapshot suitable for CloudWatch dashboards,
//! alerting, or basic health checks.  Fields:
//!
//! - `uptime_secs`        — process uptime in seconds
//! - `db.pool_size`       — current number of open connections (idle + in-use)
//! - `db.pool_idle`       — idle connections waiting to be acquired
//! - `counts.*`           — row counts for each core table
//! - `version`            — value of the `CARGO_PKG_VERSION` build-time env var

use axum::{extract::State, Extension, Json};
use common::auth::AuthClaims;
use serde::Serialize;

use crate::{auth::permissions::require_admin, error::ApiError, state::AppState};

pub fn router() -> axum::Router<AppState> {
    use axum::routing::get;
    axum::Router::new().route("/", get(get_metrics))
}

// ── Response shape ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct Metrics {
    pub version: &'static str,
    pub uptime_secs: u64,
    pub db: DbMetrics,
    pub counts: EntityCounts,
}

#[derive(Serialize)]
pub struct DbMetrics {
    pub pool_size: u32,
    pub pool_idle: u32,
}

#[derive(Serialize)]
pub struct EntityCounts {
    pub nodes: i64,
    pub edges: i64,
    pub tags: i64,
    pub notes: i64,
    pub tasks: i64,
    pub attachments: i64,
    pub favorites: i64,
}

// ── Handler ───────────────────────────────────────────────────────────────────

async fn get_metrics(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
) -> Result<Json<Metrics>, ApiError> {
    require_admin(&claims)?;

    let uptime_secs = state.started_at.elapsed().as_secs();

    let pool_size = state.pool.size();
    let pool_idle = state.pool.num_idle() as u32;

    // Run all count queries concurrently.
    let (nodes, edges, tags, notes, tasks, attachments, favorites) = tokio::try_join!(
        count(&state.pool, "nodes"),
        count(&state.pool, "edges"),
        count(&state.pool, "tags"),
        count(&state.pool, "notes"),
        count(&state.pool, "tasks"),
        count(&state.pool, "attachments"),
        count(&state.pool, "user_favorites"),
    )?;

    Ok(Json(Metrics {
        version: env!("CARGO_PKG_VERSION"),
        uptime_secs,
        db: DbMetrics {
            pool_size,
            pool_idle,
        },
        counts: EntityCounts {
            nodes,
            edges,
            tags,
            notes,
            tasks,
            attachments,
            favorites,
        },
    }))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn count(pool: &sqlx::PgPool, table: &str) -> Result<i64, ApiError> {
    // Table name comes from compile-time string literals — no injection risk.
    let sql = format!("SELECT COUNT(*) FROM {table}");
    sqlx::query_scalar::<_, i64>(&sql)
        .fetch_one(pool)
        .await
        .map_err(|e| ApiError::Internal(format!("metrics count({table}) failed: {e}")))
}
