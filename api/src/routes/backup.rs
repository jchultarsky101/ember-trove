//! Backup / restore routes — all require admin role.
//!
//! Mounted under `/api/admin/backups` in `routes/mod.rs`.

use axum::{
    body::Body,
    extract::{Path, State},
    http::{StatusCode, header},
    response::Response,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use common::auth::AuthClaims;
use serde::Deserialize;
use uuid::Uuid;

/// Optional request body for creating a backup.
#[derive(Debug, Deserialize, Default)]
struct CreateBackupRequest {
    /// Optional user-provided comment describing the purpose of the backup.
    #[serde(default)]
    comment: Option<String>,
}

use crate::{
    auth::permissions::require_admin,
    backup as svc,
    error::ApiError,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_backups).post(create_backup_handler))
        .route("/{id}", delete(delete_backup_handler))
        .route("/{id}/download", get(download_backup_handler))
        .route("/{id}/preview", get(preview_restore_handler))
        .route("/{id}/restore", post(execute_restore_handler))
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn list_backups(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
) -> Result<Json<Vec<common::backup::BackupJob>>, ApiError> {
    require_admin(&claims)?;
    let jobs = state
        .backup
        .list_for_owner(&claims.sub)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(jobs))
}

async fn create_backup_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    body: Option<Json<CreateBackupRequest>>,
) -> Result<(StatusCode, Json<common::backup::BackupJob>), ApiError> {
    require_admin(&claims)?;

    // Rate limit: at most one backup per hour per user.
    let existing = state
        .backup
        .list_for_owner(&claims.sub)
        .await
        .map_err(ApiError::from)?;
    if let Some(latest) = existing.first() {
        let age = chrono::Utc::now() - latest.created_at;
        if age < chrono::Duration::hours(1) {
            let mins_left = 60 - age.num_minutes();
            return Err(ApiError::Validation(format!(
                "backup rate limit: try again in {mins_left} minute(s)"
            )));
        }
    }

    let comment = body.and_then(|b| b.0.comment);
    let job = svc::create_backup(&state, &claims.sub, comment.as_deref()).await?;
    Ok((StatusCode::CREATED, Json(job)))
}

async fn delete_backup_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    require_admin(&claims)?;

    // Verify ownership.
    let job = state.backup.get(id).await.map_err(ApiError::from)?;
    if job.created_by != claims.sub {
        return Err(ApiError::Forbidden(
            "backup belongs to a different owner".to_string(),
        ));
    }

    // Delete from S3 (best-effort).
    if let Err(e) = state.object_store.delete(&job.s3_key).await {
        tracing::warn!(key = %job.s3_key, "S3 delete during backup deletion failed: {e}");
    }

    state.backup.delete(id).await.map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn download_backup_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    require_admin(&claims)?;

    let job = state.backup.get(id).await.map_err(ApiError::from)?;
    if job.created_by != claims.sub {
        return Err(ApiError::Forbidden(
            "backup belongs to a different owner".to_string(),
        ));
    }

    let bytes = state
        .object_store
        .get(&job.s3_key)
        .await
        .map_err(|e| ApiError::Storage(format!("backup download failed: {e}")))?;

    let filename = format!("ember-trove-backup-{id}.tar.gz");
    let disposition = format!("attachment; filename=\"{filename}\"");

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/gzip")
        .header(header::CONTENT_DISPOSITION, disposition)
        .header(header::CONTENT_LENGTH, bytes.len())
        .body(Body::from(bytes))
        .map_err(|e| ApiError::Internal(format!("response build failed: {e}")))
}

async fn preview_restore_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
) -> Result<Json<common::backup::BackupPreview>, ApiError> {
    require_admin(&claims)?;
    let preview = svc::preview_restore(&state, id, &claims.sub).await?;
    Ok(Json(preview))
}

async fn execute_restore_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    require_admin(&claims)?;
    svc::execute_restore(&state, id, &claims.sub).await?;
    Ok(StatusCode::ACCEPTED)
}
