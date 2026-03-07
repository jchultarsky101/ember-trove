use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::node::NodeType;
use crate::id::NodeId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchQuery {
    pub q: String,
    pub fuzzy: Option<bool>,
    pub node_type: Option<NodeType>,
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
