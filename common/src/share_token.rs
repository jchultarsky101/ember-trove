use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::id::{NodeId, ShareTokenId};

/// A public read-only share link for a node.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ShareToken {
    pub id: ShareTokenId,
    pub node_id: NodeId,
    /// The opaque URL token (UUID).
    pub token: Uuid,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Request body for `POST /nodes/{id}/share`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateShareTokenRequest {
    /// Optional expiry. `None` means the token never expires.
    pub expires_at: Option<DateTime<Utc>>,
}
