use crate::components::Slideout;
use crate::icons::Icon;
use dioxus::prelude::*;
use s_e_e_core::TaskExecution;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
struct RenderableTask {
    id: String,
    name: String,
    status_icon: &'static str,
    status_color: &'static str,
    children: Vec<RenderableTask>,
}

#[derive(Props, PartialEq, Clone)]
struct TaskBoxProps {
    task: RenderableTask,
    on_click: EventHandler<String>,
}

#[component]
pub fn WorkflowFlowGraph(snapshot: Value, tasks: Vec<TaskExecution>) -> Element {
    let mut slideout_open = use_signal(|| false);
    let mut selected_task_name = use_signal(|| String::new());

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
                        on_click: move |task_name| {
                            selected_task_name.set(task_name);
                            slideout_open.set(true);
                        },
                    }
                }
            }
        }

        Slideout {
            is_open: slideout_open(),
            backdrop_class: "bg-zinc-500/20 backdrop-blur-sm".to_string(),
            on_close: move |_| slideout_open.set(false),
            title: selected_task_name().to_string(),
            subtitle: None,
            show_close_button: Some(true),

            children: rsx! {
                div { class: "text-zinc-700 dark:text-zinc-300",
                    "{selected_task_name()}"
                }
            },

            footer: None,
        }
    }
}

#[component]
fn TaskBox(props: TaskBoxProps) -> Element {
    let TaskBoxProps { task, on_click } = props;

    rsx! {
        div { class: "min-w-[120px]",
            div {
                class: "bg-zinc-50 dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 p-6 shadow-sm cursor-pointer hover:bg-zinc-100 dark:hover:bg-zinc-700 transition-colors relative",
                onclick: move |_| on_click.call(task.name.clone()),
                // Status icon - absolute positioned in top right
                div {
                    class: "{task.status_color} absolute top-2 right-2 w-8 h-8 rounded-full flex items-center justify-center",
                    Icon {
                        name: task.status_icon.to_string(),
                        size: Some("w-4 h-4".to_string()),
                        variant: Some("outline".to_string()),
                        class: Some("".to_string()),
                    }
                }
                div { class: "flex items-start justify-between mb-2",
                    h4 { class: "text-base font-semibold text-zinc-950 dark:text-white flex-1 pr-10 truncate", "{task.name}" }
                }
                div { class: "text-sm text-zinc-500 dark:text-zinc-400 truncate", "ID: {task.id}" }
            }

            if !task.children.is_empty() {
                div { class: "grid gap-4 mt-4 overflow-x-auto", style: format!("grid-template-columns: repeat({}, minmax(120px, 1fr)); min-width: {}px", task.children.len(), task.children.len() * 140),
                    for child in task.children.iter() {
                        TaskBox {
                            task: child.clone(),
                            on_click,
                        }
                    }
                }
            }
        }
    }
}

fn build_renderable_task(
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

fn get_status_style(status: &str) -> (&'static str, &'static str) {
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
