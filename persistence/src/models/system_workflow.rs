//! SystemWorkflow model
//!
//! This file contains ONLY SystemWorkflow struct following Single Responsibility Principle.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// System-defined workflow template
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemWorkflow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub content: String, // JSON string
    pub version: String, // Version of this system workflow
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for SystemWorkflow {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            description: None,
            content: String::new(),
            version: "1.0.0".to_string(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl SystemWorkflow {
    /// Validate workflow content
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("System workflow ID cannot be empty".to_string());
        }
        if self.name.is_empty() {
            return Err("System workflow name cannot be empty".to_string());
        }
        if self.content.is_empty() {
            return Err("System workflow content cannot be empty".to_string());
        }
        if self.version.is_empty() {
            return Err("System workflow version cannot be empty".to_string());
        }

        // Validate JSON content
        serde_json::from_str::<serde_json::Value>(&self.content)
            .map_err(|e| format!("Invalid JSON content: {}", e))?;

        Ok(())
    }

    /// Check if this system workflow needs updating based on version
    pub fn needs_update(&self, new_version: &str) -> bool {
        self.version != new_version
    }
}
