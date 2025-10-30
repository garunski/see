use dioxus::prelude::*;
use s_e_e_core::{TaskExecution, WorkflowExecutionStatus};
use serde_json::Value;
use std::collections::HashMap;

use super::task_box::TaskBox;
use super::task_preprocessing::build_renderable_task;

#[component]
pub fn WorkflowFlowGraph(
    snapshot: Value,
    tasks: Vec<TaskExecution>,
    execution_id: String,
    workflow_status: WorkflowExecutionStatus,
) -> Element {
    let task_map: HashMap<String, &TaskExecution> =
        tasks.iter().map(|t| (t.id.clone(), t)).collect();

    let root_tasks = snapshot
        .get("tasks")
        .and_then(|v| v.as_array())
        .map(|arr| arr.to_vec())
        .unwrap_or_default();

    let workflow_is_failed = matches!(workflow_status, WorkflowExecutionStatus::Failed);

    let renderable_tasks = root_tasks
        .iter()
        .filter_map(|task_data| build_renderable_task(task_data, &task_map, workflow_is_failed))
        .collect::<Vec<_>>();

    if renderable_tasks.is_empty() && tasks.is_empty() {
        return rsx! {
            div { class: "bg-white dark:bg-zinc-900 rounded-lg shadow-sm ring-1 ring-zinc-950/5 dark:ring-white/10 overflow-x-auto p-4",
                div { class: "flex flex-col items-center justify-center py-12 text-center",
                    div { class: "mb-4",
                        svg {
                            class: "h-12 w-12 text-zinc-400 dark:text-zinc-500 mx-auto animate-spin",
                            xmlns: "http://www.w3.org/2000/svg",
                            fill: "none",
                            view_box: "0 0 24 24",
                            circle {
                                class: "opacity-25",
                                cx: "12",
                                cy: "12",
                                r: "10",
                                stroke: "currentColor",
                                stroke_width: "4"
                            }
                            path {
                                class: "opacity-75",
                                fill: "currentColor",
                                d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                            }
                        }
                    }
                    h3 { class: "text-base font-semibold text-zinc-950 dark:text-white mb-2",
                        "Workflow Executing..."
                    }
                    p { class: "text-sm text-zinc-600 dark:text-zinc-400",
                        "The workflow is starting up. Tasks will appear here as they execute."
                    }
                }
            }
        };
    }

    rsx! {
        div { class: "bg-white dark:bg-zinc-900 rounded-lg shadow-sm ring-1 ring-zinc-950/5 dark:ring-white/10 overflow-x-auto p-4",
            h3 { class: "text-base font-semibold text-zinc-950 dark:text-white mb-4", "Workflow Flow" }
            div { class: "grid gap-4 py-4", style: format!("grid-template-columns: repeat({}, minmax(120px, 1fr)); min-width: max(100%, {}px)", renderable_tasks.len(), renderable_tasks.len() * 140),
                for task in renderable_tasks.iter() {
                    TaskBox {
                        task: task.clone(),
                        execution_id: execution_id.clone(),
                    }
                }
            }
        }
    }
}
