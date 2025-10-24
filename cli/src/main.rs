use clap::Parser;
use s_e_e_core::{execute_workflow, OutputCallback};

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

    let output: OutputCallback = std::sync::Arc::new(|line| println!("{}", line));
    match execute_workflow(&args.file, Some(output)).await {
        Ok(result) => {
            tracing::info!(
                workflow = %result.workflow_name,
                success = result.success,
                task_count = result.task_count,
                "Workflow completed"
            );
            println!(
                "Workflow '{}' completed: success={} tasks={}",
                result.workflow_name, result.success, result.task_count
            );
        }
        Err(e) => {
            tracing::error!(error = %e, "Workflow execution failed");
            eprintln!("Execution failed: {}", e);
            std::process::exit(1);
        }
    }
}
