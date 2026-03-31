use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::id::{NodeId, NoteId};

fn default_note_color() -> String {
    "default".to_string()
}

/// A note attached to a node.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Note {
    pub id: NoteId,
    pub node_id: NodeId,
    pub owner_id: String,
    pub body: String,
    /// Named palette key for the card background colour (e.g. "amber", "rose").
    /// "default" means neutral / no colour.
    #[serde(default = "default_note_color")]
    pub color: String,
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
    /// Palette key: "default" | "amber" | "rose" | "lime" | "sky" | "violet".
    #[garde(length(min = 1, max = 20))]
    #[serde(default = "default_note_color")]
    pub color: String,
}

/// Request body for editing an existing note.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateNoteRequest {
    #[garde(length(min = 1, max = 10000))]
    pub body: String,
    /// Palette key for the note background.
    #[garde(length(min = 1, max = 20))]
    #[serde(default = "default_note_color")]
    pub color: String,
}
