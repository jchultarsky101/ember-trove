use axum::{
    Extension, Json, Router,
    extract::{Query, State},
    response::Html,
    routing::{get, post},
};
use axum_extra::extract::{
    PrivateCookieJar,
    cookie::{Cookie, SameSite},
};
use common::auth::{AuthClaims, UserInfo};
use serde::{Deserialize, Serialize};

use crate::{auth::middleware::SESSION_COOKIE, error::ApiError, state::AppState};

/// Cookie that holds the encrypted refresh token.
/// Scoped to `/api/auth/refresh` so the browser never sends it on other requests.
const REFRESH_COOKIE: &str = "ember_trove_refresh";

/// Public auth routes (no JWT required).
pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/auth/login", get(login))
        .route("/auth/callback", get(callback))
        .route("/auth/refresh", post(refresh))
}

/// Protected auth routes (JWT required — layered behind require_auth).
pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/auth/me", get(me))
        .route("/auth/logout", post(logout))
}

#[derive(Serialize)]
struct RedirectResponse {
    redirect_url: String,
}

async fn login(State(state): State<AppState>) -> Json<RedirectResponse> {
    let oidc = state.oidc.as_ref()
        .expect("OIDC not configured — auth is disabled");

    let redirect_uri = format!("{}/api/auth/callback", state.auth.api_external_url);
    let url = format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope=openid+email+profile",
        oidc.authorization_endpoint,
        state.auth.client_id,
        urlencoding::encode(&redirect_uri),
    );
    Json(RedirectResponse { redirect_url: url })
}

#[derive(Deserialize)]
struct CallbackQuery {
    code: String,
}

async fn callback(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Query(params): Query<CallbackQuery>,
) -> Result<(PrivateCookieJar, Html<String>), ApiError> {
    let oidc = state.oidc.as_ref()
        .ok_or_else(|| ApiError::Internal("OIDC not configured — auth is disabled".to_string()))?;

    let redirect_uri = format!("{}/api/auth/callback", state.auth.api_external_url);
    let token_resp = oidc
        .exchange_code(&params.code, &redirect_uri)
        .await?;

    let access_cookie = Cookie::build((SESSION_COOKIE, token_resp.access_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(false)
        .build();

    let mut updated_jar = jar.add(access_cookie);

    if let Some(refresh_token) = token_resp.refresh_token {
        let refresh_cookie = Cookie::build((REFRESH_COOKIE, refresh_token))
            .path("/api/auth/refresh")
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(false)
            .build();
        updated_jar = updated_jar.add(refresh_cookie);
    }

    let frontend_url = &state.auth.frontend_url;

    let html = format!(
        r#"<!DOCTYPE html>
<html><head><title>Redirecting...</title></head>
<body><script>window.location.replace("{frontend_url}");</script></body>
</html>"#
    );

    Ok((updated_jar, Html(html)))
}

async fn refresh(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> Result<(PrivateCookieJar, Json<serde_json::Value>), ApiError> {
    let oidc = state.oidc.as_ref()
        .ok_or_else(|| ApiError::Internal("OIDC not configured".to_string()))?;

    let refresh_token = jar
        .get(REFRESH_COOKIE)
        .map(|c| c.value().to_string())
        .ok_or_else(|| ApiError::Unauthorized("no refresh token".to_string()))?;

    let token_resp = oidc.exchange_refresh_token(&refresh_token).await?;

    let access_cookie = Cookie::build((SESSION_COOKIE, token_resp.access_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(false)
        .build();

    let mut updated_jar = jar.add(access_cookie);

    // Keycloak rotates the refresh token — update the cookie with the new one.
    if let Some(new_refresh) = token_resp.refresh_token {
        let refresh_cookie = Cookie::build((REFRESH_COOKIE, new_refresh))
            .path("/api/auth/refresh")
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(false)
            .build();
        updated_jar = updated_jar.add(refresh_cookie);
    }

    Ok((updated_jar, Json(serde_json::json!({"ok": true}))))
}

async fn me(Extension(claims): Extension<AuthClaims>) -> Json<UserInfo> {
    Json(UserInfo::from(claims))
}

async fn logout(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> (PrivateCookieJar, Json<RedirectResponse>) {
    let updated_jar = jar
        .remove(Cookie::from(SESSION_COOKIE))
        .remove(Cookie::from(REFRESH_COOKIE));
    let redirect = RedirectResponse {
        redirect_url: state.auth.frontend_url.clone(),
    };
    (updated_jar, Json(redirect))
}
