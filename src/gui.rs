use dioxus::prelude::*;
use simple_workflow_app::{execute_workflow, OutputCallback, WorkflowResult};
use std::sync::Arc;
use tokio::sync::mpsc;

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
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            workflow_file: "workflow.json".to_string(),
            execution_status: ExecutionStatus::Idle,
            workflow_result: None,
            output_logs: Vec::new(),
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
                }
                Err(e) => {
                    state.write().execution_status = ExecutionStatus::Failed;
                    state.write().output_logs.push(format!("Error: {}", e));
                }
            }
        });
    };

    rsx! {
        div {
            style: "padding: 20px; font-family: system-ui, sans-serif;",
            
            h1 { "Workflow Executor" }
            
            div {
                style: "margin-bottom: 20px;",
                
                label {
                    style: "display: block; margin-bottom: 5px; font-weight: bold;",
                    "Workflow File:"
                }
                
                input {
                    style: "width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;",
                    r#type: "text",
                    value: state.read().workflow_file.clone(),
                    oninput: move |evt| {
                        state.write().workflow_file = evt.value();
                    }
                }
            }
            
            div {
                style: "margin-bottom: 20px;",
                
                button {
                    style: "padding: 10px 20px; background: #28a745; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 16px;",
                    onclick: move |_| execute_workflow(),
                    disabled: matches!(state.read().execution_status, ExecutionStatus::Running),
                    "Execute Workflow"
                }
                
                if matches!(state.read().execution_status, ExecutionStatus::Running) {
                    span {
                        style: "margin-left: 10px; color: #007bff;",
                        "⏳ Running..."
                    }
                }
            }
            
            // Workflow info
            if let Some(ref result) = state.read().workflow_result {
                div {
                    style: "margin-bottom: 20px; padding: 15px; background: #f8f9fa; border-radius: 4px;",
                    
                    h3 { "Workflow Information" }
                    p { "Name: " {result.workflow_name.clone()} }
                    p { "Tasks: " {result.task_count.to_string()} }
                    p { "Status: " {if result.success { "✅ Success" } else { "❌ Failed" }} }
                }
            }
            
            // Output logs
            if !state.read().output_logs.is_empty() {
                div {
                    style: "margin-bottom: 20px;",
                    
                    h3 { "Execution Output" }
                    div {
                        style: "background: #000; color: #0f0; padding: 15px; border-radius: 4px; font-family: monospace; white-space: pre-wrap; max-height: 300px; overflow-y: auto;",
                        {state.read().output_logs.join("\n")}
                    }
                }
            }
            
            // Final context
            if let Some(ref result) = state.read().workflow_result {
                div {
                    style: "margin-bottom: 20px;",
                    
                    h3 { "Final Context" }
                    div {
                        style: "background: #f8f9fa; padding: 15px; border-radius: 4px;",
                        pre {
                            style: "white-space: pre-wrap; font-family: monospace; font-size: 12px;",
                            {serde_json::to_string_pretty(&result.final_context).unwrap_or_else(|_| "{}".to_string())}
                        }
                    }
                }
            }
            
            // Errors
            if let Some(ref result) = state.read().workflow_result {
                if !result.errors.is_empty() {
                    div {
                        style: "margin-bottom: 20px;",
                        
                        h3 { "Errors" }
                        div {
                            style: "background: #f8d7da; color: #721c24; padding: 15px; border-radius: 4px; border: 1px solid #f5c6cb;",
                            {result.errors.join("\n")}
                        }
                    }
                }
            }
        }
    }
}