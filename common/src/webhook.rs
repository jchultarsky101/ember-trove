use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::id::WebhookId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Webhook {
    pub id: WebhookId,
    pub owner_id: String,
    pub url: String,
    /// Optional shared secret used to HMAC-sign payloads.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    pub events: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateWebhookRequest {
    #[garde(skip)]
    pub url: String,
    #[garde(skip)]
    pub secret: Option<String>,
    #[garde(skip)]
    #[serde(default = "default_events")]
    pub events: Vec<String>,
}

fn default_events() -> Vec<String> {
    vec![
        "node.created".to_string(),
        "node.updated".to_string(),
        "node.deleted".to_string(),
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateWebhookRequest {
    #[garde(skip)]
    pub url: String,
    #[garde(skip)]
    pub secret: Option<String>,
    #[garde(skip)]
    pub events: Vec<String>,
    #[garde(skip)]
    pub is_active: bool,
}

/// Payload sent to webhook endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WebhookPayload {
    pub event: String,
    pub webhook_id: WebhookId,
    pub node_id: Option<crate::id::NodeId>,
    pub triggered_by: String,
    pub timestamp: DateTime<Utc>,
}
