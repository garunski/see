use s_e_e_core::TaskExecution;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct RenderableTask {
    pub id: String,
    pub name: String,
    pub function_name: String,
    pub function_icon: &'static str,
    pub function_color: &'static str,
    pub status_icon: &'static str,
    pub status_color: &'static str,
    pub children: Vec<RenderableTask>,
    pub has_execution_data: bool,
}

pub fn build_renderable_task(
    task_data: &Value,
    task_map: &HashMap<String, &TaskExecution>,
    workflow_is_failed: bool,
) -> Option<RenderableTask> {
    let task_id = task_data.get("id")?.as_str()?;
    let task_name = task_data
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");

    // Get function type
    let function_name = task_data
        .get("function")
        .and_then(|f| f.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("unknown");

    let (function_icon, function_color) = get_function_style(function_name);

    // Try to get task execution data
    let has_execution_data = task_map.contains_key(task_id);
    let (status_icon, status_color) = if let Some(task_exec) = task_map.get(task_id) {
        // Task has execution data - use its status
        get_status_style(task_exec.status.as_str())
    } else if workflow_is_failed {
        // Task has no execution data and workflow failed - show as errored (never started)
        get_status_style("errored")
    } else {
        // Task has no execution data and workflow is still running - show as pending
        get_status_style("pending")
    };

    let next_tasks = task_data
        .get("next_tasks")
        .and_then(|v| v.as_array())
        .map(|arr| arr.to_vec())
        .unwrap_or_default();

    let children = next_tasks
        .iter()
        .filter_map(|child_data| build_renderable_task(child_data, task_map, workflow_is_failed))
        .collect();

    Some(RenderableTask {
        id: task_id.to_string(),
        name: task_name.to_string(),
        function_name: function_name.to_string(),
        function_icon,
        function_color,
        status_icon,
        status_color,
        children,
        has_execution_data,
    })
}

pub fn get_function_style(function_name: &str) -> (&'static str, &'static str) {
    match function_name {
        "cli_command" => ("terminal", "bg-blue-600 dark:bg-blue-700"),
        "cursor_agent" => ("cursor", "bg-purple-600 dark:bg-purple-700"),
        "user_input" => ("bars_3", "bg-amber-600 dark:bg-amber-700"),
        _ => ("code_bracket", "bg-gray-600 dark:bg-gray-700"),
    }
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
        "errored" => (
            "exclamation_circle",
            "bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300 opacity-60",
        ),
        "in-progress" | "in_progress" => (
            "bars_3",
            "bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300",
        ),
        "waiting_for_input" => (
            "pause",
            "bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-300",
        ),
        "pending" => (
            "bars_3",
            "bg-zinc-100 dark:bg-zinc-800 text-zinc-500 dark:text-zinc-400 opacity-50",
        ),
        _ => (
            "bars_3",
            "bg-zinc-100 dark:bg-zinc-800 text-zinc-700 dark:text-zinc-300",
        ),
    }
}
