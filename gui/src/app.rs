use crate::components::Toast;
use crate::router::Route;
use crate::state::AppStateProvider;
use dioxus::prelude::*;
use see_core::{AuditStore, RedbStore, Theme};
use std::sync::Arc;

#[component]
pub fn App() -> Element {
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

    // 3. Show database initialization error notification if needed
    let store_clone_for_notification = store.clone();
    use_effect(move || {
        if store_clone_for_notification.is_none() {
            state_provider.ui.write().show_toast(
                "⚠️ Database unavailable - workflow history and settings will not be saved"
                    .to_string(),
            );
        }
    });

    // 3. Load settings once on mount
    let store_clone = store.clone();
    use_effect(move || {
        let store = store_clone.clone();
        let mut settings = state_provider.settings;
        spawn(async move {
            if let Some(ref s) = store {
                if let Ok(Some(loaded)) = s.load_settings().await {
                    settings.write().apply_loaded_settings(loaded);
                }
            }
        });
    });

    // 4. Load history - reactive to needs_history_reload flag
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

    // 5. Compute dark mode from settings (for UI classes)
    let dark_mode_signal = use_memo(
        move || match state_provider.settings.read().settings.theme {
            Theme::Dark => true,
            Theme::Light => false,
            Theme::System => matches!(dark_light::detect(), dark_light::Mode::Dark),
        },
    );

    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }
        div {
            class: format!("min-h-screen bg-white dark:bg-zinc-900 text-zinc-950 dark:text-white {}", if *dark_mode_signal.read() { "dark" } else { "" }),
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
