use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::id::{EdgeId, NodeId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    References,
    Contains,
    RelatedTo,
    DependsOn,
    DerivedFrom,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Edge {
    pub id: EdgeId,
    pub source_id: NodeId,
    pub target_id: NodeId,
    pub edge_type: EdgeType,
    pub label: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateEdgeRequest {
    pub source_id: NodeId,
    pub target_id: NodeId,
    pub edge_type: EdgeType,
    pub label: Option<String>,
}
