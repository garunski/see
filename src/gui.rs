use dioxus::prelude::*;
use simple_workflow_app::{execute_workflow, OutputCallback, WorkflowResult};
use std::sync::Arc;
use tokio::sync::mpsc;
use rfd::FileDialog;

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
                    state.write().toast_message = Some("Workflow completed successfully!".to_string());
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
            class: "min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900 text-white",
            
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
                class: "min-h-screen flex",
                
                // Sidebar
                aside {
                    class: "w-80 bg-black/20 backdrop-blur-md border-r border-white/10 p-8",
                    
                    // Logo and title
                    div {
                        class: "mb-12",
                        div {
                            class: "flex items-center space-x-4 mb-4",
                            div {
                                class: "w-12 h-12 bg-gradient-to-r from-cyan-400 to-blue-500 rounded-2xl flex items-center justify-center text-2xl",
                                "⚡"
                            }
                            div {
                                h1 {
                                    class: "text-2xl font-bold bg-gradient-to-r from-cyan-400 to-blue-500 bg-clip-text text-transparent",
                                    "Workflow Executor"
                                }
                                p {
                                    class: "text-white/60 text-sm",
                                    "Execute and manage workflows"
                                }
                            }
                        }
                        
                        // Theme toggle
                        button {
                            class: "w-full p-3 bg-white/10 hover:bg-white/20 rounded-xl transition-all duration-200 flex items-center justify-center space-x-2",
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
                        class: "mb-8",
                        label {
                            class: "block text-sm font-medium text-white/80 mb-3",
                            "Workflow File"
                        }
                        div {
                            class: "space-y-3",
                            input {
                                class: "w-full px-4 py-3 bg-white/10 border border-white/20 rounded-xl text-white placeholder-white/50 focus:outline-none focus:ring-2 focus:ring-cyan-400 focus:border-transparent transition-all duration-200",
                                r#type: "text",
                                placeholder: "Select workflow file...",
                                value: state.read().workflow_file.clone(),
                                oninput: move |evt| {
                                    state.write().workflow_file = evt.value();
                                }
                            }
                            button {
                                class: "w-full p-3 bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-600 hover:to-blue-600 text-white rounded-xl font-medium transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center space-x-2",
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
                    
                    // Execute button
                    button {
                        class: "w-full p-4 bg-gradient-to-r from-emerald-500 to-teal-500 hover:from-emerald-600 hover:to-teal-600 text-white text-lg font-semibold rounded-xl shadow-lg hover:shadow-xl transform hover:scale-105 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none flex items-center justify-center space-x-3",
                        onclick: move |_| execute_workflow(),
                        disabled: matches!(state.read().execution_status, ExecutionStatus::Running),
                        if matches!(state.read().execution_status, ExecutionStatus::Running) {
                            div {
                                class: "animate-spin w-6 h-6 border-2 border-white border-t-transparent rounded-full"
                            }
                            span { "Executing..." }
                        } else {
                            div {
                                class: "w-6 h-6",
                                "🚀"
                            }
                            span { "Execute Workflow" }
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
                    class: "flex-1 p-8 overflow-y-auto",
                    
                    // Workflow info card
                    if let Some(ref result) = state.read().workflow_result {
                        div {
                            class: "mb-8 bg-white/10 backdrop-blur-md border border-white/20 rounded-2xl p-8 animate-fade-in",
                            div {
                                class: "flex items-center justify-between mb-6",
                                h2 {
                                    class: "text-2xl font-bold text-white",
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