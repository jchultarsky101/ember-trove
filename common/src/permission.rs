use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::id::{NodeId, PermissionId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PermissionRole {
    Owner,
    Editor,
    Viewer,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Permission {
    pub id: PermissionId,
    pub node_id: NodeId,
    pub subject_id: String,
    pub role: PermissionRole,
    pub granted_by: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct GrantPermissionRequest {
    #[garde(length(min = 1))]
    pub subject_id: String,
    #[garde(skip)]
    pub role: PermissionRole,
}
