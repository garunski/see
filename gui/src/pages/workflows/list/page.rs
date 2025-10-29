use crate::components::layout::{List, ListItem};
use crate::components::{
    EmptyState, IconButton, IconButtonSize, IconButtonVariant, PageHeader, SectionCard,
};
use crate::icons::Icon;
use crate::layout::router::Route;
use crate::services::workflow::read_and_parse_workflow_file;
use dioxus::prelude::*;
use dioxus_router::prelude::{use_navigator, Link};
use rfd::FileDialog;

use super::hooks::{use_upload_workflow, use_workflows_list};

#[component]
pub fn WorkflowsListPage() -> Element {
    let workflows = match use_workflows_list() {
        Ok(w) => w,
        Err(e) => {
            return rsx! {
                div { class: "space-y-8",
                    PageHeader {
                        title: "Workflows".to_string(),
                        description: "Manage your workflow definitions".to_string(),
                        actions: None,
                    }
                    SectionCard {
                        title: Some("Error".to_string()),
                        children: rsx! {
                            div { class: "text-red-600 dark:text-red-400",
                                "Failed to load workflows: {e}"
                            }
                        },
                        padding: None,
                    }
                }
            };
        }
    };

    let navigator = use_navigator();
    let upload_state = use_upload_workflow();

    let mut workflow_file = use_signal(String::new);
    let mut is_picking_file = use_signal(|| false);
    let mut error_message = use_signal(String::new);

    let mut pick_file = move || {
        is_picking_file.set(true);
        spawn(async move {
            if let Some(path) = FileDialog::new()
                .add_filter("JSON files", &["json"])
                .set_title("Select Workflow File")
                .pick_file()
            {
                if let Some(path_str) = path.to_str() {
                    workflow_file.set(path_str.to_string());
                }
            }
            is_picking_file.set(false);
        });
    };

    let mut on_save = move || {
        if workflow_file().is_empty() {
            error_message.set("Please select a workflow file first".to_string());
            return;
        }

        error_message.set(String::new());

        let mut workflow_file_clone = workflow_file;
        let mut error_message_clone = error_message;
        let upload_fn = upload_state.upload_fn.clone();

        spawn(async move {
            // Read and parse the workflow file
            match read_and_parse_workflow_file(workflow_file_clone().clone()) {
                Ok(workflow) => {
                    // Upload using mutation
                    let json_str = match serde_json::to_string(&workflow) {
                        Ok(s) => s,
                        Err(e) => {
                            error_message_clone.set(format!("Failed to serialize workflow: {}", e));
                            return;
                        }
                    };

                    upload_fn(json_str);

                    // Clear the file input after successful upload
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    workflow_file_clone.set(String::new());
                }
                Err(e) => {
                    error_message_clone.set(e);
                }
            }
        });
    };

    rsx! {
        div { class: "space-y-8",
            PageHeader {
                title: "Workflows".to_string(),
                description: "Upload, create, and manage your workflow definitions".to_string(),
                actions: Some(rsx! {
                    Link {
                        to: Route::WorkflowEditPageNew {},
                        class: "inline-flex items-center gap-x-1.5 rounded-md bg-blue-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-blue-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600",
                        Icon {
                            name: "plus".to_string(),
                            class: Some("-ml-0.5 h-5 w-5".to_string()),
                            size: None,
                            variant: Some("outline".to_string()),
                        }
                        "Create workflow"
                    }
                }),
            }

            // Upload Section
            SectionCard {
                title: Some("Upload Workflow".to_string()),
                children: rsx! {
                    div { class: "space-y-4",
                        div { class: "flex items-center gap-4",
                            input {
                                r#type: "text",
                                placeholder: "Select workflow file...",
                                value: workflow_file(),
                                readonly: true,
                                class: "block w-full px-3 py-2 text-sm text-zinc-950 dark:text-white bg-white dark:bg-zinc-800 border border-zinc-300 dark:border-zinc-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                            }
                            IconButton {
                                variant: IconButtonVariant::Secondary,
                                size: IconButtonSize::Medium,
                                disabled: Some(is_picking_file()),
                                loading: Some(is_picking_file()),
                                onclick: move |_| pick_file(),
                                icon: if is_picking_file() { None } else { Some("upload".to_string()) },
                                icon_variant: "outline".to_string(),
                                if is_picking_file() { "" } else { "Browse" }
                            }
                        }

                        if !workflow_file().is_empty() {
                            div { class: "p-4 bg-zinc-50 dark:bg-zinc-800 rounded-lg border border-zinc-200 dark:border-zinc-700",
                                p { class: "text-sm text-zinc-700 dark:text-zinc-300", "Selected file: {workflow_file()}" }
                            }
                        }

                        if !error_message().is_empty() {
                            div { class: "p-4 bg-red-50 dark:bg-red-900/20 rounded-lg border border-red-200 dark:border-red-700",
                                p { class: "text-sm text-red-700 dark:text-red-300", "{error_message()}" }
                            }
                        }

                        {{
                            let is_saving = upload_state.state.read().is_loading;
                            rsx! {
                                IconButton {
                                    variant: IconButtonVariant::Primary,
                                    size: IconButtonSize::Large,
                                    disabled: Some(is_saving || workflow_file().is_empty()),
                                    loading: Some(is_saving),
                                    onclick: move |_| on_save(),
                                    class: Some("w-full font-semibold".to_string()),
                                    icon: if is_saving { None } else { Some("save".to_string()) },
                                    icon_variant: "outline".to_string(),
                                    if is_saving { "Saving..." } else { "Save Workflow" }
                                }
                            }
                        }}
                    }
                },
                padding: None,
            }

            // Workflows Section
            if workflows.is_empty() {
                SectionCard {
                    title: Some("Workflows".to_string()),
                    children: rsx! {
                        EmptyState {
                            message: "No workflows yet. Create your first workflow to get started.".to_string(),
                        }
                    },
                    padding: None,
                }
            } else {
                SectionCard {
                    title: Some("Workflows".to_string()),
                    children: rsx! {
                        List {
                            for workflow in workflows.iter() {
                                {let workflow_id = workflow.id.clone();
                                rsx! {
                                    ListItem {
                                        icon_name: "workflows".to_string(),
                                        icon_variant: Some("outline".to_string()),
                                        title: rsx! {
                                            {workflow.get_name().to_string()}
                                        },
                                        subtitle: Some(rsx! {
                                            if workflow.is_default {
                                                span { class: "inline-flex items-center rounded-md bg-blue-50 dark:bg-blue-900/20 px-2 py-1 text-xs font-medium text-blue-700 dark:text-blue-300 ring-1 ring-inset ring-blue-700/10",
                                                    "Default"
                                                }
                                            } else {
                                                span { class: "inline-flex items-center rounded-md bg-gray-50 dark:bg-gray-800 px-2 py-1 text-xs font-medium text-gray-600 dark:text-gray-300 ring-1 ring-inset ring-gray-500/10",
                                                    "Custom"
                                                }
                                            }
                                        }),
                                        right_content: Some(rsx! {
                                            if workflow.is_default && workflow.is_edited {
                                                span { class: "inline-flex items-center rounded-md bg-yellow-50 dark:bg-yellow-900/20 px-2 py-1 text-xs font-medium text-yellow-700 dark:text-yellow-300 ring-1 ring-inset ring-yellow-600/10",
                                                    "Modified"
                                                }
                                            } else {
                                                span { class: "inline-flex items-center rounded-md bg-green-50 dark:bg-green-900/20 px-2 py-1 text-xs font-medium text-green-700 dark:text-green-300 ring-1 ring-inset ring-green-600/10",
                                                    "Active"
                                                }
                                            }
                                        }),
                                        onclick: move |_| {
                                            navigator.push(Route::WorkflowEditPage { id: workflow_id.clone() });
                                        },
                                    }
                                }}
                            }
                        }
                    },
                    padding: None,
                }
            }
        }
    }
}
