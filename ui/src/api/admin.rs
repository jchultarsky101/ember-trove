use common::admin::{AdminUser, CreateAdminUserRequest, UpdateUserRolesRequest};
use gloo_net::http::Request;

use super::{api_url, parse_json};
use crate::error::UiError;

pub async fn list_admin_users() -> Result<Vec<AdminUser>, UiError> {
    let resp = Request::get(&api_url("/admin/users"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn create_admin_user(req: &CreateAdminUserRequest) -> Result<AdminUser, UiError> {
    let resp = Request::post(&api_url("/admin/users"))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn delete_admin_user(id: &str) -> Result<(), UiError> {
    let resp = Request::delete(&api_url(&format!("/admin/users/{id}")))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    if resp.ok() {
        Ok(())
    } else {
        let status = resp.status();
        let text = resp
            .text()
            .await
            .unwrap_or_else(|_| "unknown error".to_string());
        Err(UiError::api(status, text))
    }
}

pub async fn list_realm_roles() -> Result<Vec<String>, UiError> {
    let resp = Request::get(&api_url("/admin/users/roles"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn set_user_roles(id: &str, req: &UpdateUserRolesRequest) -> Result<(), UiError> {
    let resp = Request::put(&api_url(&format!("/admin/users/{id}/roles")))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    if resp.ok() {
        Ok(())
    } else {
        let status = resp.status();
        let text = resp
            .text()
            .await
            .unwrap_or_else(|_| "unknown error".to_string());
        Err(UiError::api(status, text))
    }
}
