use crate::router::Route;
use crate::state::AppStateProvider;
use dioxus::prelude::*;
use dioxus_router::prelude::Link;
use see_core::Theme;
use std::sync::Arc;

pub mod components;

#[component]
pub fn SettingsPage() -> Element {
    let state_provider = use_context::<AppStateProvider>();
    let store = use_context::<Option<Arc<see_core::RedbStore>>>();

    let current_theme = use_memo(move || state_provider.settings.read().settings.theme);

    let store_clone = store.clone();
    let change_theme = {
        let mut state_provider = state_provider.clone();
        move |new_theme: Theme| {
            state_provider.settings.write().change_theme(new_theme);

            // Save immediately with error handling
            let store_clone = store_clone.clone();
            let mut ui_state = state_provider.ui;
            spawn(async move {
                if let Some(ref s) = store_clone {
                    match s
                        .save_settings(&see_core::AppSettings {
                            theme: new_theme,
                            workflows: state_provider.settings.read().settings.workflows.clone(),
                        })
                        .await
                    {
                        Ok(_) => {
                            ui_state
                                .write()
                                .show_toast("Settings saved successfully".to_string());
                        }
                        Err(e) => {
                            ui_state
                                .write()
                                .show_toast(format!("Failed to save settings: {}", e));
                        }
                    }
                }
            });
        }
    };

    rsx! {
        div { class: "space-y-8",
            // Header
            div {
                h1 { class: "text-xl font-bold text-zinc-900 dark:text-white", "Settings" }
                p { class: "mt-2 text-zinc-600 dark:text-zinc-400", "Customize your application preferences" }
            }

            // Theme Section
            div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 p-8 shadow-sm",
                h3 { class: "text-base font-semibold text-zinc-900 dark:text-white mb-6", "Appearance" }
                div { class: "space-y-4",
                    for theme in [Theme::Light, Theme::Dark, Theme::System] {
                        button {
                            class: format!(
                                "w-full flex items-center justify-between px-6 py-4 rounded-xl border-2 transition-colors {}",
                                if current_theme() == theme {
                                    "border-blue-500 bg-blue-50 dark:bg-blue-900/20"
                                } else {
                                    "border-zinc-200 dark:border-zinc-700 hover:border-zinc-300 dark:hover:border-zinc-600 hover:bg-zinc-50 dark:hover:bg-zinc-700"
                                }
                            ),
                            onclick: {
                                let mut change_theme = change_theme.clone();
                                move |_| change_theme(theme)
                            },

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

            // Workflow Settings Section
            div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 p-8 shadow-sm",
                h3 { class: "text-base font-semibold text-zinc-900 dark:text-white mb-4", "Workflow Management" }
                div { class: "space-y-4",
                    p { class: "text-zinc-600 dark:text-zinc-400",
                        "Create, edit, and manage your workflow definitions. Default workflows are provided to get you started."
                    }
                    Link {
                        to: Route::WorkflowsListPage {},
                        class: "inline-flex items-center gap-x-1.5 rounded-md bg-blue-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-blue-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600",
                        // Cog icon
                        svg { class: "-ml-0.5 h-4 w-4", view_box: "0 0 20 20", fill: "currentColor",
                            path { d: "M10 2a8 8 0 100 16 8 8 0 000-16zM8.5 10a1.5 1.5 0 113 0 1.5 1.5 0 01-3 0zM10 6a4 4 0 100 8 4 4 0 000-8z" }
                        }
                        "Manage Workflows"
                    }
                }
            }
        }
    }
}
