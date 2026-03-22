use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::id::{FavoriteId, NodeId};

/// A single sidebar favorite — either a pinned internal node or an external URL.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Favorite {
    pub id: FavoriteId,
    pub owner_id: String,
    /// Set when this favorite links to an internal node.
    pub node_id: Option<NodeId>,
    /// Set when this favorite links to an external URL.
    pub url: Option<String>,
    /// Display label. For node favorites this is the node's current title
    /// (resolved server-side on list); for URL favorites it is the user-supplied label.
    pub label: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
}

/// Request body for creating a new favorite.
/// Exactly one of `node_id` or `url` must be provided.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateFavoriteRequest {
    /// UUID of an existing node to pin.
    pub node_id: Option<Uuid>,
    /// External URL to bookmark.
    pub url: Option<String>,
    /// Display label.
    /// Required for URL favorites; for node favorites this is overridden by the node title
    /// at list time but still stored as a fallback.
    pub label: String,
}

/// Reorder request: caller sends the complete ordered list of favorite IDs.
/// Any IDs not belonging to the caller are silently ignored.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReorderFavoritesRequest {
    pub ids: Vec<Uuid>,
}
