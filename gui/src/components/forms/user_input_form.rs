use dioxus::prelude::*;
use s_e_e_core::TaskInfo;
use tracing::error;

#[component]
pub fn UserInputForm(task: TaskInfo, execution_id: Option<String>) -> Element {
    let mut input_value = use_signal(String::new);
    let mut is_submitting = use_signal(|| false);
    let mut error_message = use_signal(String::new);

    rsx! {
        div {
            class: "space-y-4",

            div {
                class: "space-y-2",
                label {
                    class: "block text-sm font-medium text-amber-900 dark:text-amber-100",
                    "Enter value"
                }

                input {
                    r#type: "text",
                    placeholder: "Type your input here...",
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
                class: "w-full px-4 py-2 bg-amber-600 hover:bg-amber-700 text-white rounded-md font-medium transition-colors inline-flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed",
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
                                        tracing::info!("Input provided successfully for task {}", task_id);
                                        // TODO: Refresh execution state
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
