//! Workflow definition model

use serde::{Deserialize, Serialize};
// DateTime not used in this model
use uuid::Uuid;
use tracing::debug;

/// Represents a workflow definition (template) in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub content: String,             // JSON workflow definition
    pub tags: Vec<String>,
    pub is_default: bool,
    pub is_edited: bool,
    pub metadata: serde_json::Value,
}

impl Workflow {
    /// Create a new workflow definition
    pub fn new(name: String, content: String) -> Self {
        debug!("Creating new workflow definition: {}", name);
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: None,
            content,
            tags: Vec::new(),
            is_default: false,
            is_edited: false,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
    
    /// Create a default workflow
    pub fn new_default(name: String, content: String) -> Self {
        debug!("Creating default workflow definition: {}", name);
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: None,
            content,
            tags: Vec::new(),
            is_default: true,
            is_edited: false,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
    
    /// Add a tag to the workflow
    pub fn add_tag(&mut self, tag: String) {
        debug!("Adding tag '{}' to workflow {}", tag, self.id);
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
    
    /// Remove a tag from the workflow
    pub fn remove_tag(&mut self, tag: &str) {
        debug!("Removing tag '{}' from workflow {}", tag, self.id);
        self.tags.retain(|t| t != tag);
    }
    
    /// Mark the workflow as edited
    pub fn mark_edited(&mut self) {
        debug!("Marking workflow {} as edited", self.id);
        self.is_edited = true;
    }
    
    /// Get the workflow name for display
    pub fn get_name(&self) -> &str {
        &self.name
    }
}