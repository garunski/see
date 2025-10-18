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
    show_advanced: bool,
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
            show_advanced: false,
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
            class: if state.read().dark_mode { "dark" } else { "" },
            class: "min-h-screen bg-gray-50 dark:bg-gray-900 transition-colors duration-300",
            
            // Inject Tailwind CSS
            style {
                dangerous_inner_html: TAILWIND_CSS
            }
            
            // Toast notification
            if let Some(ref toast) = state.read().toast_message {
                div {
                    class: "fixed top-4 right-4 z-50 animate-slide-in",
                    div {
                        class: "bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg p-4 max-w-sm",
                        div {
                            class: "flex items-center justify-between",
                            span {
                                class: "text-gray-900 dark:text-white",
                                {toast.clone()}
                            }
                            button {
                                class: "ml-4 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300",
                                onclick: move |_| dismiss_toast(),
                                "✕"
                            }
                        }
                    }
                }
            }
            
            // Header
            header {
                class: "bg-gradient-to-r from-blue-600 to-purple-600 dark:from-blue-800 dark:to-purple-800 text-white shadow-lg",
                div {
                    class: "container mx-auto px-6 py-4",
                    div {
                        class: "flex items-center justify-between",
                        div {
                            class: "flex items-center space-x-3",
                            div {
                                class: "w-8 h-8 bg-white bg-opacity-20 rounded-lg flex items-center justify-center",
                                "⚡"
                            }
                            h1 {
                                class: "text-2xl font-bold",
                                "Workflow Executor"
                            }
                        }
                button {
                            class: "p-2 rounded-lg bg-white bg-opacity-20 hover:bg-opacity-30 transition-all duration-200",
                            onclick: move |_| toggle_dark_mode(),
                            if state.read().dark_mode { "Dark" } else { "Light" }
                        }
                    }
                }
            }
            
            // Main content
            main {
                class: "container mx-auto px-6 py-8",
                
                // File input section
                div {
                    class: "bg-white dark:bg-gray-800 rounded-xl shadow-lg p-6 mb-6",
                    label {
                        class: "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2",
                        "Workflow File"
                    }
                    div {
                        class: "flex space-x-3",
                        input {
                            class: "flex-1 px-4 py-3 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400 focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200",
                            r#type: "text",
                            placeholder: "Select or enter workflow file path",
                            value: state.read().workflow_file.clone(),
                            oninput: move |evt| {
                                state.write().workflow_file = evt.value();
                            }
                        }
                        button {
                            class: "px-6 py-3 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed flex items-center space-x-2",
                            disabled: state.read().is_picking_file,
                            onclick: move |_| pick_file(),
                            if state.read().is_picking_file {
                                "..."
                            } else {
                                "📁"
                            }
                            span { "Browse" }
                        }
                    }
                }
                
                // Execute button section
                div {
                    class: "text-center mb-8",
                    button {
                        class: "px-8 py-4 bg-gradient-to-r from-green-500 to-blue-500 hover:from-green-600 hover:to-blue-600 text-white text-lg font-semibold rounded-xl shadow-lg hover:shadow-xl transform hover:scale-105 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none flex items-center space-x-3 mx-auto",
                        onclick: move |_| execute_workflow(),
                        disabled: matches!(state.read().execution_status, ExecutionStatus::Running),
                        if matches!(state.read().execution_status, ExecutionStatus::Running) {
                            div {
                                class: "animate-spin w-5 h-5 border-2 border-white border-t-transparent rounded-full"
                            }
                            span { "Executing..." }
                        } else {
                            "Execute Workflow"
                        }
                    }
                    
                    // Status badge
                    if !matches!(state.read().execution_status, ExecutionStatus::Idle) {
                        div {
                            class: "mt-4",
                            if matches!(state.read().execution_status, ExecutionStatus::Running) {
                                div {
                                    class: "inline-flex items-center px-4 py-2 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded-full text-sm font-medium animate-pulse",
                                    "Running"
                                }
                            } else if matches!(state.read().execution_status, ExecutionStatus::Complete) {
                                div {
                                    class: "inline-flex items-center px-4 py-2 bg-green-100 dark:bg-green-900 text-green-800 dark:text-green-200 rounded-full text-sm font-medium animate-fade-in",
                                    "Complete"
                                }
                            } else if matches!(state.read().execution_status, ExecutionStatus::Failed) {
                                div {
                                    class: "inline-flex items-center px-4 py-2 bg-red-100 dark:bg-red-900 text-red-800 dark:text-red-200 rounded-full text-sm font-medium animate-fade-in",
                                    "Failed"
                                }
                            }
                        }
                    }
                }
                
                // Workflow info card
                if let Some(ref result) = state.read().workflow_result {
                    div {
                        class: "bg-white dark:bg-gray-800 rounded-xl shadow-lg p-6 mb-6 animate-fade-in",
                        div {
                            class: "flex items-center justify-between mb-4",
                            h3 {
                                class: "text-xl font-semibold text-gray-900 dark:text-white",
                                "Workflow Information"
                            }
                            div {
                                class: "flex items-center space-x-2",
                                if result.success {
                                    div {
                                        class: "w-6 h-6 bg-green-500 rounded-full flex items-center justify-center text-white text-sm",
                                        "OK"
                                    }
                                } else {
                                    div {
                                        class: "w-6 h-6 bg-red-500 rounded-full flex items-center justify-center text-white text-sm",
                                        "X"
                                    }
                                }
                            }
                        }
                        div {
                            class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                            div {
                                class: "bg-gray-50 dark:bg-gray-700 rounded-lg p-4",
                                div {
                                    class: "text-sm text-gray-600 dark:text-gray-400 mb-1",
                                    "Name"
                                }
                                div {
                                    class: "font-medium text-gray-900 dark:text-white",
                                    {result.workflow_name.clone()}
                                }
                            }
                            div {
                                class: "bg-gray-50 dark:bg-gray-700 rounded-lg p-4",
                                div {
                                    class: "text-sm text-gray-600 dark:text-gray-400 mb-1",
                                    "Tasks"
                                }
                                div {
                                    class: "font-medium text-gray-900 dark:text-white",
                                    {result.task_count.to_string()}
                                }
                            }
                            div {
                                class: "bg-gray-50 dark:bg-gray-700 rounded-lg p-4",
                                div {
                                    class: "text-sm text-gray-600 dark:text-gray-400 mb-1",
                                    "Status"
                                }
                                div {
                                    class: "font-medium text-gray-900 dark:text-white",
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
                            class: "bg-white dark:bg-gray-800 rounded-xl shadow-lg overflow-hidden",
                            button {
                                class: "w-full px-6 py-4 text-left flex items-center justify-between hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors duration-200",
                                onclick: move |_| {
                                    let current_show_logs = state.read().show_logs;
                                    state.write().show_logs = !current_show_logs;
                                },
                                div {
                                    class: "flex items-center space-x-3",
                                    "Logs"
                                    span {
                                        class: "font-semibold text-gray-900 dark:text-white",
                                        "Execution Output"
                                    }
                                    span {
                                        class: "text-sm text-gray-500 dark:text-gray-400",
                                        "({state.read().output_logs.len()} lines)"
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
                                    class: "border-t border-gray-200 dark:border-gray-700",
                                    div {
                                        class: "bg-black text-green-400 p-4 font-mono text-sm max-h-80 overflow-y-auto",
                                        {state.read().output_logs.join("\n")}
                                    }
                                    div {
                                        class: "px-6 py-3 bg-gray-50 dark:bg-gray-700 flex justify-end",
                                        button {
                                            class: "px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm font-medium transition-colors duration-200",
                                            onclick: move |_| {
                                                println!("Copy output to clipboard");
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
                            class: "bg-white dark:bg-gray-800 rounded-xl shadow-lg overflow-hidden",
                            button {
                                class: "w-full px-6 py-4 text-left flex items-center justify-between hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors duration-200",
                                onclick: move |_| {
                                    let current_show_context = state.read().show_context;
                                    state.write().show_context = !current_show_context;
                                },
                                div {
                                    class: "flex items-center space-x-3",
                                    "Data"
                                    span {
                                        class: "font-semibold text-gray-900 dark:text-white",
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
                                    class: "border-t border-gray-200 dark:border-gray-700",
                                    div {
                                        class: "bg-gray-50 dark:bg-gray-700 p-4",
                                        pre {
                                            class: "text-sm text-gray-800 dark:text-gray-200 whitespace-pre-wrap overflow-x-auto",
                                            {serde_json::to_string_pretty(&result.final_context).unwrap_or_else(|_| "{}".to_string())}
                                        }
                                    }
                                    div {
                                        class: "px-6 py-3 bg-gray-50 dark:bg-gray-700 flex justify-end",
                                        button {
                                            class: "px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm font-medium transition-colors duration-200",
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
                                class: "bg-red-50 dark:bg-red-900 border border-red-200 dark:border-red-700 rounded-xl p-6 animate-fade-in",
                                div {
                                    class: "flex items-center space-x-3 mb-4",
                                    "Error"
                                    h3 {
                                        class: "text-lg font-semibold text-red-800 dark:text-red-200",
                                        "Errors"
                                    }
                                }
                                div {
                                    class: "space-y-2",
                                    for error in &result.errors {
                                        div {
                                            class: "text-red-700 dark:text-red-300 font-mono text-sm",
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