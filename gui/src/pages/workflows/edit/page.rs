use crate::queries::{use_create_workflow_mutation, use_workflow_query};
use dioxus::prelude::*;
use engine::parse_workflow;
use s_e_e_core::WorkflowDefinition;

use super::{EditorHeader, VisualEditor};

#[component]
pub fn WorkflowEditPage(id: String) -> Element {
    let is_new = id.is_empty();

    // Load workflow data
    let loaded_workflow = if !is_new {
        let (query_state, _refetch) = use_workflow_query(id.clone());

        if query_state.is_loading {
            return rsx! {
                div { class: "flex items-center justify-center h-64",
                    "Loading workflow..."
                }
            };
        }

        if query_state.is_error {
            return rsx! {
                div { class: "text-red-600 dark:text-red-400",
                    "Error loading workflow: {query_state.error.clone().unwrap_or_default()}"
                }
            };
        }

        query_state.data.clone().and_then(|opt| opt)
    } else {
        None
    };

    // Form signals inline (keep it simple!)
    let mut content = use_signal(String::new);
    let mut edited_workflow_name = use_signal(String::new);
    let mut original_content = use_signal(String::new);
    let mut original_name = use_signal(String::new);
    let mut has_unsaved_changes = use_signal(|| false);
    let selected_node_info = use_signal(|| String::from("No node selected"));

    // Load data into form
    use_effect(move || {
        if let Some(workflow) = &loaded_workflow {
            content.set(workflow.content.clone());
            edited_workflow_name.set(workflow.name.clone());
            original_content.set(workflow.content.clone());
            original_name.set(workflow.name.clone());
        }
    });

    // Track unsaved changes
    use_effect(move || {
        let content_changed = content() != original_content();
        let name_changed = edited_workflow_name() != original_name();
        has_unsaved_changes.set(content_changed || name_changed);
    });

    // Prepare workflow JSON for visual editor
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

    // Mutations
    let (mutation_state, create_fn) = use_create_workflow_mutation();
    let is_saving = use_memo(move || mutation_state.read().is_loading);

    // Handlers
    let workflow_id_clone = id.clone();
    let save_workflow = move || {
        let content_str = content();
        let name_str = edited_workflow_name();

        if content_str.is_empty() {
            // TODO: Show validation error
            return;
        }

        // Parse the content to ensure it's valid JSON
        let _json_value: serde_json::Value = match serde_json::from_str(&content_str) {
            Ok(v) => v,
            Err(_) => {
                tracing::error!("Invalid JSON content");
                return;
            }
        };

        // Create/update workflow
        let workflow_id = if is_new {
            format!("custom-workflow-{}", chrono::Utc::now().timestamp())
        } else {
            workflow_id_clone.clone()
        };

        let workflow = WorkflowDefinition {
            id: workflow_id,
            name: name_str,
            description: None,
            content: content_str.clone(),
            is_default: false,
            is_edited: !is_new,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let json_str = match serde_json::to_string(&workflow) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to serialize workflow: {}", e);
                return;
            }
        };

        create_fn(json_str);
    };

    rsx! {
        div { class: "space-y-8",
            EditorHeader {
                is_new,
                workflow_id: id,
                is_saving: *is_saving.read(),
                has_unsaved_changes,
                on_save: move |_| save_workflow(),
            }

            // Visual Editor only
            VisualEditor {
                workflow_json_str,
                edited_workflow_name,
                selected_node_info,
            }
        }
    }
}

#[component]
pub fn WorkflowEditPageNew() -> Element {
    rsx! {
        WorkflowEditPage { id: "".to_string() }
    }
}
