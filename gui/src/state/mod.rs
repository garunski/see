pub mod history_state;
pub mod settings_state;
pub mod ui_state;
pub mod workflow_state;

pub use history_state::HistoryState;
pub use settings_state::SettingsState;
pub use ui_state::UIState;
pub use workflow_state::WorkflowState;

use dioxus::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum SidebarTab {
    Upload,
    History,
}

// State provider for context
#[derive(Clone)]
pub struct AppStateProvider {
    pub workflow: Signal<WorkflowState>,
    pub ui: Signal<UIState>,
    pub history: Signal<HistoryState>,
    pub settings: Signal<SettingsState>,
}

impl Default for AppStateProvider {
    fn default() -> Self {
        Self {
            workflow: Signal::new(WorkflowState::default()),
            ui: Signal::new(UIState::default()),
            history: Signal::new(HistoryState::default()),
            settings: Signal::new(SettingsState::default()),
        }
    }
}

impl AppStateProvider {
    pub fn new() -> Self {
        Self::default()
    }
}
