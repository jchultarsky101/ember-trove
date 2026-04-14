use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Extension, Json, Router,
};
use common::{
    auth::AuthClaims,
    id::WebhookId,
    webhook::{CreateWebhookRequest, UpdateWebhookRequest, Webhook},
};
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_webhooks).post(create_webhook))
        .route(
            "/{id}",
            axum::routing::put(update_webhook).delete(delete_webhook),
        )
}

async fn list_webhooks(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
) -> Result<Json<Vec<Webhook>>, ApiError> {
    let hooks = state.webhooks.list(&claims.sub).await?;
    Ok(Json(hooks))
}

async fn create_webhook(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Json(req): Json<CreateWebhookRequest>,
) -> Result<(StatusCode, Json<Webhook>), ApiError> {
    let hook = state.webhooks.create(&claims.sub, req).await?;
    Ok((StatusCode::CREATED, Json(hook)))
}

async fn update_webhook(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateWebhookRequest>,
) -> Result<Json<Webhook>, ApiError> {
    let hook = state
        .webhooks
        .update(WebhookId(id), &claims.sub, req)
        .await?;
    Ok(Json(hook))
}

async fn delete_webhook(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state
        .webhooks
        .delete(WebhookId(id), &claims.sub)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
