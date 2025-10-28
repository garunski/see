use s_e_e_core::TaskExecution;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct RenderableTask {
    pub id: String,
    pub name: String,
    pub status_icon: &'static str,
    pub status_color: &'static str,
    pub children: Vec<RenderableTask>,
}

pub fn build_renderable_task(
    task_data: &Value,
    task_map: &HashMap<String, &TaskExecution>,
) -> Option<RenderableTask> {
    let task_id = task_data.get("id")?.as_str()?;
    let task_name = task_data
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");

    let task_exec = task_map.get(task_id)?;
    let (status_icon, status_color) = get_status_style(task_exec.status.as_str());

    let next_tasks = task_data
        .get("next_tasks")
        .and_then(|v| v.as_array())
        .map(|arr| arr.to_vec())
        .unwrap_or_default();

    let children = next_tasks
        .iter()
        .filter_map(|child_data| build_renderable_task(child_data, task_map))
        .collect();

    Some(RenderableTask {
        id: task_id.to_string(),
        name: task_name.to_string(),
        status_icon,
        status_color,
        children,
    })
}

pub fn get_status_style(status: &str) -> (&'static str, &'static str) {
    match status {
        "complete" => (
            "check_circle",
            "bg-emerald-100 dark:bg-emerald-900/30 text-emerald-700 dark:text-emerald-300",
        ),
        "failed" => (
            "exclamation_circle",
            "bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300",
        ),
        "in-progress" | "in_progress" => (
            "bars_3",
            "bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300",
        ),
        "waiting_for_input" => (
            "pause",
            "bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-300",
        ),
        _ => (
            "bars_3",
            "bg-zinc-100 dark:bg-zinc-800 text-zinc-700 dark:text-zinc-300",
        ),
    }
}
