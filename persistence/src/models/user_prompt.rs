//! User prompt model (for user-defined prompts)

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::debug;

/// Represents a user-defined prompt in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPrompt {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub template: String,            // Prompt template with {{variables}}
    pub variables: Vec<String>,      // List of required variables
    pub tags: Vec<String>,
    pub metadata: serde_json::Value,
}

impl UserPrompt {
    /// Create a new user prompt
    pub fn new(name: String, template: String) -> Self {
        debug!("Creating new user prompt: {}", name);
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: None,
            template,
            variables: Vec::new(),
            tags: Vec::new(),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
    
    /// Add a variable to the prompt
    pub fn add_variable(&mut self, variable: String) {
        debug!("Adding variable '{}' to user prompt {}", variable, self.id);
        if !self.variables.contains(&variable) {
            self.variables.push(variable);
        }
    }
    
    /// Remove a variable from the prompt
    pub fn remove_variable(&mut self, variable: &str) {
        debug!("Removing variable '{}' from user prompt {}", variable, self.id);
        self.variables.retain(|v| v != variable);
    }
    
    /// Add a tag to the prompt
    pub fn add_tag(&mut self, tag: String) {
        debug!("Adding tag '{}' to user prompt {}", tag, self.id);
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
    
    /// Remove a tag from the prompt
    pub fn remove_tag(&mut self, tag: &str) {
        debug!("Removing tag '{}' from user prompt {}", tag, self.id);
        self.tags.retain(|t| t != tag);
    }
    
    /// Get the prompt name
    pub fn get_name(&self) -> &str {
        &self.name
    }
    
    /// Check if the prompt has a specific variable
    pub fn has_variable(&self, variable: &str) -> bool {
        self.variables.contains(&variable.to_string())
    }
    
    /// Get all variables in the template
    pub fn extract_variables_from_template(&mut self) {
        debug!("Extracting variables from template for user prompt {}", self.id);
        let mut variables = Vec::new();
        let template = &self.template;
        
        // Simple regex-like extraction of {{variable}} patterns
        let mut chars = template.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '{' && chars.peek() == Some(&'{') {
                chars.next(); // consume second {
                let mut var = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '}' && chars.peek() == Some(&'}') {
                        chars.next(); // consume second }
                        if !var.is_empty() {
                            variables.push(var.trim().to_string());
                        }
                        break;
                    }
                    var.push(ch);
                }
            }
        }
        
        self.variables = variables;
        debug!("Extracted {} variables from template", self.variables.len());
    }
}
