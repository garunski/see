use crate::components::{
    ConfirmDialog, IconButton, IconButtonSize, IconButtonVariant, Notification, NotificationData,
    NotificationType, PageHeader,
};
use crate::pages::settings::components::SettingsForm;
use crate::pages::settings::hooks::{use_settings_mutation, use_settings_query};
use crate::services::clear_database;
use crate::state::AppStateProvider;
use dioxus::prelude::*;
use s_e_e_core::AppSettings;

#[component]
pub fn SettingsPage() -> Element {
    let state_provider = use_context::<AppStateProvider>();

    let mut show_confirm_dialog = use_signal(|| false);
    let mut notification = use_signal(|| NotificationData {
        r#type: NotificationType::Success,
        title: String::new(),
        message: String::new(),
        show: false,
    });

    // Load settings via hook
    let settings_result = use_settings_query();
    let loaded_settings = match settings_result {
        Ok(s) => s,
        Err(e) => {
            return rsx! {
                div { class: "space-y-8",
                    PageHeader {
                        title: "Settings".to_string(),
                        description: "Customize your application preferences".to_string(),
                        actions: None,
                    }
                    div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 p-8 shadow-sm",
                        div { class: "text-red-600 dark:text-red-400",
                            "Failed to load settings: {e}"
                        }
                    }
                }
            };
        }
    };

    // Initialize mutations
    let mutations = use_settings_mutation();

    // Form state signals
    let mut theme = use_signal(|| loaded_settings.theme.clone());

    // Populate signals from loaded settings
    use_effect(move || {
        theme.set(loaded_settings.theme.clone());
    });

    // Helper to save settings
    let save_settings = {
        let notification = notification;
        let update_mutation = mutations.update_mutation;
        move |settings: AppSettings| {
            let mut notification = notification;
            spawn(async move {
                tracing::info!("[SettingsPage] Starting mutation to save settings");
                let reader = update_mutation.mutate_async(settings.clone()).await;
                tracing::debug!("[SettingsPage] Mutation completed, reading state");
                let state = reader.state();
                match state.unwrap() {
                    Ok(_) => {
                        tracing::info!("[SettingsPage] Settings saved successfully, showing success notification");
                        notification.set(NotificationData {
                            r#type: NotificationType::Success,
                            title: "Settings saved".to_string(),
                            message: "Your settings have been successfully saved.".to_string(),
                            show: true,
                        });
                    }
                    Err(e) => {
                        tracing::error!("[SettingsPage] Settings save failed: {}", e);
                        notification.set(NotificationData {
                            r#type: NotificationType::Error,
                            title: "Save failed".to_string(),
                            message: format!("Failed to save settings: {}", e),
                            show: true,
                        });
                    }
                }
            });
        }
    };

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

            Notification {
                notification,
                on_close: move |_| {
                    notification.set(NotificationData {
                        r#type: notification().r#type.clone(),
                        title: notification().title.clone(),
                        message: notification().message.clone(),
                        show: false,
                    });
                },
            }

            div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 p-8 shadow-sm",
                h3 { class: "text-base font-semibold text-zinc-900 dark:text-white mb-6", "Preferences" }
                SettingsForm {
                    theme,
                    on_theme_change: move |new_theme: s_e_e_core::Theme| {
                        tracing::info!("[SettingsPage] Theme changed to {:?}", new_theme);
                        theme.set(new_theme.clone());
                        let settings = AppSettings {
                            theme: new_theme,
                            auto_save: loaded_settings.auto_save,
                            notifications: loaded_settings.notifications,
                            default_workflow: loaded_settings.default_workflow.clone(),
                        };
                        tracing::debug!("[SettingsPage] Calling save_settings with theme: {:?}", settings.theme);
                        save_settings(settings);
                    },
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
                    IconButton {
                        variant: IconButtonVariant::Danger,
                        size: IconButtonSize::Large,
                        onclick: move |_| show_confirm_dialog.set(true),
                        class: Some("w-full".to_string()),
                        icon: Some("trash".to_string()),
                        icon_variant: "outline".to_string(),
                        "Clear All Data"
                    }
                }
            }
        }
    }
}
