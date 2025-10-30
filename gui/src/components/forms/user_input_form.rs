use dioxus::prelude::*;
use s_e_e_core::{TaskInfo, UserInputRequest};
use tracing::error;

#[component]
pub fn UserInputForm(
    task: TaskInfo,
    execution_id: Option<String>,
    input_request: Option<UserInputRequest>,
) -> Element {
    let mut input_value = use_signal(String::new);
    let mut is_submitting = use_signal(|| false);
    let mut error_message = use_signal(String::new);
    let mut needs_refresh = use_signal(|| false);

    let label_text = input_request
        .as_ref()
        .map(|req| req.prompt_text.clone())
        .unwrap_or_else(|| "Enter value".to_string());

    let placeholder_text = input_request
        .as_ref()
        .and_then(|req| req.default_value.as_ref())
        .and_then(|v| v.as_str())
        .map(|s| format!("Default: {}", s))
        .unwrap_or_else(|| "Type your input here...".to_string());

    let input_type_str = input_request
        .as_ref()
        .map(|req| match req.input_type.to_string().as_str() {
            "number" => "number",
            "boolean" => "checkbox",
            _ => "text",
        })
        .unwrap_or("text");

    let is_required = input_request
        .as_ref()
        .map(|req| req.required)
        .unwrap_or(true);

    rsx! {
        div {
            class: "space-y-4",


            if let Some(req) = input_request.as_ref() {
                div {
                    class: "text-sm text-amber-700 dark:text-amber-300 space-y-1",
                    if !req.prompt_text.is_empty() {
                        div { "Prompt: {req.prompt_text}" }
                    }
                    div {
                        "Type: {req.input_type}"
                        if !req.required {
                            " (optional)"
                        }
                    }
                    if let Some(default) = &req.default_value {
                        div { "Default: {default}" }
                    }
                }
            }

            div {
                class: "space-y-2",
                label {
                    class: "block text-sm font-medium text-amber-900 dark:text-amber-100",
                    "{label_text}"
                    if is_required {
                        span { class: "text-red-600 dark:text-red-400 ml-1", "*" }
                    }
                }

                input {
                    r#type: input_type_str,
                    placeholder: "{placeholder_text}",
                    class: "w-full px-3 py-2 border border-amber-300 dark:border-amber-700 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100",
                    value: "{input_value()}",
                    oninput: move |e| input_value.set(e.value()),
                }

                if !error_message().is_empty() {
                    p {
                        class: "text-red-600 dark:text-red-400 text-sm",
                        "{error_message}"
                    }
                }
            }

            button {
                class: "w-full px-4 py-2 bg-amber-600 hover:bg-amber-700 text-white rounded-md font-medium transition-colors inline-flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer",
                disabled: is_submitting() || input_value().is_empty(),
                onclick: move |_| {
                    let input = input_value();
                    let exec_id = execution_id.clone();
                    let task_id = task.id.clone();

                    if input.is_empty() {
                        error_message.set("Input cannot be empty".to_string());
                        return;
                    }

                    is_submitting.set(true);

                    spawn(async move {
                        match exec_id.as_ref() {
                            Some(id) => {
                                match s_e_e_core::provide_user_input(id, &task_id, input).await {
                                    Ok(_) => {
                                        tracing::debug!("Input provided successfully for task {}", task_id);
                                        needs_refresh.set(true);

                                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                                    }
                                    Err(e) => {
                                        error!("Failed to provide input: {}", e);
                                        error_message.set(format!("Failed to provide input: {}", e));
                                    }
                                }
                            }
                            None => {
                                error!("No execution ID provided");
                                error_message.set("Cannot submit: missing execution ID".to_string());
                            }
                        }
                        is_submitting.set(false);
                    });
                },
                if is_submitting() {
                    "Submitting..."
                } else {
                    "Submit Input"
                }
            }
        }
    }
}
