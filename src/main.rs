use dataflow_rs::{Engine, Workflow};
use dataflow_rs::engine::{
    AsyncFunctionHandler,
    message::Message,
};
use serde_json::json;
use std::collections::HashMap;
use std::fs;

mod json_parser;
mod cli_handler;


#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Get workflow file from command line args or default to workflow.json
    let workflow_file = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "workflow.json".to_string());
    
    println!("Loading workflow from: {}", workflow_file);
    
    // Read workflow JSON
    let workflow_data = fs::read_to_string(&workflow_file)
        .map_err(|e| format!("Failed to read workflow file '{}': {}", workflow_file, e))?;
    
    // Parse workflow using dataflow-rs
    let workflow = Workflow::from_json(&workflow_data)
        .map_err(|e| format!("Failed to parse workflow: {}", e))?;
    
    println!("Loaded workflow: {}", workflow.name);
    println!("Number of tasks: {}", workflow.tasks.len());
    
    // Register custom function handler
    let mut custom_functions: HashMap<String, Box<dyn AsyncFunctionHandler + Send + Sync>> =
        HashMap::new();
    custom_functions.insert("cli_command".to_string(), Box::new(cli_handler::CliCommandHandler));
    
    // Create engine with the workflow and custom functions
    let engine = Engine::new(vec![workflow], Some(custom_functions));
    
    // Create initial message with empty context
    let mut message = Message::from_value(&json!({}));
    
    println!("\nExecuting workflow...\n");
    
    // Process the message through the workflow
    match engine.process_message(&mut message).await {
        Ok(_) => {
            println!("\n✅ Workflow execution complete!");
            
            // Display results
            println!("\n📊 Final Context:");
            println!("{}", serde_json::to_string_pretty(&message.context["data"])?);
            
            // Display audit trail
            println!("\n📋 Audit Trail:");
            for (i, audit) in message.audit_trail.iter().enumerate() {
                println!(
                    "{}. Task: {} (Status: {})",
                    i + 1,
                    audit.task_id,
                    audit.status
                );
                println!("   Timestamp: {}", audit.timestamp);
                println!("   Changes: {} field(s) modified", audit.changes.len());
            }
            
            // Check for errors
            if message.has_errors() {
                println!("\n⚠️  Errors encountered:");
                for error in &message.errors {
                    eprintln!(
                        "   - {}: {}",
                        error.task_id.as_ref().unwrap_or(&"unknown".to_string()),
                        error.message
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("\n❌ Workflow execution failed: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}
