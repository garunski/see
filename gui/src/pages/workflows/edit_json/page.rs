use crate::components::{
    IconButton, IconButtonSize, IconButtonVariant, Notification, NotificationData, NotificationType,
};
use crate::pages::workflows::edit::JsonEditor;
use crate::queries::{use_create_workflow_mutation, use_workflow_query};
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use s_e_e_core::WorkflowDefinition;

#[component]
pub fn WorkflowJsonEditPage(id: String) -> Element {
    let navigator = use_navigator();
    let is_new = id.is_empty();

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

    let mut content = use_signal(String::new);
    let workflow_name = use_signal(String::new);
    let validation_error = use_signal(String::new);
    let mut has_unsaved_changes = use_signal(|| false);
    let mut original_content = use_signal(String::new);

    let mut notification = use_signal(|| NotificationData {
        r#type: NotificationType::Success,
        title: String::new(),
        message: String::new(),
        show: false,
    });

    let (mutation_state, create_fn) = use_create_workflow_mutation();
    let is_saving = use_memo(move || mutation_state.read().is_loading);

    use_effect(move || {
        if let Some(workflow) = &loaded_workflow {
            content.set(workflow.content.clone());
            original_content.set(workflow.content.clone());
        }
    });

    use_effect(move || {
        let content_changed = content() != original_content();
        has_unsaved_changes.set(content_changed);
    });

    let save_workflow = move |_| {
        let content_str = content();

        if content_str.is_empty() {
            notification.set(NotificationData {
                r#type: NotificationType::Error,
                title: "Validation Error".to_string(),
                message: "Content cannot be empty".to_string(),
                show: true,
            });
            return;
        }

        let json_value: serde_json::Value = match serde_json::from_str(&content_str) {
            Ok(v) => v,
            Err(e) => {
                notification.set(NotificationData {
                    r#type: NotificationType::Error,
                    title: "Invalid JSON".to_string(),
                    message: format!("Failed to parse JSON: {}", e),
                    show: true,
                });
                return;
            }
        };

        let workflow_name_str = json_value
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unnamed Workflow")
            .to_string();

        let workflow_id = if is_new {
            format!("custom-workflow-{}", chrono::Utc::now().timestamp())
        } else {
            id.clone()
        };

        let workflow = WorkflowDefinition {
            id: workflow_id,
            name: workflow_name_str,
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
                notification.set(NotificationData {
                    r#type: NotificationType::Error,
                    title: "Error".to_string(),
                    message: format!("Failed to save workflow: {}", e),
                    show: true,
                });
                return;
            }
        };

        create_fn(json_str);

        notification.set(NotificationData {
            r#type: NotificationType::Success,
            title: "Saved".to_string(),
            message: "Workflow saved successfully".to_string(),
            show: true,
        });
    };

    rsx! {
        div { class: "space-y-8",

            if notification().show {
                Notification {
                    notification,
                    on_close: move |_| {
                        notification.set(NotificationData {
                            r#type: notification().r#type,
                            title: notification().title,
                            message: notification().message,
                            show: false,
                        });
                    },
                }
            }


            div { class: "flex items-center justify-between",
                div { class: "flex items-center gap-4",
                    IconButton {
                        variant: IconButtonVariant::Ghost,
                        size: IconButtonSize::Medium,
                        onclick: move |_| {
                            if has_unsaved_changes() {

                            }
                            navigator.go_back();
                        },
                        class: Some("inline-flex items-center gap-x-1.5 rounded-md bg-zinc-100 dark:bg-zinc-800 px-3 py-2 text-sm font-semibold text-zinc-900 dark:text-zinc-100 shadow-sm hover:bg-zinc-200 dark:hover:bg-zinc-700".to_string()),
                        icon: Some("arrow_left".to_string()),
                        icon_variant: "outline".to_string(),
                        "Back"
                    }
                    div {
                        h1 { class: "text-xl font-bold text-zinc-900 dark:text-white",
                            if is_new { "Create Workflow (JSON)" } else { "Edit Workflow (JSON)" }
                        }
                        p { class: "mt-2 text-zinc-600 dark:text-zinc-400",
                            if is_new { "Create a new workflow definition in JSON" } else { "Edit workflow definition in JSON" }
                        }
                    }
                }
                div { class: "flex items-center gap-3",
                IconButton {
                    variant: IconButtonVariant::Primary,
                    size: IconButtonSize::Medium,
                    disabled: Some(is_saving()),
                    loading: Some(is_saving()),
                    onclick: save_workflow,
                    icon: Some("save".to_string()),
                    icon_variant: "outline".to_string(),
                    if is_saving() { "Saving..." } else { "Save" }
                }
                }
            }


            JsonEditor {
                content,
                workflow_name,
                validation_error,
                on_content_change: move |value| content.set(value),
                is_readonly: None,
            }
        }
    }
}
