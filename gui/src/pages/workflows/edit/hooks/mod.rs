use crate::state::AppStateProvider;
use dioxus::prelude::*;
use see_core::WorkflowJson;

use crate::pages::workflows::edit::EditMode;

#[derive(Clone)]
pub struct WorkflowEditState {
    pub content: Signal<String>,
    pub validation_error: Signal<String>,
    pub is_saving: Signal<bool>,
    pub can_reset: Signal<bool>,
    pub workflow_name: Signal<String>,
    pub edited_workflow_name: Signal<String>,
    pub has_unsaved_changes: Signal<bool>,
    pub original_content: Signal<String>,
    pub original_name: Signal<String>,
    pub edit_mode: Signal<EditMode>,
    pub selected_node_info: Signal<String>,
    pub workflow_json_str: Memo<Option<String>>,
}

pub fn use_workflow_edit(id: String) -> WorkflowEditState {
    let state_provider = use_context::<AppStateProvider>();
    let is_new = id.is_empty();

    let mut content = use_signal(String::new);
    let validation_error = use_signal(String::new);
    let is_saving = use_signal(|| false);
    let mut can_reset = use_signal(|| false);
    let mut workflow_name = use_signal(String::new);
    let mut edited_workflow_name = use_signal(String::new);
    let mut has_unsaved_changes = use_signal(|| false);
    let mut original_content = use_signal(String::new);
    let mut original_name = use_signal(String::new);
    let edit_mode = use_signal(|| EditMode::Visual);
    let selected_node_info = use_signal(|| String::from("No node selected"));

    // Load existing workflow data if editing
    let workflow_id_for_effect = id.clone();
    use_effect(move || {
        if !is_new && !workflow_id_for_effect.is_empty() {
            if let Some(workflow) = state_provider
                .settings
                .read()
                .get_workflow(workflow_id_for_effect.clone())
            {
                content.set(workflow.content.clone());
                workflow_name.set(workflow.get_name());
                edited_workflow_name.set(workflow.get_name());
                original_content.set(workflow.content.clone());
                original_name.set(workflow.get_name());
                can_reset.set(workflow.is_default && workflow.is_edited);
            }
        }
    });

    // Update workflow name when content changes (only in JSON mode)
    use_effect(move || {
        if edit_mode() == EditMode::Json {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content()) {
                if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
                    let name_str = name.to_string();
                    workflow_name.set(name_str);
                } else {
                    workflow_name.set("Unnamed Workflow".to_string());
                }
            } else {
                workflow_name.set("Invalid Workflow".to_string());
            }
        }
    });

    // Track unsaved changes - simplified logic
    use_effect(move || {
        let content_changed = content() != original_content();
        let name_changed = if edit_mode() == EditMode::Visual {
            edited_workflow_name() != original_name()
        } else {
            workflow_name() != original_name()
        };
        has_unsaved_changes.set(content_changed || name_changed);
    });

    // Prepare workflow JSON for visual editor
    let workflow_json_str = use_memo(move || {
        if let Ok(workflow_json) = serde_json::from_str::<WorkflowJson>(&content()) {
            serde_json::to_string(&workflow_json).ok()
        } else {
            None
        }
    });

    WorkflowEditState {
        content,
        validation_error,
        is_saving,
        can_reset,
        workflow_name,
        edited_workflow_name,
        has_unsaved_changes,
        original_content,
        original_name,
        edit_mode,
        selected_node_info,
        workflow_json_str,
    }
}
