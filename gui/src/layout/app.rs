use super::router::Route;
use crate::state::AppStateProvider;
use dioxus::prelude::*;
use dioxus_desktop::use_window;
use s_e_e_core::AppSettings;

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
    let mut state_provider = use_hook(AppStateProvider::new);
    use_context_provider(|| state_provider.clone());

    let settings_loader = use_resource(|| async move {
        match s_e_e_core::get_global_store() {
            Ok(store) => match store.load_settings().await {
                Ok(Some(loaded)) => Ok(loaded),
                Ok(None) => Ok(AppSettings::default()),
                Err(e) => Err(format!("Failed to load settings: {}", e)),
            },
            Err(e) => {
                eprintln!("Failed to get global store for settings: {}", e);
                Ok(AppSettings::default())
            }
        }
    });

    use_effect(move || {
        if let Some(Ok(settings)) = settings_loader.read().as_ref() {
            state_provider
                .settings
                .write()
                .apply_loaded_settings(settings.clone());

            // Load workflows from database
            let mut settings_state = state_provider.settings;
            spawn(async move {
                match s_e_e_core::get_global_store() {
                    Ok(store) => {
                        // Load workflows from database
                        match store.list_workflows().await {
                            Ok(workflows) => {
                                tracing::debug!(
                                    "Loaded {} workflows from database",
                                    workflows.len()
                                );

                                // Add workflows to settings state
                                settings_state.write().workflows = workflows;
                            }
                            Err(e) => {
                                eprintln!("Failed to load workflows from database: {}", e);
                            }
                        }

                        // Save settings to ensure consistency
                        let settings_to_save = settings_state.read().settings.clone();
                        if let Err(e) = store.save_settings(&settings_to_save).await {
                            eprintln!("Failed to save settings: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to get global store for loading workflows: {}", e);
                    }
                }
            });

            // Load system templates from database (ONE-TIME, no infinite loop)
            let mut settings_state = state_provider.settings;
            let mut prompt_state = state_provider.prompts;
            spawn(async move {
                match s_e_e_core::get_global_store() {
                    Ok(store) => {
                        // Load system workflows
                        if let Ok(system_workflows) = store.list_system_workflows().await {
                            tracing::debug!("Loaded {} system workflows", system_workflows.len());
                            settings_state
                                .write()
                                .set_system_workflows(system_workflows);
                        }

                        // Load system prompts
                        if let Ok(system_prompts) = store.list_system_prompts().await {
                            tracing::debug!("Loaded {} system prompts", system_prompts.len());
                            prompt_state.write().set_system_prompts(system_prompts);
                            prompt_state.write().needs_reload = false;
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "Failed to get global store for loading system templates: {}",
                            e
                        );
                    }
                }
            });
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
                                .map(|exec| {
                                    // Check if any tasks are waiting for input
                                    let has_pending_inputs = exec.tasks.iter().any(|t| {
                                        t.status.as_str() == "waiting_for_input"
                                            || t.status.as_str() == "WaitingForInput"
                                    });

                                    // Adjust success field: if we have pending inputs, success should be None (not Failed)
                                    let adjusted_success = if has_pending_inputs {
                                        None // Don't mark as failed when waiting for input
                                    } else {
                                        exec.success
                                    };

                                    s_e_e_core::WorkflowExecutionSummary {
                                        id: exec.id,
                                        workflow_name: exec.workflow_name,
                                        status: exec.status,
                                        created_at: exec.created_at,
                                        completed_at: exec.completed_at,
                                        success: adjusted_success,
                                        task_count: exec.tasks.len(),
                                        timestamp: exec.timestamp,
                                    }
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

    let theme_signal = use_memo(move || state_provider.settings.read().settings.theme.clone());

    use_effect(move || {
        let settings_to_save = state_provider.settings.read().settings.clone();
        spawn(async move {
            match s_e_e_core::get_global_store() {
                Ok(store) => {
                    if let Err(e) = store.save_settings(&settings_to_save).await {
                        eprintln!("Failed to save theme settings: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get global store for saving theme: {}", e);
                }
            }
        });
    });

    rsx! {
        div {
            class: format!("min-h-screen bg-white dark:bg-zinc-900 text-zinc-950 dark:text-white {}",
                match theme_signal() {
                    s_e_e_core::Theme::Light => "light",
                    s_e_e_core::Theme::Dark => "dark",
                    s_e_e_core::Theme::System => {
                        if matches!(dark_light::detect(), dark_light::Mode::Dark) {
                            "dark"
                        } else {
                            "light"
                        }
                    }
                }
            ),


            Router::<Route> {}
        }
    }
}
