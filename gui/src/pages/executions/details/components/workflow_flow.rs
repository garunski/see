use dioxus::prelude::*;
use s_e_e_core::TaskExecution;
use serde_json::Value;
use std::collections::HashMap;

use super::task_box::TaskBox;
use super::task_preprocessing::build_renderable_task;

#[component]
pub fn WorkflowFlowGraph(
    snapshot: Value,
    tasks: Vec<TaskExecution>,
    execution_id: String,
) -> Element {
    // Build task_id -> TaskExecution map for quick lookups during prep
    let task_map: HashMap<String, &TaskExecution> =
        tasks.iter().map(|t| (t.id.clone(), t)).collect();

    // Extract root tasks from snapshot
    let root_tasks = snapshot
        .get("tasks")
        .and_then(|v| v.as_array())
        .map(|arr| arr.to_vec())
        .unwrap_or_default();

    // Build the full tree structure before rendering
    let renderable_tasks = root_tasks
        .iter()
        .filter_map(|task_data| build_renderable_task(task_data, &task_map))
        .collect::<Vec<_>>();

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
