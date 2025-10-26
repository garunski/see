# Engine Layer Specification - User Input

## Overview

This document specifies the engine layer changes for user input support, including the new UserInput handler, engine modifications, and resume functionality.

## UserInput Task Function

### TaskFunction Extension

**File**: `engine/src/types.rs`

**New Variant**:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskFunction {
    #[serde(rename = "cli_command")]
    CliCommand { command: String, args: Vec<String> },
    
    #[serde(rename = "cursor_agent")]
    CursorAgent { prompt: String, config: Value },
    
    #[serde(rename = "custom")]
    Custom { name: String, input: Value },
    
    // NEW
    #[serde(rename = "user_input")]
    UserInput {
        prompt: String,
        input_type: String,
        required: bool,
        default: Option<Value>,
    },
}
```

**Example Workflow JSON**:
```json
{
    "id": "workflow-with-input",
    "name": "Workflow with User Input",
    "tasks": [
        {
            "id": "get-name",
            "name": "Get User Name",
            "function": {
                "user_input": {
                    "prompt": "Please enter your name",
                    "input_type": "string",
                    "required": true,
                    "default": null
                }
            },
            "next_tasks": []
        }
    ]
}
```

## UserInputHandler

### New Handler Module

**File**: `engine/src/handlers/user_input.rs`

```rust
use async_trait::async_trait;
use crate::errors::*;
use crate::handlers::TaskHandler;
use crate::types::*;
use tracing::{debug, info, warn, error};

pub struct UserInputHandler;

#[async_trait]
impl TaskHandler for UserInputHandler {
    async fn execute(
        &self,
        context: &mut ExecutionContext,
        task: &EngineTask,
    ) -> Result<TaskResult, HandlerError> {
        info!(
            execution_id = %context.execution_id,
            task_id = %task.id,
            task_name = %task.name,
            "Executing user input task"
        );

        // Extract input configuration
        let config = match &task.function {
            TaskFunction::UserInput { prompt, input_type, required, default } => {
                UserInputConfig {
                    prompt: prompt.clone(),
                    input_type: input_type.clone(),
                    required: *required,
                    default: default.clone(),
                }
            }
            _ => return Err(HandlerError::InvalidConfig("Not a user input task".to_string())),
        };

        // Mark task as waiting for input
        context.update_task_status(task.id.clone(), TaskStatus::WaitingForInput);
        
        debug!(
            execution_id = %context.execution_id,
            task_id = %task.id,
            "Task marked as WaitingForInput"
        );

        // Log the request
        context.log(format!("Waiting for user input: {}", config.prompt));
        context.log_task(task.id.clone(), format!("Requesting input: {}", config.prompt));

        // Return special result indicating input required
        Ok(TaskResult {
            success: true,  // Not a failure - just waiting
            output: serde_json::json!({
                "waiting_for_input": true,
                "prompt": config.prompt,
                "input_type": config.input_type,
                "required": config.required,
                "default": config.default,
            }),
            error: None,
        })
    }
}

struct UserInputConfig {
    prompt: String,
    input_type: String,
    required: bool,
    default: Option<Value>,
}
```

### Handler Registration

**File**: `engine/src/handlers/mod.rs`

```rust
pub mod cli_command;
pub mod cursor_agent;
pub mod custom;
pub mod user_input; // NEW

impl HandlerRegistry {
    pub fn new() -> Self {
        debug!("Creating new handler registry");
        let mut handlers: HashMap<String, Box<dyn TaskHandler>> = HashMap::new();

        // ... existing handlers ...

        trace!("Registering user input handler");
        handlers.insert(
            "user_input".to_string(),
            Box::new(user_input::UserInputHandler),
        );

        // ...
    }
}

/// Get the function type from a task
pub fn get_function_type(task: &EngineTask) -> &'static str {
    let function_type = match &task.function {
        TaskFunction::CliCommand { .. } => "cli_command",
        TaskFunction::CursorAgent { .. } => "cursor_agent",
        TaskFunction::Custom { .. } => "custom",
        TaskFunction::UserInput { .. } => "user_input", // NEW
    };
    // ...
}
```

## Engine Execution Loop Modifications

### Handle WaitingForInput Status

**File**: `engine/src/engine.rs`

**Modified Method**: `get_ready_tasks_from_tree()`

```rust
fn get_ready_tasks_from_tree(
    &self,
    root_tasks: &[EngineTask],
    completed_tasks: &HashSet<String>,
    waiting_for_input: &HashSet<String>, // NEW parameter
) -> Vec<EngineTask> {
    let mut ready_tasks = Vec::new();

    fn collect_ready_tasks(
        tasks: &[EngineTask],
        completed_tasks: &HashSet<String>,
        waiting_for_input: &HashSet<String>, // NEW
        ready_tasks: &mut Vec<EngineTask>,
    ) {
        for task in tasks {
            // Skip if already completed
            if completed_tasks.contains(&task.id) {
                collect_ready_tasks(&task.next_tasks, completed_tasks, waiting_for_input, ready_tasks);
                continue;
            }
            
            // Skip if waiting for input
            if waiting_for_input.contains(&task.id) {
                debug!("Task {} skipped - waiting for input", task.id);
                continue;
            }
            
            // This task is ready to execute
            ready_tasks.push(task.clone());
        }
    }

    collect_ready_tasks(root_tasks, completed_tasks, waiting_for_input, &mut ready_tasks);
    ready_tasks
}
```

**Modified Method**: `execute_workflow()`

```rust
pub async fn execute_workflow(
    &self,
    workflow: EngineWorkflow,
) -> Result<WorkflowResult, EngineError> {
    // ... existing setup ...

    // Track execution state
    let mut completed_tasks = HashSet::new();
    let mut waiting_for_input = HashSet::new(); // NEW
    let mut audit_trail = Vec::new();
    let mut errors = Vec::new();
    let mut execution_round = 0;

    loop {
        execution_round += 1;

        // Get ready tasks (excluding completed and waiting for input)
        let ready_tasks = self.get_ready_tasks_from_tree(
            &workflow.tasks,
            &completed_tasks,
            &waiting_for_input, // NEW
        );

        if ready_tasks.is_empty() {
            // Check if we're truly done or waiting for input
            if waiting_for_input.is_empty() {
                debug!("All tasks complete");
                break;
            } else {
                debug!("Workflow paused - waiting for {} input(s)", waiting_for_input.len());
                // Signal that workflow is waiting for input
                break;
            }
        }

        // Execute ready tasks
        let results = self.execute_round(ready_tasks, &mut context).await?;

        // Process results
        for (task, result) in results {
            // Check if task is waiting for input
            if let Some(waiting) = result.output.get("waiting_for_input") {
                if waiting.as_bool().unwrap_or(false) {
                    waiting_for_input.insert(task.id.clone());
                    debug!("Task {} waiting for input", task.id);
                    continue;
                }
            }

            if result.success {
                completed_tasks.insert(task.id.clone());
                // ... existing audit trail logic ...
            } else {
                // ... existing error handling ...
            }
        }
    }

    // Build result with waiting tasks
    let success = errors.is_empty() && waiting_for_input.is_empty();
    
    // ...
}
```

## Resume Task with Input

### New Method on WorkflowEngine

**File**: `engine/src/engine.rs`

```rust
impl WorkflowEngine {
    /// Resume a task that was waiting for input
    pub async fn resume_task_with_input(
        &self,
        execution_id: &str,
        task_id: &str,
        input_value: String,
    ) -> Result<TaskResult, EngineError> {
        info!(
            execution_id = %execution_id,
            task_id = %task_id,
            "Resuming task with input"
        );

        // Get the task from context (would need to load from state)
        // For now, return success and let caller handle state update
        debug!("Task {} resumed with input", task_id);
        
        Ok(TaskResult {
            success: true,
            output: serde_json::json!({
                "resumed": true,
                "input_value": input_value,
            }),
            error: None,
        })
    }
}
```

## Example Workflows

### Simple Input Workflow

**File**: `engine/examples/user_input_simple.json`

```json
{
    "id": "simple-input",
    "name": "Simple User Input Workflow",
    "tasks": [
        {
            "id": "greeting",
            "name": "Display Greeting",
            "function": {
                "cli_command": {
                    "command": "echo",
                    "args": ["Hello! What's your name?"]
                }
            },
            "next_tasks": [
                {
                    "id": "get-name",
                    "name": "Get User Name",
                    "function": {
                        "user_input": {
                            "prompt": "Please enter your name:",
                            "input_type": "string",
                            "required": true,
                            "default": null
                        }
                    },
                    "next_tasks": [
                        {
                            "id": "thank-you",
                            "name": "Thank You",
                            "function": {
                                "cli_command": {
                                    "command": "echo",
                                    "args": ["Thank you for your input!"]
                                }
                            },
                            "next_tasks": []
                        }
                    ]
                }
            ]
        }
    ]
}
```

### Parallel Input Workflow

**File**: `engine/examples/user_input_parallel.json`

```json
{
    "id": "parallel-input",
    "name": "Parallel User Input Workflow",
    "tasks": [
        {
            "id": "start",
            "name": "Start",
            "function": {
                "cli_command": {
                    "command": "echo",
                    "args": ["Starting parallel input tasks"]
                }
            },
            "next_tasks": [
                {
                    "id": "input-a",
                    "name": "Input A",
                    "function": {
                        "user_input": {
                            "prompt": "Enter value A:",
                            "input_type": "string",
                            "required": true,
                            "default": null
                        }
                    },
                    "next_tasks": []
                },
                {
                    "id": "input-b",
                    "name": "Input B",
                    "function": {
                        "user_input": {
                            "prompt": "Enter value B:",
                            "input_type": "string",
                            "required": true,
                            "default": null
                        }
                    },
                    "next_tasks": []
                }
            ]
        }
    ]
}
```

### Nested Input Workflow

**File**: `engine/examples/user_input_nested.json`

```json
{
    "id": "nested-input",
    "name": "Nested User Input Workflow",
    "tasks": [
        {
            "id": "step1",
            "name": "Step 1",
            "function": {
                "cli_command": {
                    "command": "echo",
                    "args": ["Step 1 complete"]
                }
            },
            "next_tasks": [
                {
                    "id": "step2-input",
                    "name": "Step 2 Input",
                    "function": {
                        "user_input": {
                            "prompt": "Enter value for step 2:",
                            "input_type": "string",
                            "required": true,
                            "default": null
                        }
                    },
                    "next_tasks": [
                        {
                            "id": "step3",
                            "name": "Step 3",
                            "function": {
                                "cli_command": {
                                    "command": "echo",
                                    "args": ["Step 3 complete"]
                                }
                            },
                            "next_tasks": []
                        }
                    ]
                }
            ]
        }
    ]
}
```

## Testing Requirements

### Test Files

1. **`engine/tests/user_input_handler_tests.rs`**
   - Handler creation
   - Execution returns WaitingForInput
   - Output format validation
   - Error cases

2. **`engine/tests/resume_tests.rs`**
   - Resume with valid input
   - Resume with invalid input
   - Resume workflow continuation

3. **`engine/tests/engine_user_input_tests.rs`**
   - Full workflow execution with input
   - Parallel input scenarios
   - Nested input scenarios

### Test Coverage

- Handler executes correctly
- Task status transitions
- Parallel task independence
- Resume functionality
- Error handling

## Logging Requirements

### Handler Level Logging

```rust
info!("Executing user input task");
debug!("Task marked as WaitingForInput");
trace!("Input configuration: {:?}", config);
```

### Engine Level Logging

```rust
debug!("Found {} tasks waiting for input", waiting_for_input.len());
trace!("Task {} skipped - waiting for input", task_id);
info!("Workflow paused - waiting for {} input(s)", count);
```

### Resume Level Logging

```rust
info!("Resuming task {} with input", task_id);
debug!("Input validation passed");
trace!("Resume output: {:?}", output);
```

