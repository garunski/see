use serde::Deserialize;
use std::process::Command;
use std::fs;

#[derive(Deserialize, Debug)]
struct Step {
    id: String,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    step_type: String,
    command: String,
    args: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct Workflow {
    steps: Vec<Step>,
}

fn execute_cli_step(step: &Step) -> Result<String, Box<dyn std::error::Error>> {
    println!("Executing step: {}", step.id);
    println!("Command: {} {:?}", step.command, step.args);
    
    let output = Command::new(&step.command)
        .args(&step.args)
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    if !stdout.is_empty() {
        println!("Output: {}", stdout);
    }
    
    if !stderr.is_empty() {
        println!("Error: {}", stderr);
    }
    
    if !output.status.success() {
        return Err(format!("Command failed with exit code: {:?}", output.status.code()).into());
    }
    
    Ok(stdout.to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read and parse workflow JSON
    let workflow_data = fs::read_to_string("workflow.json")?;
    let workflow: Workflow = serde_json::from_str(&workflow_data)?;
    
    println!("Loaded workflow with {} steps", workflow.steps.len());
    
    // Execute the first step
    if let Some(first_step) = workflow.steps.first() {
        match execute_cli_step(first_step) {
            Ok(output) => {
                println!("Step '{}' completed successfully", first_step.id);
                println!("Final output: {}", output);
            }
            Err(e) => {
                eprintln!("Step '{}' failed: {}", first_step.id, e);
                return Err(e);
            }
        }
    } else {
        println!("No steps found in workflow");
    }
    
    println!("Workflow execution complete!");
    Ok(())
}
