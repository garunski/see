use crate::state::AppStateProvider;
use dioxus::prelude::*;
use s_e_e_core::{SystemPrompt, SystemWorkflow, UserPrompt};
use s_e_e_core::{WorkflowDefinition, WorkflowExecutionSummary, WorkflowMetadata};

// Direct context access
pub fn use_app_state() -> AppStateProvider {
    use_context::<AppStateProvider>()
}

// Memoized data access
pub fn use_workflows() -> Memo<Vec<WorkflowDefinition>> {
    let state = use_app_state();
    use_memo(move || state.settings.read().get_workflows().clone())
}

pub fn use_system_workflows() -> Memo<Vec<SystemWorkflow>> {
    let state = use_app_state();
    use_memo(move || state.settings.read().get_system_workflows().clone())
}

pub fn use_prompts() -> Memo<Vec<UserPrompt>> {
    let state = use_app_state();
    use_memo(move || state.prompts.read().get_prompts().clone())
}

pub fn use_system_prompts() -> Memo<Vec<SystemPrompt>> {
    let state = use_app_state();
    use_memo(move || state.prompts.read().get_system_prompts().clone())
}

pub fn use_workflow_history() -> Memo<Vec<WorkflowExecutionSummary>> {
    let state = use_app_state();
    use_memo(move || state.history.read().workflow_history.clone())
}

pub fn use_running_workflows() -> Memo<Vec<WorkflowMetadata>> {
    let state = use_app_state();
    use_memo(move || state.history.read().running_workflows.clone())
}
