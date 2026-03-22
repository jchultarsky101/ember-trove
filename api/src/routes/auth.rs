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
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use common::auth::{AuthClaims, UserInfo};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{auth::middleware::SESSION_COOKIE, error::ApiError, state::AppState};

/// Cookie that holds the encrypted refresh token.
/// Scoped to `/api/auth/refresh` so the browser never sends it on other requests.
const REFRESH_COOKIE: &str = "ember_trove_refresh";

/// Short-lived cookie that holds the PKCE `code_verifier`.
/// Consumed once in the callback; path restricted to the callback endpoint.
const PKCE_COOKIE: &str = "ember_trove_pkce";

/// Public auth routes (no JWT required).
pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/auth/login", get(login))
        .route("/auth/callback", get(callback))
        .route("/auth/refresh", post(refresh))
        .route("/auth/logout", post(logout))
}

/// Protected auth routes (JWT required — layered behind require_auth).
pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/auth/me", get(me))
}

#[derive(Serialize)]
struct RedirectResponse {
    redirect_url: String,
}

async fn login(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> Result<(PrivateCookieJar, Json<RedirectResponse>), ApiError> {
    let oidc = state.oidc.as_ref()
        .ok_or_else(|| ApiError::Internal("OIDC not configured — auth is disabled".to_string()))?;

    // PKCE: generate a 32-byte random code_verifier and derive the S256 challenge.
    let mut raw = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut raw);
    let code_verifier = URL_SAFE_NO_PAD.encode(raw);
    let code_challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()));

    let redirect_uri = format!("{}/api/auth/callback", state.auth.api_external_url);
    let url = format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope=openid+email+profile\
         &code_challenge={}&code_challenge_method=S256",
        oidc.authorization_endpoint,
        state.auth.client_id,
        urlencoding::encode(&redirect_uri),
        code_challenge,
    );

    let secure = state.auth.cookie_secure;
    let pkce_cookie = Cookie::build((PKCE_COOKIE, code_verifier))
        .path("/api/auth/callback")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(secure)
        .build();

    Ok((jar.add(pkce_cookie), Json(RedirectResponse { redirect_url: url })))
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

    // Consume the PKCE verifier cookie (single-use).
    let code_verifier = jar
        .get(PKCE_COOKIE)
        .map(|c| c.value().to_string());
    let jar = jar.remove(Cookie::build((PKCE_COOKIE, "")).path("/api/auth/callback").build());

    let redirect_uri = format!("{}/api/auth/callback", state.auth.api_external_url);
    let token_resp = oidc
        .exchange_code(&params.code, &redirect_uri, code_verifier.as_deref())
        .await?;

    // Prefer the ID token for the session cookie — it carries email, name, and
    // cognito:groups which the auth middleware needs.  Fall back to access_token
    // for providers that don't issue a separate ID token.
    let session_token = token_resp.id_token.unwrap_or(token_resp.access_token);

    let secure = state.auth.cookie_secure;

    let access_cookie = Cookie::build((SESSION_COOKIE, session_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(secure)
        .build();

    let mut updated_jar = jar.add(access_cookie);

    if let Some(refresh_token) = token_resp.refresh_token {
        let refresh_cookie = Cookie::build((REFRESH_COOKIE, refresh_token))
            .path("/api/auth/refresh")
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(secure)
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

    let session_token = token_resp.id_token.unwrap_or(token_resp.access_token);
    let secure = state.auth.cookie_secure;

    let access_cookie = Cookie::build((SESSION_COOKIE, session_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(secure)
        .build();

    let mut updated_jar = jar.add(access_cookie);

    // Cognito rotates the refresh token — update the cookie with the new one.
    if let Some(new_refresh) = token_resp.refresh_token {
        let refresh_cookie = Cookie::build((REFRESH_COOKIE, new_refresh))
            .path("/api/auth/refresh")
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(secure)
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
    // Revoke the refresh token server-side (RFC 7009).
    if let Some(oidc) = state.oidc.as_ref() {
        if let Some(refresh_token) = jar.get(REFRESH_COOKIE).map(|c| c.value().to_string()) {
            oidc.backchannel_logout(&refresh_token).await;
        }
    }

    let updated_jar = jar
        .remove(Cookie::build((SESSION_COOKIE, "")).path("/").build())
        .remove(Cookie::build((REFRESH_COOKIE, "")).path("/api/auth/refresh").build());

    // Redirect through Cognito's end-session endpoint so the SSO session cookie
    // at the Cognito domain is cleared.  Without this the browser is silently
    // re-authenticated on the very next /api/auth/login round-trip.
    let redirect_url = if let Some(oidc) = state.oidc.as_ref() {
        format!(
            "{}?client_id={}&logout_uri={}",
            oidc.end_session_endpoint,
            state.auth.client_id,
            urlencoding::encode(&state.auth.frontend_url),
        )
    } else {
        state.auth.frontend_url.clone()
    };

    (updated_jar, Json(RedirectResponse { redirect_url }))
}
