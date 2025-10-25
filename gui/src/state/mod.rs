pub mod history_state;
pub mod prompt_state;
pub mod settings_state;
pub mod ui_state;
pub mod workflow_state;

pub use history_state::HistoryState;
pub use prompt_state::UserPromptState;
pub use settings_state::SettingsState;
pub use ui_state::UIState;
pub use workflow_state::WorkflowState;

use dioxus::prelude::*;

#[derive(Clone)]
pub struct AppStateProvider {
    pub workflow: Signal<WorkflowState>,
    pub ui: Signal<UIState>,
    pub history: Signal<HistoryState>,
    pub settings: Signal<SettingsState>,
    pub prompts: Signal<UserPromptState>,
}

impl Default for AppStateProvider {
    fn default() -> Self {
        Self {
            workflow: Signal::new(WorkflowState::default()),
            ui: Signal::new(UIState::default()),
            history: Signal::new(HistoryState::default()),
            settings: Signal::new(SettingsState::default()),
            prompts: Signal::new(UserPromptState::default()),
        }
    }
}

impl AppStateProvider {
    pub fn new() -> Self {
        Self::default()
    }
}
