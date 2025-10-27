use clap::{Parser, Subcommand};
use s_e_e_core::{
    execute_workflow_by_id, init_global_store, populate_initial_data, OutputCallback,
};
use std::fs;

#[derive(Parser, Debug)]
#[command(name = "s_e_e_cli", version, about = "Run workflows")]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    // Legacy support for workflow execution
    #[arg(short, long)]
    file: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List all system workflows
    #[command(name = "list-system-workflows")]
    ListSystemWorkflows,

    /// List all system prompts
    #[command(name = "list-system-prompts")]
    ListSystemPrompts,

    /// Clone a system workflow to create a user workflow
    #[command(name = "clone-workflow")]
    CloneWorkflow {
        #[arg(short, long)]
        system_id: String,
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Clone a system prompt to create a user prompt
    #[command(name = "clone-prompt")]
    ClonePrompt {
        #[arg(short, long)]
        system_id: String,
        #[arg(short, long)]
        name: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let _tracing_guard = s_e_e_core::init_tracing(None)
        .map_err(|e| format!("Failed to initialize tracing: {}", e))
        .expect("Failed to initialize tracing");

    let args = Args::parse();

    // Handle subcommands
    if let Some(command) = args.command {
        handle_command(command).await;
        return;
    }

    // Legacy workflow execution
    if let Some(file) = args.file {
        tracing::info!(file = %file, "CLI starting");
        execute_workflow_from_file(file).await;
    } else {
        eprintln!("No command or workflow file specified. Use --help for usage.");
        std::process::exit(1);
    }
}

async fn handle_command(command: Commands) {
    // Initialize the global store with a local database
    if let Err(e) = init_global_store().await {
        tracing::error!(error = %e, "Failed to initialize global store");
        eprintln!("Failed to initialize database: {}", e);
        std::process::exit(1);
    }

    // Populate initial data if needed
    if let Err(e) = populate_initial_data().await {
        tracing::error!("Failed to populate initial data: {}", e);
        eprintln!("Failed to populate initial data: {}", e);
    }

    match command {
        Commands::ListSystemWorkflows => {
            if let Ok(store) = s_e_e_core::get_global_store() {
                match store.list_workflows().await {
                    Ok(workflows) => {
                        println!("Workflows ({}):", workflows.len());
                        for workflow in workflows {
                            println!("  - {}", workflow.name);
                            if let Some(desc) = &workflow.description {
                                println!("    {}", desc);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to list workflows: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::ListSystemPrompts => {
            if let Ok(store) = s_e_e_core::get_global_store() {
                match store.list_prompts().await {
                    Ok(prompts) => {
                        println!("Prompts ({}):", prompts.len());
                        for prompt in prompts {
                            println!("  - {}", prompt.name);
                            if let Some(desc) = &prompt.description {
                                println!("    {}", desc);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to list prompts: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::CloneWorkflow {
            system_id: _,
            name: _,
        } => {
            eprintln!("Clone command no longer needed - all workflows are editable");
            std::process::exit(1);
        }
        Commands::ClonePrompt {
            system_id: _,
            name: _,
        } => {
            eprintln!("Clone command no longer needed - all prompts are editable");
            std::process::exit(1);
        }
    }
}

async fn execute_workflow_from_file(file: String) {
    // Initialize the global store with a local database
    if let Err(e) = init_global_store().await {
        tracing::error!(error = %e, "Failed to initialize global store");
        eprintln!("Failed to initialize database: {}", e);
        std::process::exit(1);
    }

    // Read workflow file
    let workflow_content = match fs::read_to_string(&file) {
        Ok(content) => content,
        Err(e) => {
            tracing::error!(error = %e, file = %file, "Failed to read workflow file");
            eprintln!("Failed to read workflow file '{}': {}", file, e);
            std::process::exit(1);
        }
    };

    // Parse workflow JSON to get the ID
    let workflow_json: serde_json::Value = match serde_json::from_str(&workflow_content) {
        Ok(json) => json,
        Err(e) => {
            tracing::error!(error = %e, file = %file, "Failed to parse workflow JSON");
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
