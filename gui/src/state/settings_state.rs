use dark_light;
use see_core::{AppSettings, Theme, WorkflowDefinition};

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
                workflows: Vec::new(),
            },
        }
    }
}

impl SettingsState {
    pub fn change_theme(&mut self, theme: Theme) {
        self.settings.theme = theme;
    }

    pub fn get_theme(&self) -> Theme {
        self.settings.theme
    }

    pub fn apply_loaded_settings(&mut self, settings: AppSettings) {
        self.settings = settings;
    }

    pub fn add_workflow(&mut self, workflow: WorkflowDefinition) {
        self.settings.workflows.push(workflow);
    }

    pub fn update_workflow(&mut self, id: String, name: String, content: String) {
        if let Some(workflow) = self.settings.workflows.iter_mut().find(|w| w.id == id) {
            workflow.name = name;
            workflow.content = content;
            if workflow.is_default {
                workflow.is_edited = true;
            }
        }
    }

    pub fn reset_workflow_to_default(&mut self, id: String, default_content: String) {
        if let Some(workflow) = self.settings.workflows.iter_mut().find(|w| w.id == id) {
            workflow.content = default_content;
            workflow.is_edited = false;
        }
    }

    pub fn get_workflow(&self, id: String) -> Option<&WorkflowDefinition> {
        self.settings.workflows.iter().find(|w| w.id == id)
    }

    pub fn get_workflows(&self) -> &Vec<WorkflowDefinition> {
        &self.settings.workflows
    }
}
