use clap::Parser;
use see_core::{execute_workflow, OutputCallback};

#[derive(Parser, Debug)]
#[command(name = "see-cli", version, about = "Run workflows")]
struct Args {
    /// Path to workflow JSON file
    #[arg(short, long, default_value = "workflow.json")]
    file: String,
}

#[tokio::main]
async fn main() {
    // CRITICAL: Keep guard alive for entire program lifetime
    let _tracing_guard = see_core::init_tracing(None).expect("Failed to initialize tracing");

    let args = Args::parse();
    tracing::info!(file = %args.file, "CLI starting");

    let output: OutputCallback = std::sync::Arc::new(|line| println!("{}", line));
    match execute_workflow(&args.file, Some(output), None).await {
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
    // _tracing_guard dropped here, flushing logs
}
