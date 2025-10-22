use crate::router::Route;
use crate::state::AppStateProvider;
use dioxus::prelude::*;
use dioxus_desktop::use_window;
use see_core::AppSettings;

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
        match see_core::get_global_store() {
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

            let default_workflows = see_core::WorkflowDefinition::get_default_workflows();
            let mut settings_guard = state_provider.settings.write();
            for default_workflow in default_workflows {
                let exists = settings_guard
                    .settings
                    .workflows
                    .iter()
                    .any(|w| w.id == default_workflow.id);
                if !exists {
                    settings_guard.add_workflow(default_workflow);
                }
            }

            let settings_to_save = settings_guard.settings.clone();
            spawn(async move {
                match see_core::get_global_store() {
                    Ok(store) => {
                        if let Err(e) = store.save_settings(&settings_to_save).await {
                            eprintln!("Failed to save default workflows: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to get global store for saving settings: {}", e);
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
                match see_core::get_global_store() {
                    Ok(store) => match store.list_workflow_executions(50).await {
                        Ok(history) => {
                            history_state.write().set_history(history);
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

    let theme_signal = use_memo(move || state_provider.settings.read().settings.theme);

    use_effect(move || {
        let settings_to_save = state_provider.settings.read().settings.clone();
        spawn(async move {
            match see_core::get_global_store() {
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


            Router::<Route> {}
        }
    }
}
