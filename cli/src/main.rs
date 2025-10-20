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
    let args = Args::parse();
    let output: OutputCallback = std::sync::Arc::new(|line| println!("{}", line));
    match execute_workflow(&args.file, Some(output), None).await {
        Ok(result) => {
            println!(
                "Workflow '{}' completed: success={} tasks={}",
                result.workflow_name, result.success, result.task_count
            );
        }
        Err(e) => {
            eprintln!("Execution failed: {}", e);
            std::process::exit(1);
        }
    }
}
