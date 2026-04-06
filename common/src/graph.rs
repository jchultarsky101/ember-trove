use serde::{Deserialize, Serialize};

use crate::id::NodeId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePosition {
    pub node_id: NodeId,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavePositionRequest {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavePositionsRequest {
    pub positions: Vec<(NodeId, f64, f64)>,
}
