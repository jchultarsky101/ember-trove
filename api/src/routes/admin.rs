//! Admin routes — user management via the Keycloak Admin REST API.
//!
//! All endpoints require a valid JWT **and** the `admin` realm role.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, put},
    Extension, Json, Router,
};
use common::{
    admin::{AdminUser, CreateAdminUserRequest, UpdateUserRolesRequest},
    auth::AuthClaims,
};
use garde::Validate;

use crate::{
    admin::KeycloakAdminClient,
    error::ApiError,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/users/roles", get(list_roles))
        .route("/users/{id}", delete(delete_user))
        .route("/users/{id}/roles", put(set_user_roles))
}

// ── Helper ────────────────────────────────────────────────────────────────────

/// Require the caller to have the `admin` realm role.
fn require_admin_role(claims: &AuthClaims) -> Result<(), ApiError> {
    if claims.roles.contains(&"admin".to_string()) {
        Ok(())
    } else {
        Err(ApiError::Forbidden(
            "admin role required".to_string(),
        ))
    }
}

/// Unwrap the Keycloak admin client or return 503.
fn kc_client(state: &AppState) -> Result<Arc<KeycloakAdminClient>, ApiError> {
    state.keycloak_admin.clone().ok_or_else(|| {
        ApiError::Internal(
            "Keycloak admin is not configured (set KEYCLOAK_ADMIN_USER and KEYCLOAK_ADMIN_PASSWORD)".to_string(),
        )
    })
}

/// Map a `KcUser` + its roles into our `AdminUser` DTO.
async fn kc_user_to_dto(
    kc: &KeycloakAdminClient,
    user: crate::admin::keycloak::KcUser,
) -> AdminUser {
    let realm_roles = kc
        .get_user_roles(&user.id)
        .await
        .unwrap_or_default()
        .into_iter()
        // Filter out Keycloak internal roles (default-roles-*, uma_authorization, etc.)
        .filter(|r| !r.name.starts_with("default-roles") && r.name != "uma_authorization" && r.name != "offline_access")
        .map(|r| r.name)
        .collect();

    AdminUser {
        id: user.id,
        username: user.username,
        email: user.email,
        first_name: user.first_name,
        last_name: user.last_name,
        enabled: user.enabled,
        realm_roles,
    }
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// `GET /admin/users` — list all realm users with their roles.
async fn list_users(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
) -> Result<Json<Vec<AdminUser>>, ApiError> {
    require_admin_role(&claims)?;
    let kc = kc_client(&state)?;

    let kc_users = kc.list_users().await?;

    // Fetch roles for each user concurrently.
    let mut users = Vec::with_capacity(kc_users.len());
    for u in kc_users {
        users.push(kc_user_to_dto(&kc, u).await);
    }

    Ok(Json(users))
}

/// `POST /admin/users` — create a Keycloak user, optionally assign roles and send welcome email.
async fn create_user(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Json(req): Json<CreateAdminUserRequest>,
) -> Result<(StatusCode, Json<AdminUser>), ApiError> {
    require_admin_role(&claims)?;
    req.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;
    let kc = kc_client(&state)?;

    // 1. Create the user.
    let user_id = kc
        .create_user(&req.username, &req.email, &req.first_name, &req.last_name)
        .await?;

    // 2. Assign initial roles if requested.
    if !req.initial_roles.is_empty() {
        let all_roles = kc.list_roles().await?;
        let to_assign: Vec<_> = all_roles
            .into_iter()
            .filter(|r| req.initial_roles.contains(&r.name))
            .collect();
        kc.assign_roles(&user_id, &to_assign).await?;
    }

    // 3. Send password-reset email if requested.
    if req.send_welcome_email {
        // Best-effort — don't fail the whole request if email sending fails.
        if let Err(e) = kc.send_required_actions_email(&user_id).await {
            tracing::warn!(%e, user_id, "failed to send welcome email");
        }
    }

    // 4. Fetch the freshly-created user to return it.
    let all_users = kc.list_users().await?;
    let kc_user = all_users
        .into_iter()
        .find(|u| u.id == user_id)
        .ok_or_else(|| ApiError::Internal("newly created user disappeared".to_string()))?;

    let dto = kc_user_to_dto(&kc, kc_user).await;
    Ok((StatusCode::CREATED, Json(dto)))
}

/// `DELETE /admin/users/{id}` — hard-delete a Keycloak user.
async fn delete_user(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    require_admin_role(&claims)?;
    kc_client(&state)?.delete_user(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `GET /admin/users/roles` — list all available realm roles (filtered to app roles).
async fn list_roles(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
) -> Result<Json<Vec<String>>, ApiError> {
    require_admin_role(&claims)?;
    let roles = kc_client(&state)?
        .list_roles()
        .await?
        .into_iter()
        .filter(|r| !r.name.starts_with("default-roles") && r.name != "uma_authorization" && r.name != "offline_access")
        .map(|r| r.name)
        .collect::<Vec<_>>();
    Ok(Json(roles))
}

/// `PUT /admin/users/{id}/roles` — replace a user's full role set.
async fn set_user_roles(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<String>,
    Json(req): Json<UpdateUserRolesRequest>,
) -> Result<StatusCode, ApiError> {
    require_admin_role(&claims)?;
    let kc = kc_client(&state)?;

    let all_roles = kc.list_roles().await?;

    // Roles to assign = requested names that exist in the realm.
    let desired: Vec<_> = all_roles
        .iter()
        .filter(|r| req.roles.contains(&r.name))
        .cloned()
        .collect();

    // Roles to remove = currently held roles not in the desired set.
    let current = kc.get_user_roles(&id).await?;
    let to_remove: Vec<_> = current
        .into_iter()
        .filter(|r| !req.roles.contains(&r.name))
        .collect();

    kc.remove_roles(&id, &to_remove).await?;
    kc.assign_roles(&id, &desired).await?;

    Ok(StatusCode::NO_CONTENT)
}
