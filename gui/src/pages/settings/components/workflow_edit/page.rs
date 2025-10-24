use crate::components::{Button, ButtonSize, ButtonVariant};
use crate::state::AppStateProvider;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use see_core::WorkflowJson;

use super::{
    create_reset_to_default_handler, create_save_workflow_handler, create_switch_to_json_handler,
    create_switch_to_visual_handler, load_workflow_script, SaveWorkflowParams,
    MESSAGE_LISTENER_SCRIPT,
};

#[derive(PartialEq, Clone, Copy)]
pub enum EditMode {
    Visual,
    Json,
}

#[component]
pub fn WorkflowEditPage(id: String) -> Element {
    let state_provider = use_context::<AppStateProvider>();
    let navigator = use_navigator();

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
    let _editing_node = use_signal(|| Option::<String>::None);

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

    // Mode switching handlers
    let mut switch_to_visual_handler =
        create_switch_to_visual_handler(content, validation_error, edit_mode);
    let switch_to_visual = move |_| {
        switch_to_visual_handler();
    };
    let mut switch_to_json_handler =
        create_switch_to_json_handler(content, edited_workflow_name, edit_mode);
    let switch_to_json = move |_| {
        switch_to_json_handler();
    };

    let mut save_workflow_handler = create_save_workflow_handler(SaveWorkflowParams {
        state_provider: state_provider.clone(),
        id: id.clone(),
        content,
        validation_error,
        is_saving,
        edited_workflow_name,
        original_content,
        original_name,
        has_unsaved_changes,
    });
    let mut save_workflow = move || {
        save_workflow_handler();
    };

    let mut reset_to_default_handler = create_reset_to_default_handler(
        state_provider.clone(),
        id.clone(),
        content,
        workflow_name,
        can_reset,
    );
    let mut reset_to_default = move || {
        reset_to_default_handler();
    };

    rsx! {
        div { class: "space-y-8",
            div { class: "flex items-center justify-between",
                div { class: "flex items-center gap-4",
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Medium,
                        onclick: move |_| {
                            if has_unsaved_changes() {
                                // For now, just navigate back - in a real app you'd want a proper confirmation dialog
                                // TODO: Implement proper confirmation dialog using Dioxus components
                            }
                            // Navigate back using Dioxus router
                            navigator.go_back();
                        },
                        class: "inline-flex items-center gap-x-1.5 rounded-md bg-zinc-100 dark:bg-zinc-800 px-3 py-2 text-sm font-semibold text-zinc-900 dark:text-zinc-100 shadow-sm hover:bg-zinc-200 dark:hover:bg-zinc-700",
                        svg { class: "-ml-0.5 h-4 w-4", view_box: "0 0 20 20", fill: "currentColor",
                            path { fill_rule: "evenodd", d: "M17 10a.75.75 0 01-.75.75H5.612l2.158 1.96a.75.75 0 11-1.04 1.08l-3.5-3.25a.75.75 0 010-1.08l3.5-3.25a.75.75 0 111.04 1.08L5.612 9.25H16.25A.75.75 0 0117 10z", clip_rule: "evenodd" }
                        }
                        "Back"
                    }
                    div {
                        h1 { class: "text-xl font-bold text-zinc-900 dark:text-white",
                            if is_new { "Create Workflow" } else { "Edit Workflow" }
                        }
                        p { class: "mt-2 text-zinc-600 dark:text-zinc-400",
                            if is_new { "Create a new workflow definition" } else { "Edit workflow definition" }
                        }
                    }
                }
                div { class: "flex items-center gap-3",
                    // Mode toggle buttons
                    div { class: "flex rounded-lg bg-zinc-100 dark:bg-zinc-800 p-1",
                        Button {
                            variant: if edit_mode() == EditMode::Visual { ButtonVariant::Primary } else { ButtonVariant::Ghost },
                            size: ButtonSize::Small,
                            onclick: switch_to_visual,
                            "Visual Editor"
                        }
                        Button {
                            variant: if edit_mode() == EditMode::Json { ButtonVariant::Primary } else { ButtonVariant::Ghost },
                            size: ButtonSize::Small,
                            onclick: switch_to_json,
                            "JSON Editor"
                        }
                    }

                    if can_reset() {
                        Button {
                            variant: ButtonVariant::Danger,
                            size: ButtonSize::Medium,
                            onclick: move |_| reset_to_default(),
                            "Reset to Default"
                        }
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        size: ButtonSize::Medium,
                        disabled: Some(is_saving()),
                        loading: Some(is_saving()),
                        onclick: move |_| save_workflow(),
                        if is_saving() { "Saving..." } else { "Save" }
                    }
                }
            }

            // Content area - conditional rendering based on edit mode
            match edit_mode() {
                EditMode::Visual => rsx! {
                    // Message listener for node clicks from iframe
                    script {
                        dangerous_inner_html: MESSAGE_LISTENER_SCRIPT.to_string()
                    }

                        div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 shadow-sm",
                        div { class: "p-4 border-b border-zinc-200 dark:border-zinc-700",
                            div { class: "flex items-center justify-between",
                                h3 { class: "text-lg font-semibold text-zinc-900 dark:text-white",
                                    "Visual Editor"
                                }
                                div { class: "text-sm text-zinc-500 dark:text-zinc-400",
                                    "Drag nodes to reposition, double-click to edit"
                                }
                            }
                        }
                        div { class: "relative", style: "height: 600px",
                            // Selected node info display
                            div {
                                id: "selected-node-info",
                                class: "absolute top-4 right-4 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 px-3 py-2 rounded-lg text-sm font-medium z-10",
                                "{selected_node_info()}"
                            }

                            if let Some(json_str) = workflow_json_str() {
                                // Script to send workflow data to iframe and set up click handling
                                script {
                                    dangerous_inner_html: load_workflow_script(&json_str, &edited_workflow_name())
                                }

                                iframe {
                                    id: "workflow-editor-iframe",
                                    srcdoc: format!(
                                        r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Workflow Editor</title>
    <link rel="stylesheet" href="{}" />
    <link rel="stylesheet" href="{}" />
    <script>
      // Set mode before React app loads
      window.WORKFLOW_MODE = 'editor';
    </script>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="{}"></script>
  </body>
</html>"#,
                                        asset!("/assets/workflow-visualizer/index.css"),
                                        asset!("/assets/tailwind.css"),
                                        asset!("/assets/workflow-visualizer/index.js")
                                    ),
                                    class: "w-full h-full border-0 rounded-b-xl",
                                }
                            } else {
                                div { class: "flex items-center justify-center h-full",
                                    div { class: "text-center",
                                        div { class: "text-red-600 dark:text-red-400 mb-2", "Invalid Workflow" }
                                        p { class: "text-zinc-600 dark:text-zinc-400", "Please fix the JSON before switching to visual mode" }
                                    }
                                }
                            }
                        }
                    }
                },
                EditMode::Json => rsx! {
                    div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 p-8 shadow-sm",
                        div { class: "space-y-6",
                            div {
                                label { class: "block text-sm font-medium text-zinc-900 dark:text-white mb-2",
                                    "Workflow Name"
                                }
                                div { class: "block w-full rounded-md border-0 py-1.5 text-zinc-900 dark:text-white shadow-sm ring-1 ring-inset ring-zinc-300 dark:ring-zinc-600 bg-zinc-50 dark:bg-zinc-700 sm:text-sm sm:leading-6",
                                    {workflow_name()}
                                }
                                p { class: "mt-1 text-xs text-zinc-500 dark:text-zinc-400",
                                    "Name is extracted from the JSON 'name' field"
                                }
                            }

                            div {
                                label { class: "block text-sm font-medium text-zinc-900 dark:text-white mb-2",
                                    "Workflow Definition (JSON)"
                                }
                                textarea {
                                    value: "{content()}",
                                    oninput: move |evt| content.set(evt.value()),
                                    placeholder: "Enter workflow JSON definition",
                                    rows: 20,
                                    class: "block w-full rounded-md border-0 py-1.5 text-zinc-900 dark:text-white shadow-sm ring-1 ring-inset ring-zinc-300 dark:ring-zinc-600 placeholder:text-zinc-400 dark:placeholder:text-zinc-500 focus:ring-2 focus:ring-inset focus:ring-blue-600 dark:bg-zinc-700 sm:text-sm sm:leading-6 font-mono"
                                }
                                if !validation_error().is_empty() {
                                    div { class: "mt-2 text-sm text-red-600 dark:text-red-400",
                                        {validation_error()}
                                    }
                                }
                            }
                        }
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
