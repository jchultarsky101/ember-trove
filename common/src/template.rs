use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{id::TemplateId, node::NodeType};

/// A reusable body + type scaffold that users can apply when creating new nodes.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NodeTemplate {
    pub id: TemplateId,
    pub name: String,
    pub description: Option<String>,
    pub node_type: NodeType,
    /// Markdown body pre-filled into the editor when the template is used.
    pub body: String,
    /// Cognito sub of the creator.
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateTemplateRequest {
    #[garde(length(min = 1, max = 200))]
    pub name: String,
    #[garde(skip)]
    pub description: Option<String>,
    #[garde(skip)]
    pub node_type: NodeType,
    #[garde(skip)]
    #[serde(default)]
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateTemplateRequest {
    #[garde(length(min = 1, max = 200))]
    pub name: String,
    #[garde(skip)]
    pub description: Option<String>,
    #[garde(skip)]
    pub node_type: NodeType,
    #[garde(skip)]
    pub body: String,
}
