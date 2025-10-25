//! Audit event model

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Represents an audit event in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub workflow_id: Option<String>,
    pub task_id: Option<String>,
    pub event_type: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
    pub instance_id: Option<String>,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(
        workflow_id: Option<String>,
        task_id: Option<String>,
        event_type: String,
        message: String,
        data: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            workflow_id,
            task_id,
            event_type,
            message,
            timestamp: Utc::now(),
            data,
            instance_id: None,
        }
    }
}
