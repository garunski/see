use crate::queries::GetWorkflow;
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use engine::parse_workflow;
use tracing;

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
    use_effect(move || {
        if !is_new && !id.is_empty() {
            if let Ok(Ok(Some(workflow))) = use_query(Query::new(id.clone(), GetWorkflow)).suspend()
            {
                content.set(workflow.content.clone());
                workflow_name.set(workflow.get_name().to_string());
                edited_workflow_name.set(workflow.get_name().to_string());
                original_content.set(workflow.content.clone());
                original_name.set(workflow.get_name().to_string());
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
    // Use the engine parser which properly handles the workflow structure
    let workflow_json_str = use_memo(move || {
        let content_str = content();
        if content_str.is_empty() {
            tracing::debug!("Content is empty, skipping visual editor parsing");
            return None;
        }
        match parse_workflow(&content_str) {
            Ok(workflow) => serde_json::to_string(&workflow).ok(),
            Err(e) => {
                tracing::error!(
                    "Failed to parse workflow JSON for visual editor: {} - Content length: {}",
                    e,
                    content_str.len()
                );
                None
            }
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
