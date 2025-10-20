use crate::components::ExecutionStatus;
use crate::state::SidebarTab;
use dioxus::prelude::*;
use see_core::WorkflowExecutionSummary;

#[component]
fn HistoryItem(
    execution: WorkflowExecutionSummary,
    on_load_execution: EventHandler<String>,
    on_delete_execution: EventHandler<String>,
    is_selected: bool,
) -> Element {
    let execution_id = execution.id.clone();
    let execution_id_for_delete = execution.id.clone();
    rsx! {
        div {
            class: format!("flex items-center justify-between p-3 rounded-lg transition-colors cursor-pointer {}",
                if is_selected {
                    "bg-blue-100 dark:bg-blue-900 border-2 border-blue-500"
                } else {
                    "bg-zinc-50 dark:bg-zinc-800 hover:bg-zinc-100 dark:hover:bg-zinc-700"
                }
            ),
            onclick: move |_| on_load_execution.call(execution_id.clone()),
            div { class: "flex-1 min-w-0",
                div { class: "flex items-center gap-2 mb-1",
                    h4 { class: "text-sm font-medium text-zinc-950 dark:text-white truncate", "{execution.workflow_name}" }
                    div {
                        class: format!("px-2 py-0.5 text-xs rounded-full {}",
                            if execution.success {
                                "bg-emerald-100 text-emerald-800 dark:bg-emerald-900 dark:text-emerald-200"
                            } else {
                                "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200"
                            }
                        ),
                        if execution.success { "Success" } else { "Failed" }
                    }
                }
                div { class: "text-xs text-zinc-500 dark:text-zinc-400",
                    "{execution.timestamp}"
                }
                div { class: "text-xs text-zinc-500 dark:text-zinc-400",
                    "{execution.task_count} tasks"
                }
            }
            button {
                class: "ml-2 p-1 text-zinc-400 hover:text-red-600 dark:hover:text-red-400 transition-colors",
                onclick: move |evt| {
                    evt.stop_propagation();
                    on_delete_execution.call(execution_id_for_delete.clone());
                },
                "🗑️"
            }
        }
    }
}

#[component]
pub fn Sidebar(
    workflow_file: String,
    on_workflow_file_change: EventHandler<String>,
    is_picking_file: bool,
    on_pick_file: EventHandler<()>,
    dark_mode: bool,
    on_toggle_dark_mode: EventHandler<()>,
    execution_status: ExecutionStatus,
    on_execute: EventHandler<()>,
    is_viewing_history: bool,
    sidebar_tab: SidebarTab,
    on_tab_change: EventHandler<SidebarTab>,
    workflow_history: Vec<WorkflowExecutionSummary>,
    on_load_execution: EventHandler<String>,
    on_delete_execution: EventHandler<String>,
    selected_history_id: Option<String>,
) -> Element {
    rsx! {
        aside { class: "fixed top-0 left-0 z-10 flex flex-col w-full lg:w-64 h-screen bg-white dark:bg-zinc-900 border-r border-zinc-950/5 dark:border-white/5",
            div { class: "flex flex-col border-b border-zinc-950/5 p-4 dark:border-white/5",
                div { class: "flex items-center space-x-3 mb-4",
                    div { class: "w-8 h-8 bg-zinc-900 dark:bg-white rounded-lg flex items-center justify-center text-white dark:text-zinc-900 text-lg font-semibold", "⚡" }
                    div {
                        h1 { class: "text-lg font-semibold text-zinc-950 dark:text-white", "Workflow Executor" }
                        p { class: "text-zinc-500 dark:text-zinc-400 text-sm", "Execute and manage workflows" }
                    }
                }

                button { class: "flex w-full items-center gap-3 rounded-lg px-2 py-2.5 text-left text-base/6 font-medium text-zinc-950 sm:py-2 sm:text-sm/5 data-hover:bg-zinc-950/5 data-active:bg-zinc-950/5 dark:text-white dark:data-hover:bg-white/5 dark:data-active:bg-white/5",
                    onclick: move |_| on_toggle_dark_mode.call(()),
                    div { class: "w-5 h-5", if dark_mode { "🌙" } else { "☀️" } }
                    span { if dark_mode { "Dark Mode" } else { "Light Mode" } }
                }
            }

            // Tab navigation
            div { class: "flex border-b border-zinc-950/5 dark:border-white/5",
                button {
                    class: format!("flex-1 px-4 py-2 text-sm font-medium border-b-2 transition-colors {}",
                        if matches!(sidebar_tab, SidebarTab::Upload) {
                            "border-blue-500 text-blue-600 dark:text-blue-400"
                        } else {
                            "border-transparent text-zinc-500 dark:text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-300"
                        }
                    ),
                    onclick: move |_| on_tab_change.call(SidebarTab::Upload),
                    "Upload"
                }
                button {
                    class: format!("flex-1 px-4 py-2 text-sm font-medium border-b-2 transition-colors {}",
                        if matches!(sidebar_tab, SidebarTab::History) {
                            "border-blue-500 text-blue-600 dark:text-blue-400"
                        } else {
                            "border-transparent text-zinc-500 dark:text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-300"
                        }
                    ),
                    onclick: move |_| on_tab_change.call(SidebarTab::History),
                    "History"
                }
            }

            div { class: "flex flex-1 flex-col overflow-y-auto p-4",
                if matches!(sidebar_tab, SidebarTab::Upload) {
                    div { class: "flex flex-col gap-0.5",
                        label { class: "mb-1 px-2 text-xs/6 font-medium text-zinc-500 dark:text-zinc-400", "Workflow File" }
                        div { class: "space-y-3",
                            input { class: "w-full px-3 py-2 bg-white dark:bg-zinc-900 border border-zinc-300 dark:border-zinc-700 rounded-lg text-zinc-950 dark:text-white placeholder-zinc-500 dark:placeholder-zinc-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200", r#type: "text", placeholder: "Select workflow file...", value: workflow_file.clone(), oninput: move |evt| { on_workflow_file_change.call(evt.value()); } }
                            if !workflow_file.is_empty() && workflow_file != "workflow.json" {
                                div { class: "text-sm text-zinc-600 dark:text-zinc-400 bg-zinc-50 dark:bg-zinc-800 px-3 py-2 rounded-lg",
                                    "Selected: {workflow_file}"
                                }
                            }
                            button { class: "relative isolate inline-flex items-baseline justify-center gap-x-2 rounded-lg border text-base/6 font-semibold px-[calc(--spacing(3.5)-1px)] py-[calc(--spacing(2.5)-1px)] sm:px-[calc(--spacing(3)-1px)] sm:py-[calc(--spacing(1.5)-1px)] sm:text-sm/6 border-zinc-950/10 text-zinc-950 data-active:bg-zinc-950/2.5 data-hover:bg-zinc-950/2.5 dark:border-white/15 dark:text-white dark:data-active:bg-white/5 dark:data-hover:bg-white/5 disabled:opacity-50 disabled:cursor-not-allowed", disabled: is_picking_file, onclick: move |_| on_pick_file.call(()),
                                div { class: "w-5 h-5", if is_picking_file { "⏳" } else { "📁" } }
                                span { "Browse Files" }
                            }
                        }
                    }
                } else {
                    div { class: "flex flex-col gap-2",
                        h3 { class: "text-sm font-medium text-zinc-950 dark:text-white mb-2", "Workflow History" }
                        if workflow_history.is_empty() {
                            div { class: "text-center py-8 text-zinc-500 dark:text-zinc-400",
                                div { class: "text-4xl mb-2", "📋" }
                                p { "No workflow executions yet" }
                            }
                        } else {
                            div { class: "space-y-2",
                                for execution in workflow_history.iter() {
                                    HistoryItem {
                                        execution: execution.clone(),
                                        on_load_execution: on_load_execution,
                                        on_delete_execution: on_delete_execution,
                                        is_selected: selected_history_id.as_ref().map_or(false, |id| id == &execution.id),
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div { class: "flex flex-col border-t border-zinc-950/5 p-4 dark:border-white/5",
                button { class: "relative isolate inline-flex items-baseline justify-center gap-x-2 rounded-lg border text-base/6 font-semibold px-[calc(--spacing(3.5)-1px)] py-[calc(--spacing(2.5)-1px)] sm:px-[calc(--spacing(3)-1px)] sm:py-[calc(--spacing(1.5)-1px)] sm:text-sm/6 text-white bg-emerald-600 border-emerald-700/90 data-hover:bg-emerald-700 data-active:bg-emerald-700 disabled:opacity-50 disabled:cursor-not-allowed", onclick: move |_| on_execute.call(()), disabled: matches!(execution_status, ExecutionStatus::Running) || is_viewing_history,
                    if matches!(execution_status, ExecutionStatus::Running) { div { class: "animate-spin w-5 h-5 border-2 border-white border-t-transparent rounded-full" } span { "Executing..." } }
                    else { div { class: "w-5 h-5", "🚀" } span { "Execute Workflow" } }
                }
            }

            if !matches!(execution_status, ExecutionStatus::Idle) {
                div { class: "flex flex-col border-t border-zinc-950/5 p-4 dark:border-white/5",
                    div { class: "flex items-center space-x-3",
                        div { class: format!("w-3 h-3 rounded-full {}", match execution_status { ExecutionStatus::Running => "bg-blue-500 animate-pulse", ExecutionStatus::Complete => "bg-emerald-500", ExecutionStatus::Failed => "bg-red-500", _ => "bg-zinc-400" }) }
                        span { class: "text-sm font-medium text-zinc-950 dark:text-white", match execution_status { ExecutionStatus::Running => "Running", ExecutionStatus::Complete => "Complete", ExecutionStatus::Failed => "Failed", _ => "Idle" } }
                    }
                }
            }
        }
    }
}
