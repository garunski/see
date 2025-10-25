//! Simple CLI tool to test the workflow engine

use ::engine::*;
use std::env;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <workflow_file>", args[0]);
        eprintln!("Example: {} examples/simple.json", args[0]);
        std::process::exit(1);
    }

    let workflow_file = &args[1];
    
    // Read workflow file
    let json = fs::read_to_string(workflow_file)
        .map_err(|e| format!("Failed to read file {}: {}", workflow_file, e))?;

    println!("🚀 Executing workflow from: {}", workflow_file);
    println!("📄 Workflow content:");
    println!("{}", json);
    println!();

    // Execute workflow
    match execute_workflow_from_json(&json).await {
        Ok(result) => {
            println!("✅ Workflow execution completed!");
            println!("📊 Results:");
            println!("  - Success: {}", result.success);
            println!("  - Workflow: {}", result.workflow_name);
            println!("  - Tasks: {}", result.tasks.len());
            println!("  - Errors: {}", result.errors.len());
            
            if !result.errors.is_empty() {
                println!("❌ Errors:");
                for error in &result.errors {
                    println!("  - {}", error);
                }
            }
            
            println!("\n📋 Task Details:");
            for task in &result.tasks {
                println!("  - {}: {:?}", task.name, task.status);
            }
            
            println!("\n📝 Audit Trail:");
            for entry in &result.audit_trail {
                println!("  - {}: {} ({:?})", entry.timestamp, entry.message, entry.status);
            }
        }
        Err(e) => {
            eprintln!("❌ Workflow execution failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
