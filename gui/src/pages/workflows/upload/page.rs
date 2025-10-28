use crate::components::{IconButton, IconButtonSize, IconButtonVariant, PageHeader, SectionCard};
use crate::layout::router::Route;
use crate::services::workflow::read_and_parse_workflow_file;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use rfd::FileDialog;

use super::hooks::use_upload_workflow;

#[component]
pub fn UploadPage() -> Element {
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

        spawn(async move {
            // Read and parse the workflow file
            match read_and_parse_workflow_file(workflow_file().clone()) {
                Ok(workflow) => {
                    // Upload using mutation
                    let json_str = match serde_json::to_string(&workflow) {
                        Ok(s) => s,
                        Err(e) => {
                            error_message.set(format!("Failed to serialize workflow: {}", e));
                            return;
                        }
                    };

                    upload_state.create_mutation.mutate(json_str);

                    // Navigate after a short delay to allow mutation to complete
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    navigator.push(Route::HomePage {});
                }
                Err(e) => {
                    error_message.set(e);
                }
            }
        });
    };

    rsx! {
        div { class: "space-y-8",
            PageHeader {
                title: "Upload Workflow".to_string(),
                description: "Upload workflow files to save them to your workflow library".to_string(),
                actions: None,
            }

            SectionCard {
                title: None,
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

                        if !workflow_file().is_empty() && workflow_file() != "workflow.json" {
                            div { class: "p-4 bg-zinc-50 dark:bg-zinc-800 rounded-lg border border-zinc-200 dark:border-zinc-700",
                                p { class: "text-sm text-zinc-700 dark:text-zinc-300", "Selected file: {workflow_file()}" }
                            }
                        }

                        if !error_message().is_empty() {
                            div { class: "p-4 bg-red-50 dark:bg-red-900/20 rounded-lg border border-red-200 dark:border-red-700",
                                p { class: "text-sm text-red-700 dark:text-red-300", "{error_message()}" }
                            }
                        }

                        IconButton {
                            variant: IconButtonVariant::Primary,
                            size: IconButtonSize::Large,
                            disabled: Some((upload_state.is_saving)()),
                            loading: Some((upload_state.is_saving)()),
                            onclick: move |_| on_save(),
                            class: Some("w-full font-semibold".to_string()),
                            icon: if (upload_state.is_saving)() { None } else { Some("save".to_string()) },
                            icon_variant: "outline".to_string(),
                            if (upload_state.is_saving)() { "Saving..." } else { "Save Workflow" }
                        }
                    }
                },
                padding: None,
            }
        }
    }
}
