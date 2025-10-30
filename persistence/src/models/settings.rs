use crate::models::Theme;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
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
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn set_auto_save(&mut self, auto_save: bool) {
        self.auto_save = auto_save;
    }

    pub fn set_notifications(&mut self, notifications: bool) {
        self.notifications = notifications;
    }

    pub fn set_default_workflow(&mut self, workflow_id: Option<String>) {
        self.default_workflow = workflow_id;
    }
}
