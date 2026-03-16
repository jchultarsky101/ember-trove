use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::id::{NodeId, TagId};
use crate::node::{NodeStatus, NodeType};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchQuery {
    pub q: String,
    pub fuzzy: Option<bool>,
    pub node_type: Option<NodeType>,
    /// When set, only nodes with this status are returned.
    pub status: Option<NodeStatus>,
    /// When set, only nodes that carry this tag are returned.
    pub tag_id: Option<TagId>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchResult {
    pub node_id: NodeId,
    pub title: String,
    pub slug: String,
    pub snippet: Option<String>,
    pub rank: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
}
