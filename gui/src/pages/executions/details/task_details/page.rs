use crate::components::EmptyState;
use crate::pages::executions::details::task_details::components::{
    TaskDetailsHeader, TaskDetailsInfoTab, TaskDetailsOutputTab, TaskDetailsTabs,
    TaskDetailsUserInputTab,
};
use crate::queries::use_task_details_query;
use dioxus::prelude::*;

#[component]
pub fn TaskDetailsPage(execution_id: String, task_id: String) -> Element {
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
    } else if task_state.is_loading {
        return rsx! {
            div { class: "flex items-center justify-center h-64",
                "Loading task details..."
            }
        };
    } else if task_state.is_error {
        None
    } else {
        task_state.data.clone().and_then(|opt| opt)
    };

    let mut selected_tab = use_signal(|| "Details".to_string());

    // Fetch user input request
    let input_request = use_signal(|| None::<s_e_e_core::UserInputRequest>);

    use_effect({
        let task_id = task_id.clone();
        let execution_id = execution_id.clone();
        let mut selected_tab_mirror = selected_tab;

        move || {
            if !task_id.is_empty() && !execution_id.is_empty() {
                let task_id_for_spawn = task_id.clone();
                let execution_id_for_spawn = execution_id.clone();
                let mut input_request_for_spawn = input_request;

                spawn(async move {
                    if let Ok(requests) =
                        s_e_e_core::get_pending_inputs(&execution_id_for_spawn).await
                    {
                        if let Some(req) = requests
                            .iter()
                            .find(|req| req.task_execution_id == task_id_for_spawn)
                        {
                            input_request_for_spawn.set(Some(req.clone()));
                            // Auto-focus User Input tab if there's a pending request
                            if req.status.to_string() == "pending" {
                                selected_tab_mirror.set("User Input".to_string());
                            }
                        } else {
                            input_request_for_spawn.set(None);
                        }
                    }
                });
            }
        }
    });

    let show_user_input = input_request().is_some();

    let task_name = task
        .as_ref()
        .map(|t| t.name.clone())
        .unwrap_or_else(|| "Task Details".to_string());

    rsx! {
        div { class: "space-y-6",
            TaskDetailsHeader {
                task_name: task_name.clone(),
                task_id: task_id.clone()
            }

            if let Some(task) = task.as_ref() {
                div { class: "space-y-6",
                    TaskDetailsTabs {
                        selected_tab,
                        on_tab_change: EventHandler::new(move |tab_name| {
                            selected_tab.set(tab_name);
                        }),
                        show_user_input,
                    }

                    div { class: "mt-6",
                        if selected_tab() == "Details" {
                            TaskDetailsInfoTab { task: task.clone() }
                        } else if selected_tab() == "Output" {
                            TaskDetailsOutputTab { task: task.clone() }
                        } else if selected_tab() == "User Input" && show_user_input {
                            TaskDetailsUserInputTab { input_request: input_request() }
                        }
                    }
                }
            } else {
                EmptyState {
                    message: "Task not found".to_string(),
                }
            }
        }
    }
}
