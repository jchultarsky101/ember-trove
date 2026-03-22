use garde::Validate;
use serde::{Deserialize, Serialize};

/// A Cognito user as returned by the Admin API, enriched with group memberships.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminUser {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub enabled: bool,
    pub realm_roles: Vec<String>,
}

impl AdminUser {
    /// Display name: "First Last", falling back to username.
    pub fn display_name(&self) -> String {
        match (&self.first_name, &self.last_name) {
            (Some(f), Some(l)) if !f.is_empty() || !l.is_empty() => {
                format!("{} {}", f, l).trim().to_string()
            }
            (Some(f), _) if !f.is_empty() => f.clone(),
            _ => self.username.clone(),
        }
    }
}

/// Request body for creating a new Cognito user.
/// The email address is used as the Cognito username.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateAdminUserRequest {
    #[garde(length(min = 1, max = 254))]
    pub email: String,
    #[garde(length(max = 64))]
    pub first_name: String,
    #[garde(length(max = 64))]
    pub last_name: String,
    /// Realm roles to assign immediately after creation.
    #[garde(skip)]
    pub initial_roles: Vec<String>,
    /// If true, Cognito sends a temporary-password welcome email to the user.
    #[garde(skip)]
    pub send_welcome_email: bool,
}

/// Request body for replacing a user's full set of realm roles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRolesRequest {
    pub roles: Vec<String>,
}
