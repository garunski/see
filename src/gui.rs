use dark_light::Mode;
use dioxus::prelude::*;
use rfd::FileDialog;
use simple_workflow_app::{execute_workflow, OutputCallback, WorkflowResult};
use std::sync::Arc;
use tokio::sync::mpsc;

use simple_workflow_app::components::{
    ContextPanel, ErrorsPanel, ExecutionStatus, OutputLogsPanel, Sidebar, Toast, WorkflowInfoCard,
};

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
        // Detect system theme preference
        let dark_mode = match dark_light::detect() {
            Mode::Dark => true,
            Mode::Light => false,
        };

        Self {
            workflow_file: "workflow.json".to_string(),
            execution_status: ExecutionStatus::Idle,
            workflow_result: None,
            output_logs: Vec::new(),
            dark_mode,
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

    // Create a derived signal for workflow_result that components can use
    let workflow_result_signal = use_memo(move || state.read().workflow_result.clone());

    // Create a derived signal for dark mode
    let dark_mode_signal = use_memo(move || state.read().dark_mode);

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
        document::Stylesheet {
            href: asset!("/assets/tailwind.css")
        }

        div {
            class: format!("min-h-screen bg-white dark:bg-zinc-900 text-zinc-950 dark:text-white {}",
                if *dark_mode_signal.read() { "dark" } else { "" }),

            // Toast notification
            Toast {
                message: state.read().toast_message.clone(),
                on_dismiss: move |_| dismiss_toast(),
            }

            // Main layout
            div {
                class: "flex flex-col lg:flex-row min-h-svh w-full bg-white lg:bg-zinc-100 dark:bg-zinc-900 dark:lg:bg-zinc-950",

                // Sidebar
                Sidebar {
                    workflow_file: state.read().workflow_file.clone(),
                    on_workflow_file_change: move |value| state.write().workflow_file = value,
                    is_picking_file: state.read().is_picking_file,
                    on_pick_file: move |_| pick_file(),
                    dark_mode: *dark_mode_signal.read(),
                    on_toggle_dark_mode: move |_| toggle_dark_mode(),
                    execution_status: state.read().execution_status.clone(),
                    on_execute: move |_| execute_workflow(),
                }

                // Main content area
                main {
                    class: "flex flex-1 flex-col pb-2 lg:min-w-0 lg:pt-2 lg:pr-2",
                    div {
                        class: "grow p-6 lg:rounded-lg lg:bg-white lg:p-10 lg:shadow-xs lg:ring-1 lg:ring-zinc-950/5 dark:lg:bg-zinc-900 dark:lg:ring-white/10",
                        div {
                            class: "mx-auto max-w-6xl",

                            // Workflow info card
                            if let Some(result) = workflow_result_signal.read().clone() {
                                WorkflowInfoCard {
                                    result: ReadOnlySignal::new(Signal::new(result)),
                                }
                            }

                            // Collapsible sections
                            div {
                                class: "space-y-6",

                                // Output logs section
                                OutputLogsPanel {
                                    logs: state.read().output_logs.clone(),
                                    show_logs: state.read().show_logs,
                                    on_toggle: move |_| {
                                        let current = state.read().show_logs;
                                        state.write().show_logs = !current;
                                    },
                                    on_copy: move |text| copy_to_clipboard(text),
                                }

                                // Final context section
                                if let Some(ref result) = state.read().workflow_result {
                                    ContextPanel {
                                        context: result.final_context.clone(),
                                        show_context: state.read().show_context,
                                        on_toggle: move |_| {
                                            let current = state.read().show_context;
                                            state.write().show_context = !current;
                                        },
                                        on_copy: move |_| {
                                            println!("Copy context to clipboard");
                                        },
                                    }
                                }

                                // Errors section
                                if let Some(ref result) = state.read().workflow_result {
                                    ErrorsPanel {
                                        errors: result.errors.clone(),
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
