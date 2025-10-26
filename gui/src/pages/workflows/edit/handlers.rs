use crate::state::AppStateProvider;
use dioxus::prelude::*;
use s_e_e_core::WorkflowDefinition;
use uuid::Uuid;

use super::EditMode;

/// Parameters for the save workflow handler
pub struct SaveWorkflowParams {
    pub state_provider: AppStateProvider,
    pub id: String,
    pub content: Signal<String>,
    pub validation_error: Signal<String>,
    pub is_saving: Signal<bool>,
    pub edited_workflow_name: Signal<String>,
    pub original_content: Signal<String>,
    pub original_name: Signal<String>,
    pub has_unsaved_changes: Signal<bool>,
}

/// Create a save workflow handler that takes signals as parameters
pub fn create_save_workflow_handler(params: SaveWorkflowParams) -> impl FnMut() {
    let SaveWorkflowParams {
        mut state_provider,
        id,
        mut content,
        mut validation_error,
        mut is_saving,
        edited_workflow_name,
        mut original_content,
        mut original_name,
        mut has_unsaved_changes,
    } = params;
    move || {
        // Update content with edited name before saving
        let mut final_content = content();
        if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&final_content) {
            json["name"] = serde_json::Value::String(edited_workflow_name());
            if let Ok(updated_content) = serde_json::to_string(&json) {
                final_content = updated_content;
            }
        }

        // Validate the workflow JSON using JSON Schema
        if let Err(e) = s_e_e_core::validate_workflow_json(&final_content) {
            validation_error.set(format!("Validation failed:\n{}", e));
            return;
        }

        validation_error.set(String::new());
        is_saving.set(true);

        let is_new = id.is_empty();
        let final_id = if is_new {
            Uuid::new_v4().to_string()
        } else {
            id.clone()
        };

        let workflow = WorkflowDefinition {
            id: final_id.clone(),
            name: "Workflow".to_string(),
            description: Some("Workflow description".to_string()),
            content: final_content.clone(),
            is_default: false,
            is_edited: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        if is_new {
            state_provider
                .settings
                .write()
                .add_workflow(workflow.clone());
        } else {
            state_provider
                .settings
                .write()
                .update_workflow(final_id.clone(), workflow.content.clone());
        }

        // Update local state after successful save
        content.set(final_content.clone());
        original_content.set(final_content);
        original_name.set(edited_workflow_name());
        has_unsaved_changes.set(false);

        let _ui_state = state_provider.ui;
        spawn(async move {
            match s_e_e_core::get_global_store() {
                Ok(store) => {
                    match store
                        .save_settings(&state_provider.settings.read().settings)
                        .await
                    {
                        Ok(_) => {}
                        Err(_e) => {}
                    }
                }
                Err(_e) => {}
            }
            is_saving.set(false);
        });
    }
}

/// Create a reset to default handler
pub fn create_reset_to_default_handler(
    mut state_provider: AppStateProvider,
    id: String,
    mut content: Signal<String>,
    mut workflow_name: Signal<String>,
    mut can_reset: Signal<bool>,
) -> impl FnMut() {
    move || {
        let default_workflows = s_e_e_core::WorkflowDefinition::get_default_workflows();
        if let Some(default_workflow) = default_workflows.iter().find(|w| w.id == id) {
            state_provider
                .settings
                .write()
                .reset_workflow_to_default(id.clone(), default_workflow.content.clone());

            content.set(default_workflow.content.clone());
            workflow_name.set(default_workflow.get_name().to_string().to_string());
            can_reset.set(false);

            let _ui_state = state_provider.ui;
            spawn(async move {
                match s_e_e_core::get_global_store() {
                    Ok(store) => {
                        match store
                            .save_settings(&state_provider.settings.read().settings)
                            .await
                        {
                            Ok(_) => {}
                            Err(_e) => {}
                        }
                    }
                    Err(_e) => {}
                }
            });
        }
    }
}

/// Create a switch to visual mode handler
pub fn create_switch_to_visual_handler(
    content: Signal<String>,
    mut validation_error: Signal<String>,
    mut edit_mode: Signal<EditMode>,
) -> impl FnMut() + 'static {
    move || {
        // Validate JSON before switching
        if let Err(e) = serde_json::from_str::<serde_json::Value>(&content()) {
            validation_error.set(format!("Invalid JSON: {}", e));
            return;
        }

        validation_error.set(String::new());
        // Close any open modal when switching modes
        spawn(async move {
            // Use a simple approach - the modal will be hidden when the component re-renders
        });
        edit_mode.set(EditMode::Visual);
    }
}

/// Create a switch to JSON mode handler
pub fn create_switch_to_json_handler(
    mut content: Signal<String>,
    edited_workflow_name: Signal<String>,
    mut edit_mode: Signal<EditMode>,
) -> impl FnMut() + 'static {
    move || {
        // Update JSON content with edited name before switching
        if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content()) {
            json["name"] = serde_json::Value::String(edited_workflow_name());
            if let Ok(updated_content) = serde_json::to_string(&json) {
                content.set(updated_content);
            }
        }
        // Close any open modal when switching modes
        spawn(async move {
            // Use a simple approach - the modal will be hidden when the component re-renders
        });
        edit_mode.set(EditMode::Json);
    }
}
