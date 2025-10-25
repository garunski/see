//! Setting model

use serde::{Deserialize, Serialize};
use tracing::debug;

/// Represents a setting in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,              // e.g., 'app_settings', 'theme'
    pub value: serde_json::Value, // Any JSON value
    pub description: Option<String>,
    pub metadata: serde_json::Value,
}

impl Setting {
    /// Create a new setting
    pub fn new(key: String, value: serde_json::Value) -> Self {
        debug!("Creating new setting: {}", key);
        Self {
            key,
            value,
            description: None,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    /// Create a setting with description
    pub fn new_with_description(
        key: String,
        value: serde_json::Value,
        description: String,
    ) -> Self {
        debug!("Creating new setting with description: {}", key);
        Self {
            key,
            value,
            description: Some(description),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    /// Update the setting value
    pub fn update_value(&mut self, value: serde_json::Value) {
        debug!("Updating setting value for key: {}", self.key);
        self.value = value;
    }

    /// Get the setting key
    pub fn get_key(&self) -> &str {
        &self.key
    }

    /// Get the setting value as a string
    pub fn get_value_as_string(&self) -> Option<String> {
        self.value.as_str().map(|s| s.to_string())
    }

    /// Get the setting value as a boolean
    pub fn get_value_as_bool(&self) -> Option<bool> {
        self.value.as_bool()
    }

    /// Get the setting value as a number
    pub fn get_value_as_number(&self) -> Option<f64> {
        self.value.as_f64()
    }

    /// Check if the setting has a specific value
    pub fn has_value(&self, value: &serde_json::Value) -> bool {
        &self.value == value
    }
}
