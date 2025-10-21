use crate::components::Toast;
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

    // 5. Show database initialization error notification if needed
    let store_clone_for_notification = store.clone();
    use_effect(move || {
        if store_clone_for_notification.is_none() {
            state_provider.ui.write().show_toast(
                "⚠️ Database unavailable - workflow history and settings will not be saved"
                    .to_string(),
            );
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
                    match s.list_workflow_executions(50).await {
                        Ok(history) => {
                            history_state.write().set_history(history);
                        }
                        Err(e) => {
                            state_provider
                                .ui
                                .write()
                                .show_toast(format!("Failed to load history: {}", e));
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

            Toast {
                message: state_provider.ui.read().toast_message.clone(),
                on_dismiss: move |_| state_provider.ui.write().dismiss_toast()
            }

            Router::<Route> {}
        }
    }
}
