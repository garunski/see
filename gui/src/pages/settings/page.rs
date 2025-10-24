use crate::components::{Button, ButtonSize, ButtonVariant, ConfirmDialog};
use crate::icons::Icon;
use crate::services::clear_database;
use crate::state::AppStateProvider;
use dioxus::prelude::*;
use see_core::Theme;

#[component]
pub fn SettingsPage() -> Element {
    let state_provider = use_context::<AppStateProvider>();

    let current_theme = use_memo(move || state_provider.settings.read().settings.theme);

    let change_theme = {
        let mut state_provider = state_provider.clone();
        move |new_theme: Theme| {
            state_provider.settings.write().change_theme(new_theme);

            let _ui_state = state_provider.ui;
            spawn(async move {
                match see_core::get_global_store() {
                    Ok(store) => {
                        match store
                            .save_settings(&see_core::AppSettings {
                                theme: new_theme,
                                workflows: state_provider
                                    .settings
                                    .read()
                                    .settings
                                    .workflows
                                    .clone(),
                            })
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
    };

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
                        match see_core::get_global_store() {
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
            div {
                h1 { class: "text-xl font-bold text-zinc-900 dark:text-white", "Settings" }
                p { class: "mt-2 text-zinc-600 dark:text-zinc-400", "Customize your application preferences" }
            }

            div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 p-8 shadow-sm",
                h3 { class: "text-base font-semibold text-zinc-900 dark:text-white mb-6", "Appearance" }
                div { class: "space-y-4",
                    for theme in [Theme::Light, Theme::Dark, Theme::System] {
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::Large,
                            onclick: {
                                let mut change_theme = change_theme;
                                move |_| change_theme(theme)
                            },
                            class: format!(
                                "w-full flex items-center justify-between px-6 py-4 rounded-xl border-2 transition-colors {}",
                                if current_theme() == theme {
                                    "border-blue-500 bg-blue-50 dark:bg-blue-900/20"
                                } else {
                                    "border-zinc-200 dark:border-zinc-700 hover:border-zinc-300 dark:hover:border-zinc-600 hover:bg-zinc-50 dark:hover:bg-zinc-700"
                                }
                            ),
                            div { class: "flex items-center gap-4",
                                div { class: "text-3xl",
                                    match theme {
                                        Theme::Light => "☀️",
                                        Theme::Dark => "🌙",
                                        Theme::System => "💻",
                                    }
                                }
                                div { class: "text-left",
                                    div { class: "font-semibold text-zinc-900 dark:text-white text-sm",
                                        match theme {
                                            Theme::Light => "Light",
                                            Theme::Dark => "Dark",
                                            Theme::System => "System",
                                        }
                                    }
                                    div { class: "text-sm text-zinc-500 dark:text-zinc-400 mt-1",
                                        match theme {
                                            Theme::Light => "Always use light theme",
                                            Theme::Dark => "Always use dark theme",
                                            Theme::System => "Match system preference",
                                        }
                                    }
                                }
                            }

                            if current_theme() == theme {
                                div { class: "text-blue-500 text-xl font-bold", "✓" }
                            }
                        }
                    }
                }
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
