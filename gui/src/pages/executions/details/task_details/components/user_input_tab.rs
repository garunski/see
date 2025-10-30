use crate::components::{
    Badge, BadgeColor, IconButton, IconButtonSize, IconButtonVariant, SectionCard, TextInput,
};
use crate::layout::router::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;

#[component]
pub fn TaskDetailsUserInputTab(input_request: Option<s_e_e_core::UserInputRequest>) -> Element {
    let navigator = use_navigator();

    let mut input_value = use_signal(|| {
        // Initialize with default value if available
        if let Some(ref req) = input_request {
            if let Some(ref default) = req.default_value {
                if let Some(default_str) = default.as_str() {
                    return default_str.to_string();
                }
            }
        }
        String::new()
    });
    let mut error_message = use_signal(|| None::<String>);
    let mut is_submitting = use_signal(|| false);
    let is_submitted = use_signal(|| false);

    if let Some(req) = input_request {
        let is_pending = matches!(req.status.to_string().as_str(), "pending");

        if !is_pending || is_submitted() {
            rsx! {
                SectionCard {
                    title: Some("User Input Prompt".to_string()),
                    children: rsx! {
                        div { class: "space-y-4",
                            if !req.prompt_text.is_empty() {
                                div { class: "space-y-2",
                                    span { class: "text-sm font-medium text-zinc-600 dark:text-zinc-400", "Prompt:" }
                                    div { class: "text-sm text-zinc-900 dark:text-zinc-100 bg-white dark:bg-zinc-900 rounded-lg p-3 border border-zinc-200 dark:border-zinc-700",
                                        "{req.prompt_text}"
                                    }
                                }
                            }
                            div { class: "space-y-2",
                                span { class: "text-sm font-medium text-zinc-600 dark:text-zinc-400", "Input Type:" }
                                div { class: "text-sm text-zinc-900 dark:text-zinc-100",
                                    "{req.input_type:?}"
                                    if !req.required {
                                        span { class: "text-zinc-500 dark:text-zinc-400 ml-2", "(optional)" }
                                    } else {
                                        span { class: "text-red-600 dark:text-red-400 ml-2", "(required)" }
                                    }
                                }
                            }
                            if let Some(default) = &req.default_value {
                                div { class: "space-y-2",
                                    span { class: "text-sm font-medium text-zinc-600 dark:text-zinc-400", "Default Value:" }
                                    div { class: "text-sm text-zinc-900 dark:text-zinc-100 bg-white dark:bg-zinc-900 rounded-lg p-3 border border-zinc-200 dark:border-zinc-700 font-mono",
                                        "{default}"
                                    }
                                }
                            }
                            div { class: "space-y-2",
                                span { class: "text-sm font-medium text-zinc-600 dark:text-zinc-400", "Status:" }
                                Badge {
                                    color: if req.status.to_string() == "pending" {
                                        BadgeColor::Amber
                                    } else {
                                        BadgeColor::Emerald
                                    },
                                    class: None,
                                    {format!("{}", req.status)}
                                }
                            }
                            if is_submitted() {
                                div { class: "bg-emerald-50 dark:bg-emerald-900/20 border border-emerald-200 dark:border-emerald-800 rounded-lg p-4 mt-4",
                                    div { class: "flex items-center gap-2",
                                        span { class: "text-emerald-700 dark:text-emerald-300", "✓ Input submitted successfully" }
                                    }
                                }
                            }
                        }
                    },
                    padding: None,
                }
            }
        } else {
            // Active form for pending requests
            let task_id = req.task_execution_id.clone();
            let execution_id = req.workflow_execution_id.clone();

            rsx! {
                SectionCard {
                    title: Some("User Input Prompt".to_string()),
                    children: rsx! {
                        div { class: "space-y-4",
                            if !req.prompt_text.is_empty() {
                                div { class: "space-y-2",
                                    span { class: "text-sm font-medium text-zinc-600 dark:text-zinc-400", "Prompt:" }
                                    div { class: "text-sm text-zinc-900 dark:text-zinc-100 bg-white dark:bg-zinc-900 rounded-lg p-3 border border-zinc-200 dark:border-zinc-700",
                                        "{req.prompt_text}"
                                    }
                                }
                            }

                            form {
                                onsubmit: move |evt| {
                                    evt.prevent_default();

                                    // Validate input
                                    if input_value().trim().is_empty() && req.required {
                                        error_message.set(Some("This field is required".to_string()));
                                        return;
                                    }

                                    error_message.set(None);
                                    is_submitting.set(true);

                                    let value_to_submit = input_value();
                                    let task_id_spawn = task_id.clone();
                                    let execution_id_spawn = execution_id.clone();
                                    let mut is_submitting_spawn = is_submitting;
                                    let mut error_message_spawn = error_message;
                                    let mut is_submitted_spawn = is_submitted;
                                    let nav = navigator;

                                    spawn(async move {
                                        match s_e_e_core::provide_user_input(
                                            &execution_id_spawn,
                                            &task_id_spawn,
                                            value_to_submit,
                                        )
                                        .await
                                        {
                                            Ok(_) => {
                                                is_submitted_spawn.set(true);
                                                error_message_spawn.set(None);

                                                // Redirect back to execution details page after success
                                                nav.push(Route::WorkflowDetailsPage {
                                                    id: execution_id_spawn
                                                });
                                            }
                                            Err(e) => {
                                                error_message_spawn.set(Some(format!("Failed to submit input: {}", e)));
                                            }
                                        }
                                        is_submitting_spawn.set(false);
                                    });
                                },
                                div { class: "space-y-4",
                                    if let Some(error_msg) = error_message() {
                                        div { class: "bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-3",
                                            p { class: "text-sm text-red-700 dark:text-red-300",
                                                {error_msg}
                                            }
                                        }
                                    }

                                    TextInput {
                                        label: "Your Input".to_string(),
                                        value: input_value,
                                        oninput: EventHandler::new(move |value| {
                                            input_value.set(value);
                                            error_message.set(None);
                                        }),
                                        placeholder: match format!("{:?}", req.input_type).as_str() {
                                            "Number" => Some("Enter a number".to_string()),
                                            "Boolean" => Some("true or false".to_string()),
                                            _ => Some("Enter text".to_string()),
                                        },
                                        help_text: if let Some(default) = req.default_value.as_ref() {
                                            Some(format!("Default: {}", default))
                                        } else {
                                            None
                                        },
                                        required: Some(req.required),
                                        disabled: Some(is_submitting()),
                                    }

                                    div { class: "flex justify-end gap-3 pt-4",
                                        IconButton {
                                            variant: IconButtonVariant::Primary,
                                            size: IconButtonSize::Medium,
                                            onclick: None,
                                            icon: if is_submitting() {
                                                Some("stop".to_string())
                                            } else {
                                                None
                                            },
                                            class: None,
                                            disabled: Some(is_submitting()),
                                            if is_submitting() {
                                                "Submitting..."
                                            } else {
                                                "Submit Input"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    padding: None,
                }
            }
        }
    } else {
        rsx! {
            SectionCard {
                title: Some("User Input Prompt".to_string()),
                children: rsx! {
                    div { class: "text-center text-zinc-500 dark:text-zinc-400 py-8",
                        "No user input requested for this task"
                    }
                },
                padding: None,
            }
        }
    }
}
