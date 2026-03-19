//! Amazon Cognito Identity Provider admin client.
//!
//! Uses the AWS SDK to manage users and groups in a Cognito User Pool.
//! Credentials are loaded from environment variables (`AWS_ACCESS_KEY_ID` /
//! `AWS_SECRET_ACCESS_KEY`) via `aws-config`.

use aws_config::{BehaviorVersion, Region, meta::region::RegionProviderChain};
use aws_sdk_cognitoidentityprovider::{
    Client,
    types::{AttributeType, MessageActionType},
};
use common::admin::{AdminUser, CreateAdminUserRequest};

use crate::error::ApiError;

// ── Client ───────────────────────────────────────────────────────────────────

/// Thin async wrapper around the AWS Cognito Identity Provider SDK.
pub struct CognitoAdminClient {
    client: Client,
    user_pool_id: String,
}

impl CognitoAdminClient {
    /// Build a `CognitoAdminClient` using static credentials from the environment.
    ///
    /// `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` must be set before calling
    /// this function.  `region` defaults to `us-east-2` if blank.
    pub async fn new(region: &str, user_pool_id: String) -> Self {
        let region_provider =
            RegionProviderChain::first_try(Region::new(region.to_string()))
                .or_default_provider();

        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;

        Self {
            client: Client::new(&config),
            user_pool_id,
        }
    }

    // ── User operations ───────────────────────────────────────────────────────

    /// List all users in the pool (up to 60 per SDK page; we collect all pages).
    pub async fn list_users(&self) -> Result<Vec<AdminUser>, ApiError> {
        let mut users = Vec::new();
        let mut pagination_token: Option<String> = None;

        loop {
            let mut req = self
                .client
                .list_users()
                .user_pool_id(&self.user_pool_id);

            if let Some(ref token) = pagination_token {
                req = req.pagination_token(token);
            }

            let resp = req
                .send()
                .await
                .map_err(|e| ApiError::Internal(format!("Cognito list_users failed: {e}")))?;

            for u in resp.users() {
                // Fetch group memberships for each user.
                let groups = self.list_user_groups(u.username().unwrap_or_default()).await?;
                users.push(cognito_user_to_dto(u, groups));
            }

            pagination_token = resp.pagination_token().map(str::to_string);
            if pagination_token.is_none() {
                break;
            }
        }

        Ok(users)
    }

    /// Create a user.  Returns the new `AdminUser` DTO.
    ///
    /// If `send_welcome_email` is true, Cognito sends a temporary-password email.
    /// Otherwise the user is created in FORCE_CHANGE_PASSWORD state with a
    /// suppressed invitation message.
    pub async fn create_user(&self, req: &CreateAdminUserRequest) -> Result<AdminUser, ApiError> {
        let message_action = if req.send_welcome_email {
            None
        } else {
            Some(MessageActionType::Suppress)
        };

        let mut builder = self
            .client
            .admin_create_user()
            .user_pool_id(&self.user_pool_id)
            .username(&req.email)
            .user_attributes(
                AttributeType::builder()
                    .name("email")
                    .value(&req.email)
                    .build()
                    .map_err(|e| ApiError::Internal(format!("Cognito attr build failed: {e}")))?,
            )
            .user_attributes(
                AttributeType::builder()
                    .name("email_verified")
                    .value("true")
                    .build()
                    .map_err(|e| ApiError::Internal(format!("Cognito attr build failed: {e}")))?,
            )
            .user_attributes(
                AttributeType::builder()
                    .name("given_name")
                    .value(&req.first_name)
                    .build()
                    .map_err(|e| ApiError::Internal(format!("Cognito attr build failed: {e}")))?,
            )
            .user_attributes(
                AttributeType::builder()
                    .name("family_name")
                    .value(&req.last_name)
                    .build()
                    .map_err(|e| ApiError::Internal(format!("Cognito attr build failed: {e}")))?,
            );

        if let Some(action) = message_action {
            builder = builder.message_action(action);
        }

        let resp = builder
            .send()
            .await
            .map_err(|e| ApiError::Internal(format!("Cognito admin_create_user failed: {e}")))?;

        let user = resp
            .user()
            .ok_or_else(|| ApiError::Internal("Cognito create_user returned no user".to_string()))?;

        let username = user.username().unwrap_or_default().to_string();

        // Assign initial groups.
        for group in &req.initial_roles {
            self.client
                .admin_add_user_to_group()
                .user_pool_id(&self.user_pool_id)
                .username(&username)
                .group_name(group)
                .send()
                .await
                .map_err(|e| {
                    ApiError::Internal(format!("Cognito add_to_group '{group}' failed: {e}"))
                })?;
        }

        Ok(cognito_user_to_dto(user, req.initial_roles.clone()))
    }

    /// Hard-delete a user by their Cognito username (which is their UUID sub).
    pub async fn delete_user(&self, username: &str) -> Result<(), ApiError> {
        self.client
            .admin_delete_user()
            .user_pool_id(&self.user_pool_id)
            .username(username)
            .send()
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("UserNotFoundException") {
                    ApiError::NotFound(format!("user {username} not found"))
                } else {
                    ApiError::Internal(format!("Cognito delete_user failed: {e}"))
                }
            })?;
        Ok(())
    }

    /// List all groups in the user pool (these are the equivalent of realm roles).
    pub async fn list_groups(&self) -> Result<Vec<String>, ApiError> {
        let mut groups = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let mut req = self
                .client
                .list_groups()
                .user_pool_id(&self.user_pool_id);

            if let Some(ref token) = next_token {
                req = req.next_token(token);
            }

            let resp = req
                .send()
                .await
                .map_err(|e| ApiError::Internal(format!("Cognito list_groups failed: {e}")))?;

            for g in resp.groups() {
                if let Some(name) = g.group_name() {
                    groups.push(name.to_string());
                }
            }

            next_token = resp.next_token().map(str::to_string);
            if next_token.is_none() {
                break;
            }
        }

        Ok(groups)
    }

    /// Get the groups a user currently belongs to.
    pub async fn list_user_groups(&self, username: &str) -> Result<Vec<String>, ApiError> {
        let resp = self
            .client
            .admin_list_groups_for_user()
            .user_pool_id(&self.user_pool_id)
            .username(username)
            .send()
            .await
            .map_err(|e| {
                ApiError::Internal(format!("Cognito list_user_groups failed: {e}"))
            })?;

        Ok(resp
            .groups()
            .iter()
            .filter_map(|g| g.group_name())
            .map(str::to_string)
            .collect())
    }

    /// Replace a user's full group membership with the provided list.
    pub async fn set_user_groups(
        &self,
        username: &str,
        desired: &[String],
    ) -> Result<(), ApiError> {
        let current = self.list_user_groups(username).await?;

        // Add to groups the user is not yet in.
        for group in desired {
            if !current.contains(group) {
                self.client
                    .admin_add_user_to_group()
                    .user_pool_id(&self.user_pool_id)
                    .username(username)
                    .group_name(group)
                    .send()
                    .await
                    .map_err(|e| {
                        ApiError::Internal(format!(
                            "Cognito add_to_group '{group}' failed: {e}"
                        ))
                    })?;
            }
        }

        // Remove from groups no longer desired.
        for group in &current {
            if !desired.contains(group) {
                self.client
                    .admin_remove_user_from_group()
                    .user_pool_id(&self.user_pool_id)
                    .username(username)
                    .group_name(group)
                    .send()
                    .await
                    .map_err(|e| {
                        ApiError::Internal(format!(
                            "Cognito remove_from_group '{group}' failed: {e}"
                        ))
                    })?;
            }
        }

        Ok(())
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Map a Cognito `UserType` + its groups into our `AdminUser` DTO.
fn cognito_user_to_dto(
    user: &aws_sdk_cognitoidentityprovider::types::UserType,
    groups: Vec<String>,
) -> AdminUser {
    let attrs = user.attributes();

    let get_attr = |name: &str| -> Option<String> {
        attrs
            .iter()
            .find(|a| a.name() == name)
            .and_then(|a| a.value())
            .map(str::to_string)
    };

    AdminUser {
        id: user.username().unwrap_or_default().to_string(),
        username: get_attr("email")
            .unwrap_or_else(|| user.username().unwrap_or_default().to_string()),
        email: get_attr("email"),
        first_name: get_attr("given_name"),
        last_name: get_attr("family_name"),
        enabled: user.enabled(),
        realm_roles: groups,
    }
}
