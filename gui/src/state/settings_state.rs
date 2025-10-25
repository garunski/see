use dark_light;
use s_e_e_core::{AppSettings, Theme, WorkflowDefinition};

#[derive(Debug, Clone)]
pub struct SettingsState {
    pub settings: AppSettings,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            settings: AppSettings {
                theme: match dark_light::detect() {
                    dark_light::Mode::Dark => Theme::Dark,
                    dark_light::Mode::Light => Theme::Light,
                },
                auto_save: true,
                notifications: true,
                default_workflow: None,
            },
        }
    }
}

impl SettingsState {
    pub fn change_theme(&mut self, theme: Theme) {
        self.settings.theme = theme;
    }

    pub fn apply_loaded_settings(&mut self, settings: AppSettings) {
        self.settings = settings;
    }

}
