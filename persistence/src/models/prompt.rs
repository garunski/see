//! UserPrompt model
//! 
//! This file contains ONLY UserPrompt struct following Single Responsibility Principle.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// User-defined prompt template
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserPrompt {
    pub id: String,
    pub name: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for UserPrompt {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            content: String::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl UserPrompt {
    /// Validate prompt
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Prompt ID cannot be empty".to_string());
        }
        if self.name.is_empty() {
            return Err("Prompt name cannot be empty".to_string());
        }
        if self.content.is_empty() {
            return Err("Prompt content cannot be empty".to_string());
        }
        
        Ok(())
    }

    /// Update the prompt content and timestamp
    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }

    /// Update the prompt name and timestamp
    pub fn update_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }
}
