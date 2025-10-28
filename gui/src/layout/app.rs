use super::hooks::use_theme;
use super::router::Route;
use crate::state::AppStateProvider;
use dioxus::prelude::*;
use dioxus_desktop::use_window;

#[component]
pub fn App() -> Element {
    let window = use_window();
    use_effect(move || {
        window.set_always_on_top(false);
        window.set_focus();
    });

    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }
        div {
            class: "min-h-screen bg-white dark:bg-zinc-900 text-zinc-950 dark:text-white",
            ErrorBoundary {
                handle_error: |error: ErrorContext| rsx! {
                    div { class: "flex items-center justify-center min-h-screen",
                        div { class: "text-center p-8",
                            h1 { class: "text-2xl font-bold text-red-600 dark:text-red-400 mb-4", "Application Error" }
                            p { class: "text-zinc-600 dark:text-zinc-400 mb-4", "An error occurred while initializing the application." }
                            pre { class: "text-sm text-zinc-500 dark:text-zinc-500 bg-zinc-100 dark:bg-zinc-800 p-4 rounded", "{error:#?}" }
                        }
                    }
                },
                SuspenseBoundary {
                    fallback: move |_| rsx! {
                        div { class: "flex items-center justify-center min-h-screen",
                            div { class: "text-center",
                                div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4" }
                                p { class: "text-zinc-600 dark:text-zinc-400", "Loading application..." }
                            }
                        }
                    },
                    AppContent {}
                }
            }
        }
    }
}

#[component]
fn AppContent() -> Element {
    let state_provider = use_hook(AppStateProvider::new);
    use_context_provider(|| state_provider.clone());

    // Run initial data population if needed (ONE-TIME)
    spawn(async move {
        if let Err(e) = s_e_e_core::populate_initial_data().await {
            eprintln!("Failed to populate initial data: {}", e);
        }
    });

    use_effect(move || {
        let needs_reload = state_provider.history.read().needs_history_reload;
        if needs_reload {
            let mut history_state = state_provider.history;
            spawn(async move {
                match s_e_e_core::get_global_store() {
                    Ok(store) => match store.list_workflow_executions().await {
                        Ok(history) => {
                            tracing::debug!(
                                "Loaded {} execution records from database",
                                history.len()
                            );
                            // Convert WorkflowExecution to WorkflowExecutionSummary
                            let summaries = history
                                .into_iter()
                                .map(|exec| s_e_e_core::WorkflowExecutionSummary {
                                    id: exec.id,
                                    workflow_name: exec.workflow_name,
                                    status: exec.status,
                                    created_at: exec.created_at,
                                    completed_at: exec.completed_at,
                                    task_count: exec.tasks.len(),
                                    timestamp: exec.timestamp,
                                })
                                .collect();
                            history_state.write().set_history(summaries);
                        }
                        Err(e) => {
                            eprintln!("Failed to load history: {}", e);
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to get global store for loading history: {}", e);
                    }
                }
            });
        }
    });

    let theme = use_theme();

    let theme_class = use_memo(move || {
        let theme_value = theme();
        tracing::debug!("[AppContent] use_theme returned: {:?}", theme_value);
        let class = match theme_value {
            s_e_e_core::Theme::Light => {
                tracing::debug!("[AppContent] Theme is Light, applying light class");
                "light"
            }
            s_e_e_core::Theme::Dark => {
                tracing::debug!("[AppContent] Theme is Dark, applying dark class");
                "dark"
            }
            s_e_e_core::Theme::System => {
                let detected = if matches!(dark_light::detect(), dark_light::Mode::Dark) {
                    tracing::debug!("[AppContent] Theme is System, detected dark mode");
                    "dark"
                } else {
                    tracing::debug!("[AppContent] Theme is System, detected light mode");
                    "light"
                };
                detected
            }
        };
        tracing::trace!("[AppContent] Theme class: {}", class);
        class
    });

    rsx! {
        div {
            class: format!("min-h-screen bg-white dark:bg-zinc-900 text-zinc-950 dark:text-white {}", theme_class()),


            Router::<Route> {}
        }
    }
}
