use crate::components::{Notification, NotificationData, NotificationType};
use crate::layout::router::Route;
use crate::queries::GetPrompt;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_query, Query};
use dioxus_router::prelude::use_navigator;

use super::components::{PromptDeleteDialog, PromptFormFields, PromptFormHeader};
use super::hooks::{use_notification_state, use_prompt_form, use_prompt_mutations};
use super::validation::{create_prompt_from_fields, validate_prompt_fields};

#[component]
pub fn UserPromptEditPage(id: String) -> Element {
    let navigator = use_navigator();
    let is_new = id.is_empty();

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

    // Initialize state via hooks (NO CLOSURES RETURNED)
    let mut form_state = use_prompt_form(id.clone(), loaded_prompt);
    let mutations = use_prompt_mutations();
    let mut notification = use_notification_state();
    let mut show_delete_dialog = use_signal(|| false);

    // ALL EVENT HANDLERS INLINE - NO EXTRACTION
    rsx! {
        div { class: "space-y-8",
            PromptFormHeader {
                is_new,
                is_saving: *mutations.is_saving.read(),
                is_deleting: *mutations.is_deleting.read(),
                on_delete_click: move |_| {
                    show_delete_dialog.set(true);
                },
                on_save_click: move |_| {
                    // Validation
                    match validate_prompt_fields(&form_state.prompt_id.read(), &form_state.name.read(), &form_state.content.read()) {
                        Ok(_) => form_state.validation_error.set(String::new()),
                        Err(e) => {
                            form_state.validation_error.set(e);
                            return;
                        }
                    }

                    let prompt = create_prompt_from_fields(
                        form_state.prompt_id.read().to_string(),
                        form_state.name.read().to_string(),
                        form_state.content.read().to_string(),
                    );

                    let create_mutation = mutations.create_mutation;
                    let update_mutation = mutations.update_mutation;
                    let mut notification = notification;
                    let mut validation_error = form_state.validation_error;
                    let is_new = is_new;
                    let prompt_clone = prompt.clone();

                    spawn(async move {
                        if is_new {
                            let reader = create_mutation.mutate_async(prompt_clone.clone()).await;
                            let state = reader.state();
                            match state.unwrap() {
                                Ok(_) => {
                                    notification.set(NotificationData {
                                        r#type: NotificationType::Success,
                                        title: "Prompt created".to_string(),
                                        message: "Your new prompt has been successfully created.".to_string(),
                                        show: true,
                                    });
                                }
                                Err(e) => {
                                    let e = e.clone();
                                    validation_error.set(format!("Failed to save prompt: {}", e));
                                    notification.set(NotificationData {
                                        r#type: NotificationType::Error,
                                        title: "Save failed".to_string(),
                                        message: format!("Failed to save prompt: {}", e),
                                        show: true,
                                    });
                                }
                            }
                        } else {
                            let reader = update_mutation.mutate_async(prompt_clone.clone()).await;
                            let state = reader.state();
                            match state.unwrap() {
                                Ok(_) => {
                                    notification.set(NotificationData {
                                        r#type: NotificationType::Success,
                                        title: "Prompt saved".to_string(),
                                        message: "Your changes have been successfully saved.".to_string(),
                                        show: true,
                                    });
                                }
                                Err(e) => {
                                    let e = e.clone();
                                    validation_error.set(format!("Failed to save prompt: {}", e));
                                    notification.set(NotificationData {
                                        r#type: NotificationType::Error,
                                        title: "Save failed".to_string(),
                                        message: format!("Failed to save prompt: {}", e),
                                        show: true,
                                    });
                                }
                            }
                        }
                    });
                },
            }

            PromptFormFields {
                prompt_id: form_state.prompt_id,
                name: form_state.name,
                content: form_state.content,
                validation_error: form_state.validation_error,
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

            if !is_new {
                PromptDeleteDialog {
                    show: *show_delete_dialog.read(),
                    prompt_id: form_state.prompt_id.read().to_string(),
                    on_confirm: move |_| {
                        show_delete_dialog.set(false);

                        let mut notification = notification;
                        let mut validation_error = form_state.validation_error;
                        let prompt_id = id.clone();
                        let delete_mutation = mutations.delete_mutation;

                        spawn(async move {
                            let reader = delete_mutation.mutate_async(prompt_id.clone()).await;
                            let state = reader.state();
                            match state.unwrap() {
                                Ok(_) => {
                                    notification.set(NotificationData {
                                        r#type: NotificationType::Success,
                                        title: "Prompt deleted".to_string(),
                                        message: "The prompt has been successfully deleted.".to_string(),
                                        show: true,
                                    });
                                    navigator.push(Route::UserPromptsListPage {});
                                }
                                Err(e) => {
                                    let e = e.clone();
                                    validation_error.set(format!("Failed to delete prompt: {}", e));
                                    notification.set(NotificationData {
                                        r#type: NotificationType::Error,
                                        title: "Delete failed".to_string(),
                                        message: format!("Failed to delete prompt: {}", e),
                                        show: true,
                                    });
                                }
                            }
                        });
                    },
                    on_cancel: move |_| {
                        show_delete_dialog.set(false);
                    },
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
