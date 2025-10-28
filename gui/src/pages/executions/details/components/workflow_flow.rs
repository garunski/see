use dioxus::prelude::*;
use engine::TaskStatus;
use s_e_e_core::TaskInfo;
use std::collections::HashMap;

fn get_status_badge_class(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Complete => {
            "bg-emerald-100 text-emerald-800 dark:bg-emerald-900 dark:text-emerald-200"
        }
        TaskStatus::Failed => "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
        TaskStatus::InProgress => "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
        TaskStatus::Pending => "bg-zinc-100 text-zinc-800 dark:bg-zinc-900 dark:text-zinc-200",
        TaskStatus::WaitingForInput => {
            "bg-amber-100 text-amber-800 dark:bg-amber-900 dark:text-amber-200"
        }
    }
}

#[component]
pub fn WorkflowFlow(
    tasks: Vec<TaskInfo>,
    parent_child_mapping: HashMap<String, Vec<String>>,
    on_task_click: EventHandler<usize>,
) -> Element {
    if tasks.is_empty() {
        return rsx! {
            div { class: "bg-white dark:bg-zinc-900 rounded-lg shadow-sm ring-1 ring-zinc-950/5 dark:ring-white/10 p-6",
                div { class: "text-center text-zinc-500 dark:text-zinc-400",
                    "No tasks to display"
                }
            }
        };
    }

    // Build taskId -> index map
    let task_index_map: HashMap<&str, usize> = tasks
        .iter()
        .enumerate()
        .map(|(i, t)| (t.id.as_str(), i))
        .collect();

    // Build childId -> true set from parent_child_mapping
    let mut child_ids = std::collections::HashSet::new();
    for children in parent_child_mapping.values() {
        for child_id in children {
            child_ids.insert(child_id.as_str());
        }
    }

    // Find root tasks (tasks that are not children of any other task)
    let root_task_indices: Vec<usize> = tasks
        .iter()
        .enumerate()
        .filter(|(_, task)| !child_ids.contains(task.id.as_str()))
        .map(|(i, _)| i)
        .collect();

    // Dynamic column count equals number of root tasks (minimum 1)
    let root_count = if root_task_indices.is_empty() {
        1
    } else {
        root_task_indices.len()
    };

    // Map parent task index -> immediate children indices (from parent_child_mapping)
    let mut children_for_parent: HashMap<usize, Vec<usize>> = HashMap::new();
    for (parent_id, children_ids) in parent_child_mapping.iter() {
        if let Some(&parent_idx) = task_index_map.get(parent_id.as_str()) {
            let mut indices: Vec<usize> = Vec::new();
            for child_id in children_ids {
                if let Some(&child_idx) = task_index_map.get(child_id.as_str()) {
                    indices.push(child_idx);
                }
            }
            if !indices.is_empty() {
                children_for_parent.insert(parent_idx, indices);
            }
        }
    }

    rsx! {
        div { class: "bg-white dark:bg-zinc-900 rounded-lg shadow-sm ring-1 ring-zinc-950/5 dark:ring-white/10 p-6",
            h3 { class: "text-base font-semibold text-zinc-950 dark:text-white mb-6", "Workflow Flow" }

            div { class: "grid gap-4 py-4", style: format!("grid-template-columns: repeat({}, minmax(0, 1fr))", root_count),
                for &task_idx in root_task_indices.iter() {
                    div {
                        class: format!("bg-zinc-50 dark:bg-zinc-800 rounded-xl border p-6 shadow-sm cursor-pointer hover:bg-zinc-100 dark:hover:bg-zinc-700 transition-colors {}",
                            if matches!(tasks[task_idx].status, TaskStatus::WaitingForInput) {
                                "border-amber-300 dark:border-amber-700 animate-pulse"
                            } else {
                                "border-zinc-200 dark:border-zinc-700"
                            }
                        ),
                        onclick: move |_| on_task_click.call(task_idx),

                        div { class: "flex items-center justify-between",
                            h4 { class: "text-base font-semibold text-zinc-950 dark:text-white", "{tasks[task_idx].name}" }
                            div {
                                class: format!("px-3 py-1 text-sm rounded-full font-medium {}", get_status_badge_class(&tasks[task_idx].status)),
                                match &tasks[task_idx].status {
                                    TaskStatus::Complete => "Complete",
                                    TaskStatus::Failed => "Failed",
                                    TaskStatus::InProgress => "In Progress",
                                    TaskStatus::Pending => "Pending",
                                    TaskStatus::WaitingForInput => "Waiting for Input",
                                }
                            }
                        }

                        div { class: "mt-2 text-sm text-zinc-500 dark:text-zinc-400",
                            "Task ID: {tasks[task_idx].id}"
                        }
                    }

                    // Children grid (immediate next_tasks)
                    if let Some(child_indices) = children_for_parent.get(&task_idx) {
                        if !child_indices.is_empty() {
                            div { class: "grid gap-4 mt-4", style: format!("grid-template-columns: repeat({}, minmax(0, 1fr))", child_indices.len()),
                                for &child_idx in child_indices.iter() {
                                    div {
                                        class: format!("bg-zinc-50 dark:bg-zinc-800 rounded-xl border p-6 shadow-sm cursor-pointer hover:bg-zinc-100 dark:hover:bg-zinc-700 transition-colors {}",
                                            if matches!(tasks[child_idx].status, TaskStatus::WaitingForInput) {
                                                "border-amber-300 dark:border-amber-700 animate-pulse"
                                            } else {
                                                "border-zinc-200 dark:border-zinc-700"
                                            }
                                        ),
                                        onclick: move |_| on_task_click.call(child_idx),

                                        div { class: "flex items-center justify-between",
                                            h4 { class: "text-base font-semibold text-zinc-950 dark:text-white", "{tasks[child_idx].name}" }
                                            div {
                                                class: format!("px-3 py-1 text-sm rounded-full font-medium {}", get_status_badge_class(&tasks[child_idx].status)),
                                                match &tasks[child_idx].status {
                                                    TaskStatus::Complete => "Complete",
                                                    TaskStatus::Failed => "Failed",
                                                    TaskStatus::InProgress => "In Progress",
                                                    TaskStatus::Pending => "Pending",
                                                    TaskStatus::WaitingForInput => "Waiting for Input",
                                                }
                                            }
                                        }

                                        div { class: "mt-2 text-sm text-zinc-500 dark:text-zinc-400",
                                            "Task ID: {tasks[child_idx].id}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
