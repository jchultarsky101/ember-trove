use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::id::{NodeId, NodeLinkId};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NodeLink {
    pub id: NodeLinkId,
    pub node_id: NodeId,
    pub name: String,
    pub url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateNodeLinkRequest {
    #[garde(length(min = 1, max = 500))]
    pub name: String,
    #[garde(length(min = 1, max = 2048))]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateNodeLinkRequest {
    #[garde(length(min = 1, max = 500))]
    pub name: Option<String>,
    #[garde(length(min = 1, max = 2048))]
    pub url: Option<String>,
}
