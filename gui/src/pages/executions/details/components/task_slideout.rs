use crate::components::Slideout;
use crate::queries::use_task_details_query;
use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct TaskSlideoutProps {
    pub is_open: bool,
    pub task_id: String,
    pub execution_id: String,
    pub on_close: EventHandler<()>,
}

#[component]
pub fn TaskSlideout(props: TaskSlideoutProps) -> Element {
    let TaskSlideoutProps {
        is_open,
        task_id,
        execution_id,
        on_close,
    } = props;

    // Query task details
    let (task_state, refetch) = use_task_details_query(execution_id.clone(), task_id.clone());

    // Refetch when task_id changes
    let task_id_for_invalidation = task_id.clone();
    use_effect(move || {
        if !task_id_for_invalidation.is_empty() {
            refetch();
        }
    });

    // Get task from query
    let task = if task_id.is_empty() {
        None
    } else {
        query_result
            .suspend()
            .ok()
            .and_then(|result| result.ok())
            .flatten()
    };

    let mut selected_tab = use_signal(|| "Details".to_string());

    // Fetch user input request
    let input_request = use_signal(|| None::<s_e_e_core::UserInputRequest>);

    use_effect({
        let task_id = task_id.clone();
        let execution_id = execution_id.clone();
        let mut input_request = input_request;

        move || {
            tracing::debug!("[TASK_SLIDEOUT] Effect running for task_id={}", task_id);
            if !task_id.is_empty() && !execution_id.is_empty() {
                let task_id_for_spawn = task_id.clone();
                let execution_id_for_spawn = execution_id.clone();
                let mut input_request_for_spawn = input_request.clone();

                spawn(async move {
                    tracing::debug!(
                        "[TASK_SLIDEOUT] Fetching pending inputs for execution_id={}",
                        execution_id_for_spawn
                    );
                    if let Ok(requests) =
                        s_e_e_core::get_pending_inputs(&execution_id_for_spawn).await
                    {
                        tracing::debug!(
                            "[TASK_SLIDEOUT] Found {} pending requests",
                            requests.len()
                        );
                        if let Some(req) = requests
                            .iter()
                            .find(|req| req.task_execution_id == task_id_for_spawn)
                        {
                            tracing::debug!("[TASK_SLIDEOUT] Found matching request");
                            input_request_for_spawn.set(Some(req.clone()));
                        } else {
                            tracing::debug!(
                                "[TASK_SLIDEOUT] No matching request for task_id={}",
                                task_id_for_spawn
                            );
                            input_request_for_spawn.set(None);
                        }
                    }
                });
            }
        }
    });

    let is_active = |tab: &str| -> String {
        format!(
            "py-2 px-1 border-b-2 font-medium text-sm {}",
            if selected_tab() == tab {
                "border-blue-500 text-blue-600 dark:text-blue-400"
            } else {
                "border-transparent text-zinc-500 hover:text-zinc-700 hover:border-zinc-300 dark:text-zinc-400 dark:hover:text-zinc-300"
            }
        )
    };

    rsx! {
        Slideout {
            is_open,
            backdrop_class: "bg-zinc-500/20 backdrop-blur-sm".to_string(),
            on_close,
            title: task.as_ref().map(|t| t.name.clone()).unwrap_or_else(|| "Task Details".to_string()),
            subtitle: Some(format!("ID: {task_id}")),
            show_close_button: Some(true),

            children: rsx! {
                if let Some(task) = task.as_ref() {
                    div { class: "space-y-4",
                        // Tab buttons
                        div { class: "border-b border-zinc-200 dark:border-zinc-700",
                            div { class: "flex space-x-8",
                                button {
                                    class: is_active("Details"),
                                    onclick: move |_| selected_tab.set("Details".to_string()),
                                    "Details"
                                }
                                button {
                                    class: is_active("Output"),
                                    onclick: move |_| selected_tab.set("Output".to_string()),
                                    "Output"
                                }
                                button {
                                    class: is_active("User Input"),
                                    onclick: move |_| selected_tab.set("User Input".to_string()),
                                    "User Input"
                                }
                            }
                        }

                        // Tab content
                        div { class: "mt-4",
                            if selected_tab() == "Details" {
                                div { class: "bg-zinc-50 dark:bg-zinc-800 rounded-xl p-6",
                                    h4 { class: "text-base font-semibold text-zinc-950 dark:text-white mb-4", "Task Information" }
                                    div { class: "space-y-3",
                                        div { class: "flex justify-between",
                                            span { class: "text-sm font-medium text-zinc-600 dark:text-zinc-400", "Name:" }
                                            span { class: "text-sm text-zinc-900 dark:text-zinc-100", "{task.name}" }
                                        }
                                        div { class: "flex justify-between",
                                            span { class: "text-sm font-medium text-zinc-600 dark:text-zinc-400", "Status:" }
                                            span { class: "text-sm text-zinc-900 dark:text-zinc-100", "{task.status:?}" }
                                        }
                                        div { class: "flex justify-between",
                                            span { class: "text-sm font-medium text-zinc-600 dark:text-zinc-400", "ID:" }
                                            span { class: "text-sm text-zinc-900 dark:text-zinc-100 font-mono", "{task.id}" }
                                        }
                                        if let Some(error) = task.error.as_ref() {
                                            div { class: "flex justify-between",
                                                span { class: "text-sm font-medium text-zinc-600 dark:text-zinc-400", "Error:" }
                                                span { class: "text-sm text-red-600 dark:text-red-400", "{error}" }
                                            }
                                        }
                                        div { class: "flex justify-between",
                                            span { class: "text-sm font-medium text-zinc-600 dark:text-zinc-400", "Created At:" }
                                            span { class: "text-sm text-zinc-900 dark:text-zinc-100", "{task.created_at}" }
                                        }
                                        if let Some(completed_at) = task.completed_at.as_ref() {
                                            div { class: "flex justify-between",
                                                span { class: "text-sm font-medium text-zinc-600 dark:text-zinc-400", "Completed At:" }
                                                span { class: "text-sm text-zinc-900 dark:text-zinc-100", "{completed_at}" }
                                            }
                                        }
                                    }
                                }
                            } else if selected_tab() == "Output" {
                                div { class: "bg-zinc-50 dark:bg-zinc-800 rounded-xl p-6",
                                    h4 { class: "text-base font-semibold text-zinc-950 dark:text-white mb-4", "Output" }
                                    if let Some(output) = task.output.as_ref() {
                                        div { class: "bg-white dark:bg-zinc-900 rounded-lg p-4 border border-zinc-200 dark:border-zinc-700",
                                            pre { class: "text-sm text-zinc-900 dark:text-zinc-100 whitespace-pre-wrap font-mono overflow-x-auto",
                                                "{output}"
                                            }
                                        }
                                    } else {
                                        div { class: "text-center text-zinc-500 dark:text-zinc-400 text-sm py-8",
                                            "No output available"
                                        }
                                    }
                                }
                            } else if selected_tab() == "User Input" {
                                if let Some(req) = input_request() {
                                    div { class: "bg-zinc-50 dark:bg-zinc-800 rounded-xl p-6",
                                        h4 { class: "text-base font-semibold text-zinc-950 dark:text-white mb-4", "User Input Prompt" }
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
                                                    "{req.input_type}"
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
                                                div { class: format!("inline-flex px-3 py-1 rounded-full text-xs font-medium {}",
                                                    if req.status.to_string() == "pending" {
                                                        "bg-amber-100 text-amber-800 dark:bg-amber-900 dark:text-amber-200"
                                                    } else {
                                                        "bg-emerald-100 text-emerald-800 dark:bg-emerald-900 dark:text-emerald-200"
                                                    }
                                                ),
                                                    "{req.status}"
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    div { class: "bg-zinc-50 dark:bg-zinc-800 rounded-xl p-6",
                                        h4 { class: "text-base font-semibold text-zinc-950 dark:text-white mb-4", "User Input Prompt" }
                                        div { class: "text-center text-zinc-500 dark:text-zinc-400 text-sm py-8",
                                            "No user input requested for this task"
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    div { class: "text-center text-zinc-500 dark:text-zinc-400", "Task not found" }
                }
            },

            footer: None,
        }
    }
}
