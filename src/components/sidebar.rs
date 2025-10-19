use crate::components::ExecutionStatus;
use dioxus::prelude::*;

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
) -> Element {
    rsx! {
        aside {
            class: "flex flex-col w-full lg:w-64 bg-white dark:bg-zinc-900 border-r border-zinc-950/5 dark:border-white/5",

            // Logo and title
            div {
                class: "flex flex-col border-b border-zinc-950/5 p-4 dark:border-white/5",
                div {
                    class: "flex items-center space-x-3 mb-4",
                    div {
                        class: "w-8 h-8 bg-zinc-900 dark:bg-white rounded-lg flex items-center justify-center text-white dark:text-zinc-900 text-lg font-semibold",
                        "⚡"
                    }
                    div {
                        h1 {
                            class: "text-lg font-semibold text-zinc-950 dark:text-white",
                            "Workflow Executor"
                        }
                        p {
                            class: "text-zinc-500 dark:text-zinc-400 text-sm",
                            "Execute and manage workflows"
                        }
                    }
                }

                // Theme toggle
                button {
                    class: "flex w-full items-center gap-3 rounded-lg px-2 py-2.5 text-left text-base/6 font-medium text-zinc-950 sm:py-2 sm:text-sm/5 data-hover:bg-zinc-950/5 data-active:bg-zinc-950/5 dark:text-white dark:data-hover:bg-white/5 dark:data-active:bg-white/5",
                    onclick: move |_| on_toggle_dark_mode.call(()),
                    div {
                        class: "w-5 h-5",
                        if dark_mode { "🌙" } else { "☀️" }
                    }
                    span {
                        if dark_mode { "Dark Mode" } else { "Light Mode" }
                    }
                }
            }

            // File input section
            div {
                class: "flex flex-1 flex-col overflow-y-auto p-4",
                div {
                    class: "flex flex-col gap-0.5",
                    label {
                        class: "mb-1 px-2 text-xs/6 font-medium text-zinc-500 dark:text-zinc-400",
                        "Workflow File"
                    }
                    div {
                        class: "space-y-3",
                        input {
                            class: "w-full px-3 py-2 bg-white dark:bg-zinc-900 border border-zinc-300 dark:border-zinc-700 rounded-lg text-zinc-950 dark:text-white placeholder-zinc-500 dark:placeholder-zinc-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200",
                            r#type: "text",
                            placeholder: "Select workflow file...",
                            value: workflow_file.clone(),
                            oninput: move |evt| {
                                on_workflow_file_change.call(evt.value());
                            }
                        }
                        button {
                            class: "relative isolate inline-flex items-baseline justify-center gap-x-2 rounded-lg border text-base/6 font-semibold px-[calc(--spacing(3.5)-1px)] py-[calc(--spacing(2.5)-1px)] sm:px-[calc(--spacing(3)-1px)] sm:py-[calc(--spacing(1.5)-1px)] sm:text-sm/6 border-zinc-950/10 text-zinc-950 data-active:bg-zinc-950/2.5 data-hover:bg-zinc-950/2.5 dark:border-white/15 dark:text-white dark:data-active:bg-white/5 dark:data-hover:bg-white/5 disabled:opacity-50 disabled:cursor-not-allowed",
                            disabled: is_picking_file,
                            onclick: move |_| on_pick_file.call(()),
                            div {
                                class: "w-5 h-5",
                                if is_picking_file {
                                    "⏳"
                                } else {
                                    "📁"
                                }
                            }
                            span { "Browse Files" }
                        }
                    }
                }
            }

            // Execute button
            div {
                class: "flex flex-col border-t border-zinc-950/5 p-4 dark:border-white/5",
                button {
                    class: "relative isolate inline-flex items-baseline justify-center gap-x-2 rounded-lg border text-base/6 font-semibold px-[calc(--spacing(3.5)-1px)] py-[calc(--spacing(2.5)-1px)] sm:px-[calc(--spacing(3)-1px)] sm:py-[calc(--spacing(1.5)-1px)] sm:text-sm/6 text-white bg-emerald-600 border-emerald-700/90 data-hover:bg-emerald-700 data-active:bg-emerald-700 disabled:opacity-50 disabled:cursor-not-allowed",
                    onclick: move |_| on_execute.call(()),
                    disabled: matches!(execution_status, ExecutionStatus::Running),
                    if matches!(execution_status, ExecutionStatus::Running) {
                        div {
                            class: "animate-spin w-5 h-5 border-2 border-white border-t-transparent rounded-full"
                        }
                        span { "Executing..." }
                    } else {
                        div {
                            class: "w-5 h-5",
                            "🚀"
                        }
                        span { "Execute Workflow" }
                    }
                }
            }

            // Status indicator
            if !matches!(execution_status, ExecutionStatus::Idle) {
                div {
                    class: "flex flex-col border-t border-zinc-950/5 p-4 dark:border-white/5",
                    div {
                        class: "flex items-center space-x-3",
                        div {
                            class: format!("w-3 h-3 rounded-full {}", match execution_status {
                                ExecutionStatus::Running => "bg-blue-500 animate-pulse",
                                ExecutionStatus::Complete => "bg-emerald-500",
                                ExecutionStatus::Failed => "bg-red-500",
                                _ => "bg-zinc-400"
                            })
                        }
                        span {
                            class: "text-sm font-medium text-zinc-950 dark:text-white",
                            match execution_status {
                                ExecutionStatus::Running => "Running",
                                ExecutionStatus::Complete => "Complete",
                                ExecutionStatus::Failed => "Failed",
                                _ => "Idle"
                            }
                        }
                    }
                }
            }
        }
    }
}
