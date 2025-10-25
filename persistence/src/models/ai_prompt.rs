//! AI prompt model (for AI model prompts)

use serde::{Deserialize, Serialize};
use tracing::debug;
use uuid::Uuid;

/// Represents an AI model prompt in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiPrompt {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub content: String,       // The actual prompt text
    pub model: Option<String>, // AI model this prompt is for
    pub tags: Vec<String>,
    pub metadata: serde_json::Value,
}

impl AiPrompt {
    /// Create a new AI prompt
    pub fn new(name: String, content: String) -> Self {
        debug!("Creating new AI prompt: {}", name);
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: None,
            content,
            model: None,
            tags: Vec::new(),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    /// Create a new AI prompt with model
    pub fn new_with_model(name: String, content: String, model: String) -> Self {
        debug!("Creating new AI prompt with model: {} for {}", name, model);
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: None,
            content,
            model: Some(model),
            tags: Vec::new(),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    /// Add a tag to the prompt
    pub fn add_tag(&mut self, tag: String) {
        debug!("Adding tag '{}' to AI prompt {}", tag, self.id);
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Remove a tag from the prompt
    pub fn remove_tag(&mut self, tag: &str) {
        debug!("Removing tag '{}' from AI prompt {}", tag, self.id);
        self.tags.retain(|t| t != tag);
    }

    /// Set the AI model
    pub fn set_model(&mut self, model: String) {
        debug!("Setting model '{}' for AI prompt {}", model, self.id);
        self.model = Some(model);
    }

    /// Clear the AI model
    pub fn clear_model(&mut self) {
        debug!("Clearing model for AI prompt {}", self.id);
        self.model = None;
    }

    /// Get the prompt name
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Get the prompt content
    pub fn get_content(&self) -> &str {
        &self.content
    }

    /// Update the prompt content
    pub fn update_content(&mut self, content: String) {
        debug!("Updating content for AI prompt {}", self.id);
        self.content = content;
    }
}
