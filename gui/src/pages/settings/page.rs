use crate::components::{Button, ButtonSize, ButtonVariant, ConfirmDialog, PageHeader};
use crate::icons::Icon;
use crate::services::clear_database;
use crate::state::AppStateProvider;
use dioxus::prelude::*;

#[component]
pub fn SettingsPage() -> Element {
    let state_provider = use_context::<AppStateProvider>();

    let mut show_confirm_dialog = use_signal(|| false);

    let clear_database_handler = {
        let state_provider = state_provider.clone();
        let mut show_dialog = show_confirm_dialog;
        move |_| {
            show_dialog.set(false);
            let mut state = state_provider.clone();
            spawn(async move {
                match clear_database().await {
                    Ok(_) => {
                        // Reset all state to defaults
                        state.history.write().workflow_history.clear();
                        state.history.write().running_workflows.clear();
                        state.history.write().needs_history_reload = true;
                        state.workflow.write().reset_before_run();

                        // Reload settings (will load defaults and trigger app reload)
                        match s_e_e_core::get_global_store() {
                            Ok(store) => {
                                if let Ok(Some(settings)) = store.load_settings().await {
                                    state.settings.write().apply_loaded_settings(settings);
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to reload settings: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to clear database: {}", e);
                    }
                }
            });
        }
    };

    rsx! {
        div { class: "space-y-8",
            PageHeader {
                title: "Settings".to_string(),
                description: "Customize your application preferences".to_string(),
                actions: None,
            }

            div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 p-8 shadow-sm",
                h3 { class: "text-base font-semibold text-zinc-900 dark:text-white mb-6", "Settings" }
                p { class: "text-zinc-600 dark:text-zinc-400", "Settings functionality coming soon." }
            }

            ConfirmDialog {
                show: show_confirm_dialog(),
                title: "Clear All Data?".to_string(),
                message: "This will permanently delete all workflow history, settings, and prompts. This action cannot be undone.".to_string(),
                confirm_text: "Clear All Data".to_string(),
                cancel_text: "Cancel".to_string(),
                on_confirm: clear_database_handler,
                on_cancel: move |_| show_confirm_dialog.set(false)
            }

            div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 p-8 shadow-sm",
                h3 { class: "text-base font-semibold text-zinc-900 dark:text-white mb-4", "Data Management" }
                div { class: "space-y-4",
                    p { class: "text-zinc-600 dark:text-zinc-400",
                        "Clear all application data including workflow history, settings, and prompts. This action cannot be undone."
                    }
                    Button {
                        variant: ButtonVariant::Danger,
                        size: ButtonSize::Large,
                        onclick: move |_| show_confirm_dialog.set(true),
                        class: "w-full",
                        Icon {
                            name: "trash".to_string(),
                            class: Some("w-5 h-5 mr-2".to_string()),
                            size: None,
                            variant: Some("outline".to_string()),
                        }
                        "Clear All Data"
                    }
                }
            }
        }
    }
}
