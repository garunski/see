//! AppSettings model
//! 
//! This file contains ONLY AppSettings struct following Single Responsibility Principle.

use serde::{Deserialize, Serialize};
use crate::models::Theme;

/// Application configuration settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: Theme,
    pub auto_save: bool,
    pub notifications: bool,
    pub default_workflow: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            auto_save: true,
            notifications: true,
            default_workflow: None,
        }
    }
}

impl AppSettings {
    /// Validate settings
    pub fn validate(&self) -> Result<(), String> {
        // All fields have valid defaults, no validation needed
        Ok(())
    }

    /// Update theme
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    /// Update auto-save setting
    pub fn set_auto_save(&mut self, auto_save: bool) {
        self.auto_save = auto_save;
    }

    /// Update notifications setting
    pub fn set_notifications(&mut self, notifications: bool) {
        self.notifications = notifications;
    }

    /// Update default workflow
    pub fn set_default_workflow(&mut self, workflow_id: Option<String>) {
        self.default_workflow = workflow_id;
    }
}
