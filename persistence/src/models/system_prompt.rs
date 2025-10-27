//! SystemPrompt model
//!
//! This file contains ONLY SystemPrompt struct following Single Responsibility Principle.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// System-defined prompt template
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemPrompt {
    pub id: String,
    pub name: String,
    pub content: String,
    pub description: Option<String>,
    pub template: String,
    pub variables: Vec<String>,
    pub tags: Vec<String>,
    pub metadata: Value,
    pub version: String, // Version of this system prompt
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for SystemPrompt {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            content: String::new(),
            description: None,
            template: String::new(),
            variables: Vec::new(),
            tags: Vec::new(),
            metadata: Value::Object(serde_json::Map::new()),
            version: "1.0.0".to_string(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl SystemPrompt {
    /// Validate prompt
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("System prompt ID cannot be empty".to_string());
        }
        if self.name.is_empty() {
            return Err("System prompt name cannot be empty".to_string());
        }
        if self.content.is_empty() {
            return Err("System prompt content cannot be empty".to_string());
        }
        if self.version.is_empty() {
            return Err("System prompt version cannot be empty".to_string());
        }

        Ok(())
    }

    /// Check if this system prompt needs updating based on version
    pub fn needs_update(&self, new_version: &str) -> bool {
        self.version != new_version
    }
}
