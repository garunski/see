use simple_workflow_app::{execute_workflow, OutputCallback};
use std::sync::Arc;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Get workflow file from command line args or default to workflow.json
    let workflow_file = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "workflow.json".to_string());
    
    // Create output callback that prints to stdout
    let output_callback: OutputCallback = Arc::new(|msg| {
        print!("{}", msg);
    });
    
    // Execute the workflow
    let result = execute_workflow(&workflow_file, Some(output_callback)).await?;
    
    // Display workflow info
    println!("Loaded workflow: {}", result.workflow_name);
    println!("Number of tasks: {}", result.task_count);
    
    // Display final context
    println!("\n📊 Final Context:");
    println!("{}", serde_json::to_string_pretty(&result.final_context)?);
    
    // Display audit trail
    println!("\n📋 Audit Trail:");
    for (i, audit) in result.audit_trail.iter().enumerate() {
        println!(
            "{}. Task: {} (Status: {})",
            i + 1,
            audit.task_id,
            audit.status
        );
        println!("   Timestamp: {}", audit.timestamp);
        println!("   Changes: {} field(s) modified", audit.changes_count);
    }
    
    // Display errors if any
    if !result.errors.is_empty() {
        println!("\n⚠️  Errors encountered:");
        for error in &result.errors {
            eprintln!("   - {}", error);
        }
    }
    
    Ok(())
}