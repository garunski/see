use crate::components::{
    Alert, AlertType, Button, ButtonSize, ButtonVariant, ConfirmDialog, Notification,
    NotificationData, NotificationType, PageHeader, SectionCard, TextInput, TextareaInput,
    ValidationMessage,
};
use crate::icons::Icon;
use crate::layout::router::Route;
use crate::services::prompt::UserPromptService;
use crate::state::AppStateProvider;
use dioxus::prelude::*;
use dioxus_router::prelude::{use_navigator, Link};
use s_e_e_core::clone_system_prompt as clone_prompt;
use s_e_e_core::UserPrompt;

#[component]
pub fn UserPromptEditPage(id: String) -> Element {
    let state_provider = use_context::<AppStateProvider>();
    let navigator = use_navigator();

    let is_new = id.is_empty();
    let mut is_system = use_signal(|| false);

    let mut prompt_id = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut content = use_signal(String::new);
    let mut validation_error = use_signal(String::new);
    let mut is_saving = use_signal(|| false);
    let mut show_delete_dialog = use_signal(|| false);
    let is_deleting = use_signal(|| false);
    let notification = use_signal(|| NotificationData {
        r#type: NotificationType::Success,
        title: String::new(),
        message: String::new(),
        show: false,
    });

    // Load existing prompt data if editing
    let prompt_id_for_effect = id.clone();
    use_effect(move || {
        if !is_new && !prompt_id_for_effect.is_empty() {
            // Check if it's a system prompt
            if prompt_id_for_effect.starts_with("system:") {
                is_system.set(true);
                // Try to load from system prompts
                if let Some(system_prompt) = state_provider
                    .prompts
                    .read()
                    .get_system_prompts()
                    .iter()
                    .find(|p| p.id == prompt_id_for_effect)
                {
                    prompt_id.set(system_prompt.id.clone());
                    description.set(system_prompt.description.clone().unwrap_or_default());
                    content.set(system_prompt.template.clone());
                }
            } else {
                // Load from user prompts
                if let Some(prompt) = state_provider
                    .prompts
                    .read()
                    .get_prompt(prompt_id_for_effect.clone())
                {
                    prompt_id.set(prompt.id.clone());
                    description.set(prompt.description.clone().unwrap_or_default());
                    content.set(prompt.template.clone());
                }
            }
        }
    });

    let show_notification = {
        let mut notification = notification;
        move |notification_type: NotificationType, title: String, message: String| {
            notification.set(NotificationData {
                r#type: notification_type,
                title,
                message,
                show: true,
            });
        }
    };

    let close_notification = {
        let mut notification = notification;
        move |_| {
            notification.set(NotificationData {
                r#type: notification().r#type.clone(),
                title: notification().title.clone(),
                message: notification().message.clone(),
                show: false,
            });
        }
    };

    let confirm_delete = {
        let state_provider = state_provider.clone();
        let mut show_dialog = show_delete_dialog;
        let mut deleting = is_deleting;
        let prompt_id = id.clone();
        move |_| {
            show_dialog.set(false);
            deleting.set(true);

            let mut state_provider = state_provider.clone();
            let prompt_id = prompt_id.clone();
            let mut show_notification = show_notification;
            spawn(async move {
                match UserPromptService::delete_prompt(&prompt_id).await {
                    Ok(_) => {
                        state_provider.prompts.write().remove_prompt(prompt_id);
                        show_notification(
                            NotificationType::Success,
                            "Prompt deleted".to_string(),
                            "The prompt has been successfully deleted.".to_string(),
                        );
                        // Navigate back to list page after successful deletion
                        navigator.push(Route::UserPromptsListPage {});
                    }
                    Err(e) => {
                        validation_error.set(format!("Failed to delete prompt: {}", e));
                        show_notification(
                            NotificationType::Error,
                            "Delete failed".to_string(),
                            format!("Failed to delete prompt: {}", e),
                        );
                    }
                }
                deleting.set(false);
            });
        }
    };

    rsx! {
        div { class: "space-y-8",
            PageHeader {
                title: if is_system() { "View System Prompt".to_string() } else if is_new { "Create prompt".to_string() } else { "Edit prompt".to_string() },
                description: if is_system() { "System prompts are read-only. Clone this prompt to create an editable copy.".to_string() } else if is_new { "Create a new prompt template".to_string() } else { "Edit prompt template".to_string() },
                actions: Some(rsx! {
                    div { class: "flex items-center gap-3",
                        Link {
                            to: Route::UserPromptsListPage {},
                            class: "inline-flex items-center gap-x-1.5 rounded-md bg-zinc-100 dark:bg-zinc-800 px-3 py-2 text-sm font-semibold text-zinc-900 dark:text-zinc-100 shadow-sm hover:bg-zinc-200 dark:hover:bg-zinc-700",
                            Icon {
                                name: "arrow_left".to_string(),
                                class: Some("-ml-0.5 h-4 w-4".to_string()),
                                size: None,
                                variant: Some("outline".to_string()),
                            }
                            "Back"
                        }
                        if !is_new && !is_system() {
                            Button {
                                variant: ButtonVariant::Danger,
                                size: ButtonSize::Medium,
                                disabled: Some(is_deleting()),
                                loading: Some(is_deleting()),
                                onclick: move |_| show_delete_dialog.set(true),
                                "Delete"
                            }
                        }
                        if is_system() {
                            Button {
                                variant: ButtonVariant::Primary,
                                size: ButtonSize::Medium,
                                onclick: move |_| {
                                    let prompt_id_for_clone = id.clone();
                                    let navigator = navigator;
                                    spawn(async move {
                                        match clone_prompt(&prompt_id_for_clone, None).await {
                                            Ok(cloned_prompt) => {
                                                // Navigate to the newly created prompt
                                                navigator.push(Route::UserPromptEditPage {
                                                    id: cloned_prompt.id
                                                });
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to clone prompt: {}", e);
                                            }
                                        }
                                    });
                                },
                                span { class: "inline-flex items-center gap-2",
                                    Icon {
                                        name: "copy".to_string(),
                                        class: None,
                                        size: Some("h-4 w-4".to_string()),
                                        variant: None,
                                    }
                                    span { "Clone to Edit" }
                                }
                            }
                        } else {
                            Button {
                                variant: ButtonVariant::Primary,
                                size: ButtonSize::Medium,
                                disabled: Some(is_saving()),
                                loading: Some(is_saving()),
                                onclick: move |_| {
                                // Validation
                                if prompt_id().trim().is_empty() {
                                    validation_error.set("ID is required".to_string());
                                    return;
                                }
                                if content().trim().is_empty() {
                                    validation_error.set("Content is required".to_string());
                                    return;
                                }

                                // Check for duplicate ID if creating new
                                if is_new {
                                    let prompts_guard = state_provider.prompts.read();
                                    let existing_prompt = prompts_guard.get_prompt(prompt_id().trim().to_string());
                                    if existing_prompt.is_some() {
                                        validation_error.set("A prompt with this ID already exists".to_string());
                                        return;
                                    }
                                }

                                validation_error.set(String::new());
                                is_saving.set(true);

                                let now = chrono::Utc::now();
                                let content_str = content().trim().to_string();
                                let prompt = UserPrompt {
                                    id: prompt_id().trim().to_string(),
                                    name: prompt_id().trim().to_string(),
                                    content: content_str.clone(),
                                    description: Some(description().trim().to_string()),
                                    template: content_str,
                                    variables: Vec::new(),
                                    tags: Vec::new(),
                                    metadata: serde_json::Value::Object(serde_json::Map::new()),
                                    created_at: now,
                                    updated_at: now,
                                };

                                let mut state_provider = state_provider.clone();
                                let id_for_save = id.clone();
                                let mut show_notification = show_notification;
                                spawn(async move {
                                    let result = if is_new {
                                        UserPromptService::create_prompt(prompt.clone()).await
                                    } else {
                                        UserPromptService::update_prompt(prompt.clone()).await
                                    };

                                    match result {
                                        Ok(_) => {
                                            if is_new {
                                                state_provider.prompts.write().add_prompt(prompt);
                                                show_notification(
                                                    NotificationType::Success,
                                                    "Prompt created".to_string(),
                                                    "Your new prompt has been successfully created.".to_string(),
                                                );
                                            } else {
                                                state_provider.prompts.write().update_prompt(
                                                    id_for_save.clone(),
                                                    prompt,
                                                );
                                                show_notification(
                                                    NotificationType::Success,
                                                    "Prompt saved".to_string(),
                                                    "Your changes have been successfully saved.".to_string(),
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            validation_error.set(format!("Failed to save prompt: {}", e));
                                            show_notification(
                                                NotificationType::Error,
                                                "Save failed".to_string(),
                                                format!("Failed to save prompt: {}", e),
                                            );
                                        }
                                    }
                                    is_saving.set(false);
                                });
                            },
                            if is_saving() { "Saving..." } else { "Save" }
                            }
                        }
                    }
                }),
            }

            SectionCard {
                title: None,
                children: rsx! {
                    if is_system() {
                        Alert {
                            alert_type: AlertType::Info,
                            title: Some("System Prompt".to_string()),
                            message: "This is a system prompt and cannot be edited. Click the Clone button above to create an editable copy.".to_string(),
                            dismissible: None,
                            on_dismiss: None,
                            actions: None,
                        }
                    }
                    div { class: "space-y-6",
                        TextInput {
                            label: "Prompt ID".to_string(),
                            value: prompt_id,
                            oninput: move |value| prompt_id.set(value),
                            placeholder: Some("e.g., generate-rust-code".to_string()),
                            help_text: Some("Human-readable identifier used to reference this prompt in workflows".to_string()),
                            required: Some(true),
                            disabled: Some(is_system()),
                        }

                        TextInput {
                            label: "Description".to_string(),
                            value: description,
                            oninput: move |value| description.set(value),
                            placeholder: Some("Brief description of what this prompt does".to_string()),
                            help_text: None,
                            required: Some(false),
                            disabled: Some(is_system()),
                        }

                        TextareaInput {
                            label: "Prompt Content".to_string(),
                            value: content,
                            oninput: move |value| content.set(value),
                            placeholder: Some("Enter the prompt template content...".to_string()),
                            help_text: Some("The actual prompt text that will be sent to the AI model".to_string()),
                            rows: Some(15),
                            disabled: Some(is_system()),
                        }

                        ValidationMessage {
                            message: validation_error,
                        }
                    }
                },
                padding: None,
            }
        }

        Notification {
            notification: notification,
            on_close: close_notification,
        }

        if !is_new && !is_system() {
            ConfirmDialog {
                show: show_delete_dialog(),
                title: "Delete UserPrompt?".to_string(),
                message: format!("Are you sure you want to delete the prompt '{}'? This action cannot be undone.", prompt_id()),
                confirm_text: "Delete".to_string(),
                cancel_text: "Cancel".to_string(),
                on_confirm: confirm_delete,
                on_cancel: move |_| show_delete_dialog.set(false)
            }
        }
    }
}

#[component]
pub fn UserPromptEditPageNew() -> Element {
    rsx! {
        UserPromptEditPage { id: "".to_string() }
    }
}
