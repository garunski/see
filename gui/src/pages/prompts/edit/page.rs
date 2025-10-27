use crate::components::{
    Button, ButtonSize, ButtonVariant, ConfirmDialog, Notification, NotificationData,
    NotificationType, PageHeader, SectionCard, TextInput, TextareaInput, ValidationMessage,
};
use crate::icons::Icon;
use crate::layout::router::Route;
use crate::queries::{CreatePromptMutation, DeletePromptMutation, GetPrompt, UpdatePromptMutation};
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, use_query, Mutation, Query};
use dioxus_router::prelude::{use_navigator, Link};
use s_e_e_core::Prompt;

#[component]
pub fn UserPromptEditPage(id: String) -> Element {
    let navigator = use_navigator();

    let is_new = id.is_empty();

    let mut prompt_id = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut content = use_signal(String::new);
    let mut validation_error = use_signal(String::new);
    let mut show_delete_dialog = use_signal(|| false);
    let notification = use_signal(|| NotificationData {
        r#type: NotificationType::Success,
        title: String::new(),
        message: String::new(),
        show: false,
    });

    let create_mutation = use_mutation(Mutation::new(CreatePromptMutation));
    let update_mutation = use_mutation(Mutation::new(UpdatePromptMutation));
    let delete_mutation = use_mutation(Mutation::new(DeletePromptMutation));

    // Get loading states from mutations
    let is_saving = use_memo(move || {
        create_mutation.read().state().is_loading() || update_mutation.read().state().is_loading()
    });
    let is_deleting = use_memo(move || delete_mutation.read().state().is_loading());

    // Load existing prompt data if editing
    let loaded_prompt = if !is_new {
        let query_result = use_query(Query::new(id.clone(), GetPrompt)).suspend()?;
        match query_result {
            Ok(Some(p)) => Some(p),
            Ok(None) => None,
            Err(_) => None,
        }
    } else {
        None
    };

    // Load prompt data into form fields only once
    let mut is_loaded = use_signal(|| false);
    use_effect(move || {
        if !is_loaded() && !is_new {
            if let Some(prompt) = &loaded_prompt {
                prompt_id.set(prompt.id.clone());
                description.set(prompt.description.clone().unwrap_or_default());
                content.set(prompt.template.clone());
                is_loaded.set(true);
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
        let mut show_dialog = show_delete_dialog;
        let show_notification = show_notification;
        let validation_error = validation_error;
        let prompt_id = id.clone();
        let delete_mutation = delete_mutation.clone();
        move |_| {
            show_dialog.set(false);

            let mut show_notification = show_notification.clone();
            let mut validation_error = validation_error.clone();
            let prompt_id = prompt_id.clone();
            let delete_mutation = delete_mutation.clone();

            spawn(async move {
                let reader = delete_mutation.mutate_async(prompt_id.clone()).await;
                let state = reader.state();
                match state.unwrap() {
                    Ok(_) => {
                        show_notification(
                            NotificationType::Success,
                            "Prompt deleted".to_string(),
                            "The prompt has been successfully deleted.".to_string(),
                        );
                        navigator.push(Route::UserPromptsListPage {});
                    }
                    Err(e) => {
                        let e = e.clone();
                        validation_error.set(format!("Failed to delete prompt: {}", e));
                        show_notification(
                            NotificationType::Error,
                            "Delete failed".to_string(),
                            format!("Failed to delete prompt: {}", e),
                        );
                    }
                }
            });
        }
    };

    rsx! {
            div { class: "space-y-8",
                PageHeader {
                    title: if is_new { "Create prompt".to_string() } else { "Edit prompt".to_string() },
                    description: if is_new { "Create a new prompt template".to_string() } else { "Edit prompt template".to_string() },
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
                            if !is_new {
                                Button {
                                    variant: ButtonVariant::Danger,
                                    size: ButtonSize::Medium,
                                    disabled: Some(is_deleting()),
                                    loading: Some(is_deleting()),
                                    onclick: move |_| show_delete_dialog.set(true),
                                    "Delete"
                                }
                            }
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

                                    validation_error.set(String::new());

                                    let now = chrono::Utc::now();
                                    let content_str = content().trim().to_string();
                                    let prompt = Prompt {
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

                                    let create_mutation = create_mutation.clone();
                                    let update_mutation = update_mutation.clone();
                                    let mut show_notification = show_notification.clone();
                                    let mut validation_error = validation_error.clone();
                                    let is_new = is_new;
                                    let prompt_clone = prompt.clone();
                                    spawn(async move {
                                        if is_new {
                                            let reader = create_mutation.mutate_async(prompt_clone.clone()).await;
                                            let state = reader.state();
                                            match state.unwrap() {
                                                Ok(_) => {
                                                    show_notification(
                                                        NotificationType::Success,
                                                        "Prompt created".to_string(),
                                                        "Your new prompt has been successfully created.".to_string(),
                                                    );
                                                }
                                                Err(e) => {
                                                    let e = e.clone();
                                                    validation_error.set(format!("Failed to save prompt: {}", e));
                                                    show_notification(
                                                        NotificationType::Error,
                                                        "Save failed".to_string(),
                                                        format!("Failed to save prompt: {}", e),
                                                    );
                                                }
                                            }
                                        } else {
                                            let reader = update_mutation.mutate_async(prompt_clone.clone()).await;
                                            let state = reader.state();
                                            match state.unwrap() {
                                                Ok(_) => {
                                                    show_notification(
                                                        NotificationType::Success,
                                                        "Prompt saved".to_string(),
                                                        "Your changes have been successfully saved.".to_string(),
                                                    );
                                                }
                                                Err(e) => {
                                                    let e = e.clone();
                                                    validation_error.set(format!("Failed to save prompt: {}", e));
                                                    show_notification(
                                                        NotificationType::Error,
                                                        "Save failed".to_string(),
                                                        format!("Failed to save prompt: {}", e),
                                                    );
                                                }
                                            }
                                        }
                                    });
                                },
                                if is_saving() { "Saving..." } else { "Save" }
                            }
                        }
                    }),
                }

                SectionCard {
                    title: None,
                    children: rsx! {
                        div { class: "space-y-6",
                            TextInput {
                                label: "Prompt ID".to_string(),
                                value: prompt_id,
                                oninput: move |value| prompt_id.set(value),
                                placeholder: Some("e.g., generate-rust-code".to_string()),
                                help_text: Some("Human-readable identifier used to reference this prompt in workflows".to_string()),
                                required: Some(true),
                                disabled: None,
                            }

                            TextInput {
                                label: "Description".to_string(),
                                value: description,
                                oninput: move |value| description.set(value),
                                placeholder: Some("Brief description of what this prompt does".to_string()),
                                help_text: None,
                                required: Some(false),
                                disabled: None,
                            }

                            TextareaInput {
                                label: "Prompt Content".to_string(),
                                value: content,
                                oninput: move |value| content.set(value),
                                placeholder: Some("Enter the prompt template content...".to_string()),
                                help_text: Some("The actual prompt text that will be sent to the AI model".to_string()),
                                rows: Some(15),
                                disabled: None,
                            }

                            ValidationMessage {
                                message: validation_error,
                            }
                        }
                    },
                    padding: None,
                }

                Notification {
                    notification: notification,
                    on_close: close_notification,
                }

                if !is_new {
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
}

#[component]
pub fn UserPromptEditPageNew() -> Element {
    rsx! {
        UserPromptEditPage { id: "".to_string() }
    }
}
