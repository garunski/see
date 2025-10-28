use dioxus::prelude::*;

use super::{
    create_switch_to_json_handler,
    create_switch_to_visual_handler, use_workflow_edit, EditorHeader, JsonEditor,
    VisualEditor,
};

#[derive(PartialEq, Clone, Copy)]
pub enum EditMode {
    Visual,
    Json,
}

#[component]
pub fn WorkflowEditPage(id: String) -> Element {
    let is_new = id.is_empty();

    let mut state = use_workflow_edit(id.clone());

    // Mode switching handlers
    let mut switch_to_visual_handler =
        create_switch_to_visual_handler(state.content, state.validation_error, state.edit_mode);
    let switch_to_visual = move |_| {
        switch_to_visual_handler();
    };
    let mut switch_to_json_handler =
        create_switch_to_json_handler(state.content, state.edited_workflow_name, state.edit_mode);
    let switch_to_json = move |_| {
        switch_to_json_handler();
    };

    // TODO: Save and reset handlers need to be migrated to use mutations
    let save_workflow = move || {
        tracing::warn!("Save workflow not yet migrated to mutations");
    };

    let reset_to_default = move || {
        tracing::warn!("Reset to default not yet migrated to mutations");
    };

    rsx! {
        div { class: "space-y-8",
            EditorHeader {
                is_new,
                edit_mode: state.edit_mode,
                can_reset: state.can_reset,
                is_saving: state.is_saving,
                has_unsaved_changes: state.has_unsaved_changes,
                on_mode_switch_to_visual: switch_to_visual,
                on_mode_switch_to_json: switch_to_json,
                on_save: move |_| save_workflow(),
                on_reset: move |_| reset_to_default(),
            }

            // Content area - conditional rendering based on edit mode
            match (state.edit_mode)() {
                EditMode::Visual => rsx! {
                    VisualEditor {
                        workflow_json_str: state.workflow_json_str,
                        edited_workflow_name: state.edited_workflow_name,
                        selected_node_info: state.selected_node_info,
                    }
                },
                EditMode::Json => rsx! {
                    JsonEditor {
                        content: state.content,
                        workflow_name: state.workflow_name,
                        validation_error: state.validation_error,
                        on_content_change: move |value| state.content.set(value),
                        is_readonly: None,
                    }
                }
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
