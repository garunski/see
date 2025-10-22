use crate::router::Route;
use crate::state::AppStateProvider;
use dioxus::prelude::*;
use dioxus_desktop::use_window;
use see_core::{AppSettings, AuditStore, RedbStore};
use std::sync::Arc;

#[component]
pub fn App() -> Element {
    // 0. Configure window behavior - disable "Always on Top"
    let window = use_window();
    use_effect(move || {
        window.set_always_on_top(false);
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
    // 1. Initialize store and provide as context with proper error handling
    let store = use_hook(|| match RedbStore::new_default() {
        Ok(store) => Some(Arc::new(store)),
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            None
        }
    });
    use_context_provider(|| store.clone());

    // 2. Create state provider with separated state
    let mut state_provider = use_hook(AppStateProvider::new);
    use_context_provider(|| state_provider.clone());

    // 3. Load settings using use_resource for proper async handling
    let settings_loader = use_resource({
        let store = store.clone();
        move || {
            let store = store.clone();
            async move {
                if let Some(ref s) = store {
                    match s.load_settings().await {
                        Ok(Some(loaded)) => Ok(loaded),
                        Ok(None) => {
                            // No settings found, return default settings
                            Ok(AppSettings::default())
                        }
                        Err(e) => Err(format!("Failed to load settings: {}", e)),
                    }
                } else {
                    // No store available, return default settings
                    Ok(AppSettings::default())
                }
            }
        }
    });

    // 4. Apply loaded settings to state
    use_effect({
        let store = store.clone();
        move || {
            if let Some(Ok(settings)) = settings_loader.read().as_ref() {
                state_provider
                    .settings
                    .write()
                    .apply_loaded_settings(settings.clone());

                // Load and merge default workflows
                let default_workflows = see_core::WorkflowDefinition::get_default_workflows();
                let mut settings_guard = state_provider.settings.write();
                for default_workflow in default_workflows {
                    // Check if this default workflow already exists in settings
                    let exists = settings_guard
                        .settings
                        .workflows
                        .iter()
                        .any(|w| w.id == default_workflow.id);
                    if !exists {
                        // Add default workflow if it doesn't exist
                        settings_guard.add_workflow(default_workflow);
                    }
                }

                // Save updated settings with default workflows
                if let Some(ref s) = store {
                    let settings_to_save = settings_guard.settings.clone();
                    let store_clone = s.clone();
                    spawn(async move {
                        if let Err(e) = store_clone.save_settings(&settings_to_save).await {
                            eprintln!("Failed to save default workflows: {}", e);
                        }
                    });
                }
            }
        }
    });

    // 6. Load history - reactive to needs_history_reload flag
    let store_clone2 = store.clone();
    use_effect(move || {
        let needs_reload = state_provider.history.read().needs_history_reload;
        if needs_reload {
            let store = store_clone2.clone();
            let mut history_state = state_provider.history;
            spawn(async move {
                if let Some(s) = store {
                    // Load completed workflows
                    match s.list_workflow_executions(50).await {
                        Ok(history) => {
                            history_state.write().set_history(history);
                        }
                        Err(e) => {
                            eprintln!("Failed to load history: {}", e);
                        }
                    }

                    // Load running workflows
                    match s.list_workflow_metadata(50).await {
                        Ok(metadata) => {
                            // Filter to only running workflows
                            let running: Vec<_> = metadata
                                .into_iter()
                                .filter(|m| {
                                    m.status
                                        == see_core::persistence::models::WorkflowStatus::Running
                                })
                                .collect();
                            history_state.write().set_running_workflows(running);
                        }
                        Err(e) => {
                            eprintln!("Failed to load running workflows: {}", e);
                        }
                    }
                }
            });
        }
    });

    // 7. Monitor theme changes from settings
    let theme_signal = use_memo(move || state_provider.settings.read().settings.theme);

    // Apply theme changes reactively and save to database
    use_effect({
        let store = store.clone();
        move || {
            // Save theme changes to database whenever theme changes
            if let Some(ref s) = store {
                let settings_to_save = state_provider.settings.read().settings.clone();
                let store_clone = s.clone();
                spawn(async move {
                    if let Err(e) = store_clone.save_settings(&settings_to_save).await {
                        eprintln!("Failed to save theme settings: {}", e);
                    }
                });
            }
        }
    });

    // Poll for workflow progress during execution
    let polling_trigger = use_memo(move || state_provider.workflow.read().polling_trigger);
    let mut cancel_tx_ref = use_signal(|| None::<tokio::sync::oneshot::Sender<()>>);

    use_effect(move || {
        let _trigger = polling_trigger(); // Force dependency tracking
        let is_polling = state_provider.workflow.read().is_polling;

        // Cancel any existing polling loop
        if let Some(cancel_tx) = cancel_tx_ref.write().take() {
            let _ = cancel_tx.send(());
        }

        if is_polling {
            let polling_execution_id = state_provider.workflow.read().polling_execution_id.clone();
            let store = store.clone();
            let _ui_state = state_provider.ui;
            let mut workflow_state = state_provider.workflow;

            // Create a cancellation token for this polling loop
            let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel();

            // Store the cancel sender for cleanup
            *cancel_tx_ref.write() = Some(cancel_tx);

            spawn(async move {
                loop {
                    // Check for cancellation first
                    if cancel_rx.try_recv().is_ok() {
                        break;
                    }

                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                    // Check if we should stop polling
                    if !workflow_state.read().is_polling {
                        break;
                    }

                    if let Some(ref s) = store {
                        // Try to poll based on execution_id if we have it
                        let result = if let Some(ref exec_id) = polling_execution_id {
                            // Use specific execution_id
                            crate::services::workflow::poll_workflow_progress(exec_id, s.clone())
                                .await
                        } else {
                            continue;
                        };

                        match result {
                            Ok(progress) => {
                                let _message = if let Some(ref task) = progress.current_task {
                                    format!(
                                        "Running: {} ({}/{} complete)",
                                        task, progress.completed, progress.total
                                    )
                                } else if progress.is_complete {
                                    // Workflow completed, stop polling
                                    workflow_state.write().stop_polling();
                                    break;
                                } else if progress.total > 0 {
                                    format!(
                                        "Progress: {}/{} tasks complete",
                                        progress.completed, progress.total
                                    )
                                } else {
                                    "Starting workflow...".to_string()
                                };

                                // Status updates removed
                            }
                            Err(_) => {
                                // Workflow not found yet or error, continue polling
                            }
                        }
                    }
                }
            });
        }
    });

    rsx! {
        div {
            class: format!("min-h-screen bg-white dark:bg-zinc-900 text-zinc-950 dark:text-white {}",
                match theme_signal() {
                    see_core::Theme::Light => "light",
                    see_core::Theme::Dark => "dark",
                    see_core::Theme::System => {
                        if matches!(dark_light::detect(), dark_light::Mode::Dark) {
                            "dark"
                        } else {
                            "light"
                        }
                    }
                }
            ),
            onkeydown: move |evt| {
                match evt.key() {
                    dioxus::events::Key::ArrowLeft | dioxus::events::Key::ArrowUp => {
                        let current = state_provider.workflow.read().current_step;
                        if current > 0 {
                            state_provider.workflow.write().current_step = current - 1;
                        }
                    }
                    dioxus::events::Key::ArrowRight | dioxus::events::Key::ArrowDown => {
                        let current = state_provider.workflow.read().current_step;
                        let total = state_provider.workflow.read().tasks.len();
                        if current < total.saturating_sub(1) {
                            state_provider.workflow.write().current_step = current + 1;
                        }
                    }
                    _ => {}
                }
            },


            Router::<Route> {}
        }
    }
}
