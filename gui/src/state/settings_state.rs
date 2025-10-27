use dark_light;
use s_e_e_core::{AppSettings, Theme, WorkflowDefinition};

#[derive(Debug, Clone)]
pub struct SettingsState {
    pub settings: AppSettings,
    pub workflows: Vec<WorkflowDefinition>,
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
            workflows: Vec::new(),
        }
    }
}

impl SettingsState {
    pub fn apply_loaded_settings(&mut self, settings: AppSettings) {
        self.settings = settings;
    }

    pub fn get_workflows(&self) -> &Vec<WorkflowDefinition> {
        &self.workflows
    }

    pub fn add_workflow(&mut self, workflow: WorkflowDefinition) {
        self.workflows.push(workflow);
    }

    pub fn update_workflow(&mut self, id: String, content: String) {
        if let Some(workflow) = self.workflows.iter_mut().find(|w| w.id == id) {
            workflow.content = content;
            workflow.updated_at = chrono::Utc::now();
        }
    }

    pub fn get_workflow(&self, id: String) -> Option<&WorkflowDefinition> {
        self.workflows.iter().find(|w| w.id == id)
    }

    pub fn reset_workflow_to_default(&mut self, id: String, default_content: String) {
        if let Some(workflow) = self.workflows.iter_mut().find(|w| w.id == id) {
            workflow.content = default_content;
            workflow.is_edited = false;
            workflow.updated_at = chrono::Utc::now();
        }
    }
}
