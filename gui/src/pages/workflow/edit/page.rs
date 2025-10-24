use crate::state::AppStateProvider;
use dioxus::prelude::*;

use super::{
    create_reset_to_default_handler, create_save_workflow_handler, create_switch_to_json_handler,
    create_switch_to_visual_handler, use_workflow_edit, EditorHeader, JsonEditor,
    SaveWorkflowParams, VisualEditor,
};

#[derive(PartialEq, Clone, Copy)]
pub enum EditMode {
    Visual,
    Json,
}

#[component]
pub fn WorkflowEditPage(id: String) -> Element {
    let state_provider = use_context::<AppStateProvider>();
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

    let mut save_workflow_handler = create_save_workflow_handler(SaveWorkflowParams {
        state_provider: state_provider.clone(),
        id: id.clone(),
        content: state.content,
        validation_error: state.validation_error,
        is_saving: state.is_saving,
        edited_workflow_name: state.edited_workflow_name,
        original_content: state.original_content,
        original_name: state.original_name,
        has_unsaved_changes: state.has_unsaved_changes,
    });
    let mut save_workflow = move || {
        save_workflow_handler();
    };

    let mut reset_to_default_handler = create_reset_to_default_handler(
        state_provider.clone(),
        id.clone(),
        state.content,
        state.workflow_name,
        state.can_reset,
    );
    let mut reset_to_default = move || {
        reset_to_default_handler();
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
