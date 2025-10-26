# CLI Specification - User Input

## Overview

This document specifies CLI changes for user input support, including new commands and interactive input handling.

## Commands

### workflow list

**Syntax**: `s_e_e_cli workflow list`

**Description**: List all available workflows

**Output**:
```
Available Workflows:
  - simple-input (Simple User Input Workflow)
  - parallel-input (Parallel User Input Workflow)
  - nested-input (Nested User Input Workflow)
```

### workflow start

**Syntax**: `s_e_e_cli workflow start <workflow_id>`

**Description**: Start a workflow execution with interactive input support

**Behavior**:
- Loads workflow from database
- Executes workflow through engine
- When task requires input:
  - Pause execution
  - Display prompt
  - Accept input from stdin
  - Validate input
  - Resume execution automatically
- Show progress throughout execution

**Example Session**:
```
$ s_e_e_cli workflow start simple-input

Starting workflow: Simple User Input Workflow
[Task 1/3] Display Greeting: Complete
[Task 2/3] Get User Name: Waiting for input...

Please enter your name:
> John Doe

Input received: John Doe
[Task 2/3] Get User Name: Complete
[Task 3/3] Thank You: Complete

Workflow completed successfully!
```

### workflow resume

**Syntax**: `s_e_e_cli workflow resume <execution_id>`

**Description**: Resume a paused workflow execution

**Behavior**:
- Loads execution from database
- Finds tasks waiting for input
- Prompts for each input in order
- Submits inputs and continues execution

**Example Session**:
```
$ s_e_e_cli workflow resume exec-123

Resuming workflow: exec-123

Pending inputs:
  1. Task "Get User Name" (required)

Please enter your name:
> Jane Smith

Resuming execution...
[Task 2/3] Get User Name: Complete
[Task 3/3] Thank You: Complete

Workflow completed successfully!
```

## Interactive Input Mode

### Input Prompt

When a task requires input, display:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Task: Get User Name
Status: Waiting for Input

Prompt: Please enter your name
Type: string
Required: true
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Enter input:
> 
```

### Input Validation

**String**: Any string is valid
**Number**: Must parse as number (integer or float)
**Boolean**: Must be "true", "false", "1", "0", "yes", "no" (case insensitive)

**Validation Error Display**:
```
❌ Invalid input: Expected number, got "abc"

Enter input:
> 
```

### Retry Logic

If input validation fails, allow retry:
```
Enter input:
> abc

❌ Invalid input: Expected number, got "abc"
Retry? (y/n): y

Enter input:
> 123
✓ Input accepted
```

## Implementation

### File Structure

```
cli/
├── src/
│   ├── main.rs
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── list.rs
│   │   ├── start.rs
│   │   └── resume.rs
│   └── input/
│       ├── mod.rs
│       ├── prompt.rs
│       └── validator.rs
```

### Main Entry Point

**File**: `cli/src/main.rs`

```rust
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "s_e_e_cli", version, about)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Workflow {
        #[command(subcommand)]
        command: WorkflowCommands,
    },
}

#[derive(Subcommand, Debug)]
enum WorkflowCommands {
    List,
    Start { workflow_id: String },
    Resume { execution_id: String },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    match args.command {
        Commands::Workflow { command } => {
            match command {
                WorkflowCommands::List => commands::list::execute().await,
                WorkflowCommands::Start { workflow_id } => commands::start::execute(workflow_id).await,
                WorkflowCommands::Resume { execution_id } => commands::resume::execute(execution_id).await,
            }
        }
    }
}
```

### List Command

**File**: `cli/src/commands/list.rs`

```rust
pub async fn execute() {
    // ... initialize store ...
    let workflows = store.list_workflows().await?;
    
    println!("Available Workflows:");
    for workflow in workflows {
        println!("  - {} ({})", workflow.id, workflow.name);
    }
}
```

### Start Command

**File**: `cli/src/commands/start.rs`

```rust
pub async fn execute(workflow_id: String) {
    // Initialize store
    s_e_e_core::init_global_store().await?;
    
    // Start workflow with callback
    let callback = Arc::new(|message| {
        println!("{}", message);
    });
    
    let result = s_e_e_core::execute_workflow_by_id(
        &workflow_id,
        Some(callback),
    ).await?;
    
    // Check if waiting for input
    if result.errors.contains(&"Waiting for user input".to_string()) {
        handle_pending_inputs(&result.execution_id).await?;
    }
    
    println!("Workflow completed!");
}
```

### Resume Command

**File**: `cli/src/commands/resume.rs`

```rust
pub async fn execute(execution_id: String) {
    // Get pending inputs
    let inputs = s_e_e_core::get_pending_inputs(&execution_id).await?;
    
    if inputs.is_empty() {
        println!("No pending inputs for this execution");
        return;
    }
    
    // Prompt for each input
    for input in inputs {
        let value = input::prompt::get_input(&input).await?;
        
        // Submit input
        s_e_e_core::provide_user_input(
            &execution_id,
            &input.task_execution_id,
            value,
        ).await?;
    }
    
    println!("Workflow resumed successfully!");
}
```

### Input Prompt Module

**File**: `cli/src/input/prompt.rs`

```rust
use s_e_e_core::UserInputRequest;

pub async fn get_input(request: &UserInputRequest) -> Result<String, String> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Task: {}", request.task_execution_id);
    println!("Status: Waiting for Input");
    println!();
    println!("Prompt: {}", request.prompt_text);
    println!("Type: {}", request.input_type);
    println!("Required: {}", request.required);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    print!("Enter input: ");
    std::io::Write::flush(&mut std::io::stdout())?;
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    Ok(input.trim().to_string())
}
```

### Input Validator Module

**File**: `cli/src/input/validator.rs`

```rust
pub fn validate(value: &str, input_type: &str) -> Result<(), String> {
    match input_type {
        "string" => Ok(()),
        "number" => {
            value.parse::<f64>()
                .map(|_| ())
                .map_err(|_| format!("Invalid number: {}", value))
        }
        "boolean" => {
            match value.to_lowercase().as_str() {
                "true" | "false" | "1" | "0" | "yes" | "no" => Ok(()),
                _ => Err(format!("Invalid boolean: {}", value)),
            }
        }
        _ => Err(format!("Unknown input type: {}", input_type)),
    }
}
```

## Testing Requirements

### Test File

**File**: `cli/tests/cli_input_tests.rs`

```rust
#[tokio::test]
async fn test_workflow_list() {
    // Test list command
}

#[tokio::test]
async fn test_workflow_start_with_input() {
    // Test start with input task
}

#[tokio::test]
async fn test_workflow_resume() {
    // Test resume command
}

#[tokio::test]
async fn test_input_validation() {
    // Test input validation
}
```

## Logging Requirements

All commands must log:
- Command start/completion
- Input validation results
- State transitions
- Errors

## Error Handling

Display user-friendly errors:
```rust
println!("❌ Error: {}", error_message);
println!("Please try again or use 'help' for assistance");
```

