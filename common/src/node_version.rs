use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::id::{NodeId, NodeVersionId};

/// A point-in-time snapshot of a node's body.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NodeVersion {
    pub id: NodeVersionId,
    pub node_id: NodeId,
    /// Markdown body at the time this version was saved.
    pub body: String,
    /// Cognito sub of the user who saved this version.
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}
