use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::id::{NodeId, NoteId};

/// A note attached to a node.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Note {
    pub id: NoteId,
    pub node_id: NodeId,
    pub owner_id: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Note enriched with the parent node's title — used in the central feed.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FeedNote {
    #[serde(flatten)]
    pub note: Note,
    pub node_title: String,
}

/// Request body for creating a new note.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateNoteRequest {
    #[garde(length(min = 1, max = 10000))]
    pub body: String,
}

/// Request body for editing an existing note.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateNoteRequest {
    #[garde(length(min = 1, max = 10000))]
    pub body: String,
}
