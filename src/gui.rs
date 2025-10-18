use dioxus::prelude::*;
use rfd::FileDialog;
use simple_workflow_app::{execute_workflow, OutputCallback, WorkflowResult};
use std::sync::Arc;
use tokio::sync::mpsc;

// Include the generated Tailwind CSS
const TAILWIND_CSS: &str = include_str!("../assets/tailwind.css");

#[derive(Debug, Clone, PartialEq)]
enum ExecutionStatus {
    Idle,
    Running,
    Complete,
    Failed,
}

#[derive(Debug, Clone)]
struct AppState {
    workflow_file: String,
    execution_status: ExecutionStatus,
    workflow_result: Option<WorkflowResult>,
    output_logs: Vec<String>,
    dark_mode: bool,
    show_logs: bool,
    show_context: bool,
    toast_message: Option<String>,
    is_picking_file: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            workflow_file: "workflow.json".to_string(),
            execution_status: ExecutionStatus::Idle,
            workflow_result: None,
            output_logs: Vec::new(),
            dark_mode: false,
            show_logs: true,
            show_context: true,
            toast_message: None,
            is_picking_file: false,
        }
    }
}

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    let mut state = use_signal(|| AppState::default());

    let execute_workflow = move || {
        spawn(async move {
            let file_path = state.read().workflow_file.clone();

            // Update status to running
            state.write().execution_status = ExecutionStatus::Running;
            state.write().output_logs.clear();
            state.write().workflow_result = None;

            // Create a channel for output
            let (tx, mut rx) = mpsc::channel(100);
            let tx_clone = tx.clone();

            // Create output callback
            let output_callback: OutputCallback = Arc::new(move |msg| {
                let _ = tx_clone.try_send(msg);
            });

            // Start a task to receive output
            let mut state_clone = state.clone();
            spawn(async move {
                while let Some(msg) = rx.recv().await {
                    state_clone.write().output_logs.push(msg);
                }
            });

            // Execute workflow
            match execute_workflow(&file_path, Some(output_callback)).await {
                Ok(result) => {
                    state.write().execution_status = ExecutionStatus::Complete;
                    state.write().workflow_result = Some(result);
                    state.write().toast_message =
                        Some("Workflow completed successfully!".to_string());
                }
                Err(e) => {
                    state.write().execution_status = ExecutionStatus::Failed;
                    state.write().output_logs.push(format!("Error: {}", e));
                    state.write().toast_message = Some(format!("Workflow failed: {}", e));
                }
            }
        });
    };

    let mut toggle_dark_mode = move || {
        let current_mode = state.read().dark_mode;
        state.write().dark_mode = !current_mode;
    };

    let mut pick_file = move || {
        state.write().is_picking_file = true;
        spawn(async move {
            if let Some(path) = FileDialog::new()
                .add_filter("JSON files", &["json"])
                .set_title("Select Workflow File")
                .pick_file()
            {
                if let Some(path_str) = path.to_str() {
                    state.write().workflow_file = path_str.to_string();
                }
            }
            state.write().is_picking_file = false;
        });
    };

    let copy_to_clipboard = move |text: String| {
        // For now, just log to console - clipboard functionality can be added later
        println!("Copy to clipboard: {}", text);
    };

    let mut dismiss_toast = move || {
        state.write().toast_message = None;
    };

    rsx! {
        div {
            class: format!("min-h-screen bg-white dark:bg-zinc-900 text-zinc-950 dark:text-white {}",
                if state.read().dark_mode { "dark" } else { "" }),

            // Inject Tailwind CSS
            style {
                dangerous_inner_html: TAILWIND_CSS
            }

            // Toast notification
            if let Some(ref toast) = state.read().toast_message {
                div {
                    class: "fixed top-6 right-6 z-50 animate-slide-in",
                    div {
                        class: "bg-white/10 backdrop-blur-md border border-white/20 rounded-2xl shadow-2xl p-4 max-w-sm",
                        div {
                            class: "flex items-center justify-between",
                            span {
                                class: "text-white font-medium",
                                {toast.clone()}
                            }
                            button {
                                class: "ml-4 text-white/60 hover:text-white transition-colors",
                                onclick: move |_| dismiss_toast(),
                                "✕"
                            }
                        }
                    }
                }
            }

            // Main layout
            div {
                class: "relative isolate flex min-h-svh w-full bg-white max-lg:flex-col lg:bg-zinc-100 dark:bg-zinc-900 dark:lg:bg-zinc-950",

                // Sidebar
                aside {
                    class: "fixed inset-y-0 left-0 w-64 max-lg:hidden bg-white dark:bg-zinc-900 border-r border-zinc-950/5 dark:border-white/5",

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
                            onclick: move |_| toggle_dark_mode(),
                            div {
                                class: "w-5 h-5",
                                if state.read().dark_mode { "🌙" } else { "☀️" }
                            }
                            span {
                                if state.read().dark_mode { "Dark Mode" } else { "Light Mode" }
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
                                    value: state.read().workflow_file.clone(),
                                    oninput: move |evt| {
                                        state.write().workflow_file = evt.value();
                                    }
                                }
                                button {
                                    class: "relative isolate inline-flex items-baseline justify-center gap-x-2 rounded-lg border text-base/6 font-semibold px-[calc(--spacing(3.5)-1px)] py-[calc(--spacing(2.5)-1px)] sm:px-[calc(--spacing(3)-1px)] sm:py-[calc(--spacing(1.5)-1px)] sm:text-sm/6 border-zinc-950/10 text-zinc-950 data-active:bg-zinc-950/2.5 data-hover:bg-zinc-950/2.5 dark:border-white/15 dark:text-white dark:data-active:bg-white/5 dark:data-hover:bg-white/5 disabled:opacity-50 disabled:cursor-not-allowed",
                                    disabled: state.read().is_picking_file,
                                    onclick: move |_| pick_file(),
                                    div {
                                        class: "w-5 h-5",
                                        if state.read().is_picking_file {
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
                            onclick: move |_| execute_workflow(),
                            disabled: matches!(state.read().execution_status, ExecutionStatus::Running),
                            if matches!(state.read().execution_status, ExecutionStatus::Running) {
                                div { class: "animate-spin w-5 h-5 border-2 border-white border-t-transparent rounded-full" }
                                span { "Executing..." }
                            } else {
                                div { class: "w-5 h-5", "🚀" }
                                span { "Execute Workflow" }
                            }
                        }
                    }

                    // Status indicator
                    if !matches!(state.read().execution_status, ExecutionStatus::Idle) {
                        div {
                            class: "mt-6 p-4 bg-white/10 rounded-xl",
                            div {
                                class: "flex items-center space-x-3",
                                div {
                                    class: format!("w-3 h-3 rounded-full {}", match state.read().execution_status {
                                        ExecutionStatus::Running => "bg-blue-400 animate-pulse",
                                        ExecutionStatus::Complete => "bg-emerald-400",
                                        ExecutionStatus::Failed => "bg-red-400",
                                        _ => "bg-gray-400"
                                    })
                                }
                                span {
                                    class: "text-sm font-medium",
                                    match state.read().execution_status {
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

                // Main content area
                main {
                    class: "flex flex-1 flex-col pb-2 lg:min-w-0 lg:pt-2 lg:pr-2 lg:pl-64",
                    div {
                        class: "grow p-6 lg:rounded-lg lg:bg-white lg:p-10 lg:shadow-xs lg:ring-1 lg:ring-zinc-950/5 dark:lg:bg-zinc-900 dark:lg:ring-white/10",
                        div {
                            class: "mx-auto max-w-6xl",

                    // Workflow info card
                    if let Some(ref result) = state.read().workflow_result {
                        div {
                            class: "mb-8 bg-white dark:bg-zinc-900 shadow-xs ring-1 ring-zinc-950/5 dark:ring-white/10 rounded-2xl p-8 animate-fade-in",
                            div {
                                class: "flex items-center justify-between mb-6",
                                h2 {
                                    class: "text-2xl/8 font-semibold text-zinc-950 sm:text-xl/8 dark:text-white",
                                    "Workflow Results"
                                }
                                div {
                                    class: "flex items-center space-x-2",
                                    if result.success {
                                        div {
                                            class: "w-8 h-8 bg-emerald-500 rounded-full flex items-center justify-center text-white text-sm font-bold",
                                            "✓"
                                        }
                                    } else {
                                        div {
                                            class: "w-8 h-8 bg-red-500 rounded-full flex items-center justify-center text-white text-sm font-bold",
                                            "✕"
                                        }
                                    }
                                }
                            }

                            // Stats grid
                            div {
                                class: "grid grid-cols-1 md:grid-cols-3 gap-6",
                                div {
                                    class: "bg-white/5 rounded-xl p-6",
                                    div {
                                        class: "text-white/60 text-sm mb-2",
                                        "Workflow Name"
                                    }
                                    div {
                                        class: "text-white text-lg font-semibold",
                                        {result.workflow_name.clone()}
                                    }
                                }
                                div {
                                    class: "bg-white/5 rounded-xl p-6",
                                    div {
                                        class: "text-white/60 text-sm mb-2",
                                        "Tasks"
                                    }
                                    div {
                                        class: "text-white text-lg font-semibold",
                                        {result.task_count.to_string()}
                                    }
                                }
                                div {
                                    class: "bg-white/5 rounded-xl p-6",
                                    div {
                                        class: "text-white/60 text-sm mb-2",
                                        "Status"
                                    }
                                    div {
                                        class: "text-white text-lg font-semibold",
                                        {if result.success { "Success" } else { "Failed" }}
                                    }
                                }
                            }
                        }
                    }

                    // Collapsible sections
                    div {
                        class: "space-y-6",

                        // Output logs section
                        if !state.read().output_logs.is_empty() {
                            div {
                                class: "bg-white/10 backdrop-blur-md border border-white/20 rounded-2xl overflow-hidden",
                                button {
                                    class: "w-full px-6 py-4 text-left flex items-center justify-between hover:bg-white/5 transition-colors duration-200",
                                    onclick: move |_| {
                                        let current_show_logs = state.read().show_logs;
                                        state.write().show_logs = !current_show_logs;
                                    },
                                    div {
                                        class: "flex items-center space-x-3",
                                        div {
                                            class: "w-8 h-8 bg-cyan-500/20 rounded-lg flex items-center justify-center",
                                            "📋"
                                        }
                                        div {
                                            span {
                                                class: "font-semibold text-white text-lg",
                                                "Execution Output"
                                            }
                                            span {
                                                class: "text-sm text-white/60 ml-2",
                                                "({state.read().output_logs.len()} lines)"
                                            }
                                        }
                                    }
                                    div {
                                        class: "transform transition-transform duration-200",
                                        class: if state.read().show_logs { "rotate-180" } else { "" },
                                        "▼"
                                    }
                                }
                                if state.read().show_logs {
                                    div {
                                        class: "border-t border-white/10",
                                        div {
                                            class: "bg-black/50 text-emerald-400 p-6 font-mono text-sm max-h-80 overflow-y-auto",
                                            {state.read().output_logs.join("\n")}
                                        }
                                        div {
                                            class: "px-6 py-4 bg-white/5 flex justify-end",
                                            button {
                                                class: "px-4 py-2 bg-cyan-500/20 hover:bg-cyan-500/30 text-cyan-400 rounded-xl text-sm font-medium transition-colors duration-200 border border-cyan-500/30",
                                                onclick: move |_| {
                                                    let logs = state.read().output_logs.join("\n");
                                                    copy_to_clipboard(logs);
                                                },
                                                "Copy Output"
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Final context section
                        if let Some(ref result) = state.read().workflow_result {
                            div {
                                class: "bg-white/10 backdrop-blur-md border border-white/20 rounded-2xl overflow-hidden",
                                button {
                                    class: "w-full px-6 py-4 text-left flex items-center justify-between hover:bg-white/5 transition-colors duration-200",
                                    onclick: move |_| {
                                        let current_show_context = state.read().show_context;
                                        state.write().show_context = !current_show_context;
                                    },
                                    div {
                                        class: "flex items-center space-x-3",
                                        div {
                                            class: "w-8 h-8 bg-purple-500/20 rounded-lg flex items-center justify-center",
                                            "📊"
                                        }
                                        span {
                                            class: "font-semibold text-white text-lg",
                                            "Final Context"
                                        }
                                    }
                                    div {
                                        class: "transform transition-transform duration-200",
                                        class: if state.read().show_context { "rotate-180" } else { "" },
                                        "▼"
                                    }
                                }
                                if state.read().show_context {
                                    div {
                                        class: "border-t border-white/10",
                                        div {
                                            class: "bg-black/30 p-6",
                                            pre {
                                                class: "text-sm text-white/90 whitespace-pre-wrap overflow-x-auto font-mono",
                                                {serde_json::to_string_pretty(&result.final_context).unwrap_or_else(|_| "{}".to_string())}
                                            }
                                        }
                                        div {
                                            class: "px-6 py-4 bg-white/5 flex justify-end",
                                            button {
                                                class: "px-4 py-2 bg-purple-500/20 hover:bg-purple-500/30 text-purple-400 rounded-xl text-sm font-medium transition-colors duration-200 border border-purple-500/30",
                                                onclick: move |_| {
                                                    println!("Copy context to clipboard");
                                                },
                                                "Copy Context"
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Errors section
                        if let Some(ref result) = state.read().workflow_result {
                            if !result.errors.is_empty() {
                                div {
                                    class: "bg-red-500/10 backdrop-blur-md border border-red-500/30 rounded-2xl p-6 animate-fade-in",
                                    div {
                                        class: "flex items-center space-x-3 mb-4",
                                        div {
                                            class: "w-8 h-8 bg-red-500/20 rounded-lg flex items-center justify-center",
                                            "⚠️"
                                        }
                                        h3 {
                                            class: "text-lg font-semibold text-red-400",
                                            "Errors"
                                        }
                                    }
                                    div {
                                        class: "space-y-3",
                                        for error in &result.errors {
                                            div {
                                                class: "text-red-300 font-mono text-sm bg-red-500/5 p-3 rounded-lg border border-red-500/20",
                                                {error.clone()}
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
        }
    }
}
