use crate::components::{Button, ButtonSize, ButtonVariant, PageHeader, SectionCard};
use crate::layout::router::Route;
use crate::services::workflow::read_and_parse_workflow_file;
use crate::state::AppStateProvider;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use rfd::FileDialog;

#[component]
pub fn UploadPage() -> Element {
    let mut state_provider = use_context::<AppStateProvider>();
    let navigator = use_navigator();

    let workflow_file = use_memo(move || state_provider.workflow.read().workflow_file.clone());
    let is_picking_file = use_memo(move || state_provider.ui.read().is_picking_file);
    let error_message = use_signal(String::new);
    let is_saving = use_signal(|| false);

    let mut on_workflow_file_change = move |value: String| {
        state_provider.workflow.write().workflow_file = value;
        state_provider.history.write().clear_viewing();
    };

    let mut pick_file = move || {
        state_provider.ui.write().set_picking_file(true);
        spawn(async move {
            if let Some(path) = FileDialog::new()
                .add_filter("JSON files", &["json"])
                .set_title("Select Workflow File")
                .pick_file()
            {
                if let Some(path_str) = path.to_str() {
                    state_provider.workflow.write().workflow_file = path_str.to_string();
                    state_provider.history.write().clear_viewing();
                }
            }
            state_provider.ui.write().set_picking_file(false);
        });
    };

    let on_save = move || {
        let file_path = state_provider.workflow.read().workflow_file.clone();
        let mut settings_state = state_provider.settings;
        let mut error_signal = error_message;
        let mut saving_signal = is_saving;
        let navigator = navigator;

        if file_path.is_empty() {
            error_signal.set("Please select a workflow file first".to_string());
            return;
        }

        saving_signal.set(true);
        error_signal.set(String::new());

        spawn(async move {
            // Read and parse the workflow file
            match read_and_parse_workflow_file(file_path) {
                Ok(workflow) => {
                    // Add workflow to settings
                    settings_state.write().add_workflow(workflow.clone());

                    // Save to database
                    match s_e_e_core::get_global_store() {
                        Ok(store) => {
                            // Save individual workflow to workflows table
                            if let Err(e) = store.save_workflow(&workflow).await {
                                error_signal
                                    .set(format!("Failed to save workflow to database: {}", e));
                                saving_signal.set(false);
                                return;
                            }

                            // Save settings to database
                            let settings_to_save = settings_state.read().settings.clone();
                            if let Err(e) = store.save_settings(&settings_to_save).await {
                                error_signal.set(format!("Failed to save settings: {}", e));
                                saving_signal.set(false);
                                return;
                            }
                        }
                        Err(e) => {
                            error_signal.set(format!("Database unavailable: {}", e));
                            saving_signal.set(false);
                            return;
                        }
                    }

                    // Navigate to home page
                    navigator.push(Route::HomePage {});
                }
                Err(e) => {
                    error_signal.set(e);
                    saving_signal.set(false);
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
                                oninput: move |evt| on_workflow_file_change(evt.value()),
                                class: "block w-full px-3 py-2 text-sm text-zinc-950 dark:text-white bg-white dark:bg-zinc-800 border border-zinc-300 dark:border-zinc-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                            }
                            Button {
                                variant: ButtonVariant::Secondary,
                                size: ButtonSize::Medium,
                                disabled: Some(is_picking_file()),
                                loading: Some(is_picking_file()),
                                onclick: move |_| pick_file(),
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

                        Button {
                            variant: ButtonVariant::Primary,
                            size: ButtonSize::Large,
                            disabled: Some(is_saving()),
                            loading: Some(is_saving()),
                            onclick: move |_| on_save(),
                            class: "w-full font-semibold".to_string(),
                            if is_saving() { "Saving..." } else { "Save Workflow" }
                        }
                    }
                },
                padding: None,
            }
        }
    }
}
