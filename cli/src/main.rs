use clap::Parser;
use s_e_e_core::{execute_workflow_by_id, init_global_store, OutputCallback};
use std::fs;

#[derive(Parser, Debug)]
#[command(name = "s_e_e_cli", version, about = "Run workflows")]
struct Args {
    #[arg(short, long, default_value = "workflow.json")]
    file: String,
}

#[tokio::main]
async fn main() {
    let _tracing_guard = s_e_e_core::init_tracing(None)
        .map_err(|e| format!("Failed to initialize tracing: {}", e))
        .expect("Failed to initialize tracing");

    let args = Args::parse();
    tracing::info!(file = %args.file, "CLI starting");

    // Initialize the global store with a local database
    if let Err(e) = init_global_store().await {
        tracing::error!(error = %e, "Failed to initialize global store");
        eprintln!("Failed to initialize database: {}", e);
        std::process::exit(1);
    }

    // Read workflow file
    let workflow_content = match fs::read_to_string(&args.file) {
        Ok(content) => content,
        Err(e) => {
            tracing::error!(error = %e, file = %args.file, "Failed to read workflow file");
            eprintln!("Failed to read workflow file '{}': {}", args.file, e);
            std::process::exit(1);
        }
    };

    // Parse workflow JSON to get the ID
    let workflow_json: serde_json::Value = match serde_json::from_str(&workflow_content) {
        Ok(json) => json,
        Err(e) => {
            tracing::error!(error = %e, file = %args.file, "Failed to parse workflow JSON");
            eprintln!("Failed to parse workflow JSON: {}", e);
            std::process::exit(1);
        }
    };

    let workflow_id = workflow_json
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("default");

    // Save workflow to persistence
    let store = s_e_e_core::get_global_store()
        .map_err(|e| format!("Failed to get global store: {}", e))
        .expect("Failed to get global store");

    let workflow_definition = s_e_e_core::WorkflowDefinition {
        id: workflow_id.to_string(),
        name: workflow_json
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unnamed Workflow")
            .to_string(),
        description: workflow_json
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        content: workflow_content,
        is_default: false,
        is_edited: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    if let Err(e) = store.save_workflow(&workflow_definition).await {
        tracing::error!(error = %e, workflow_id = %workflow_id, "Failed to save workflow");
        eprintln!("Failed to save workflow: {}", e);
        std::process::exit(1);
    }

    // Execute the workflow
    let output: OutputCallback = std::sync::Arc::new(|line| println!("{}", line));
    match execute_workflow_by_id(workflow_id, Some(output)).await {
        Ok(result) => {
            tracing::info!(
                workflow = %result.workflow_name,
                success = result.success,
                task_count = result.tasks.len(),
                "Workflow completed"
            );
            println!(
                "Workflow '{}' completed: success={} tasks={}",
                result.workflow_name,
                result.success,
                result.tasks.len()
            );
        }
        Err(e) => {
            tracing::error!(error = %e, "Workflow execution failed");
            eprintln!("Execution failed: {}", e);
            std::process::exit(1);
        }
    }
}
