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

/// Public auth routes (no JWT required).
pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/auth/login", get(login))
        .route("/auth/callback", get(callback))
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

    let cookie = Cookie::build((SESSION_COOKIE, token_resp.access_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(false)
        .build();

    let updated_jar = jar.add(cookie);
    let frontend_url = &state.auth.frontend_url;

    let html = format!(
        r#"<!DOCTYPE html>
<html><head><title>Redirecting...</title></head>
<body><script>window.location.replace("{frontend_url}");</script></body>
</html>"#
    );

    Ok((updated_jar, Html(html)))
}

async fn me(Extension(claims): Extension<AuthClaims>) -> Json<UserInfo> {
    Json(UserInfo::from(claims))
}

async fn logout(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> (PrivateCookieJar, Json<RedirectResponse>) {
    let updated_jar = jar.remove(Cookie::from(SESSION_COOKIE));
    let redirect = RedirectResponse {
        redirect_url: state.auth.frontend_url.clone(),
    };
    (updated_jar, Json(redirect))
}
