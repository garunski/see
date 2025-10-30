use crate::components::{Notification, NotificationData, NotificationType};
use crate::layout::router::Route;
use crate::queries::prompt_queries::use_prompt_query;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;

use super::components::{PromptDeleteDialog, PromptFormFields, PromptFormHeader};
use super::hooks::{use_notification_state, use_prompt_form, use_prompt_mutations};
use super::validation::{create_prompt_from_fields, validate_prompt_fields};

#[component]
pub fn UserPromptEditPage(id: String) -> Element {
    let navigator = use_navigator();
    let is_new = id.is_empty();

    let loaded_prompt = if !is_new {
        let (query_state, _refetch) = use_prompt_query(id.clone());

        if query_state.is_loading {
            return rsx! {
                div { class: "flex items-center justify-center h-64",
                    "Loading prompt..."
                }
            };
        }

        if query_state.is_error {
            let error_msg = query_state
                .error
                .clone()
                .unwrap_or_else(|| "Unknown error".to_string());
            return rsx! {
                div { class: "text-red-600 dark:text-red-400",
                    "Error loading prompt: {error_msg}"
                }
            };
        }

        query_state.data.and_then(|opt| opt)
    } else {
        None
    };

    let mut form_state = use_prompt_form(id.clone(), loaded_prompt);
    let mutations = use_prompt_mutations();
    let mut notification = use_notification_state();
    let mut show_delete_dialog = use_signal(|| false);

    use_effect(move || {
        let delete_state = mutations.delete_state.read();
        if delete_state.is_success {
            navigator.push(Route::UserPromptsListPage {});
        }
    });

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



                    if is_new {
                        (mutations.create_fn)(prompt);
                    } else {
                        (mutations.update_fn)(prompt);
                    }


                    notification.set(NotificationData {
                        r#type: NotificationType::Success,
                        title: "Saving...".to_string(),
                        message: "Your prompt is being saved.".to_string(),
                        show: true,
                    });
                },
            }

            PromptFormFields {
                prompt_id: form_state.prompt_id,
                name: form_state.name,
                content: form_state.content,
                validation_error: form_state.validation_error,
                is_new,
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


                        (mutations.delete_fn)(id.clone());


                        notification.set(NotificationData {
                            r#type: NotificationType::Success,
                            title: "Deleting...".to_string(),
                            message: "The prompt is being deleted.".to_string(),
                            show: true,
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
