use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::id::TagId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Tag {
    pub id: TagId,
    pub owner_id: String,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateTagRequest {
    #[garde(length(min = 1, max = 100))]
    pub name: String,
    #[garde(length(min = 4, max = 7))]
    #[serde(default = "default_color")]
    pub color: String,
}

fn default_color() -> String {
    "#3b82f6".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateTagRequest {
    pub name: Option<String>,
    pub color: Option<String>,
}
