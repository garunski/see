# API Documentation - User Input

## Overview

This document describes the API methods for user input functionality in the SEE (Speculative Execution Engine) project.

## Core API

### Input Management

#### `provide_user_input()`

Provide user input for a task that is waiting for input.

**Location:** `core/src/api/input.rs`

**Signature:**
```rust
pub async fn provide_user_input(
    execution_id: &str,
    task_id: &str,
    input_value: String,
) -> Result<(), CoreError>
```

**Parameters:**
- `execution_id` - The workflow execution ID
- `task_id` - The task execution ID that is waiting for input
- `input_value` - The input value to provide (must match expected input type)

**Returns:**
- `Ok(())` on success
- `Err(CoreError)` on failure

**Errors:**
- `CoreError::WorkflowNotFound` - Execution not found
- `CoreError::TaskNotFound` - Task not found
- `CoreError::Execution` - Task is not waiting for input
- `CoreError::InputValidationFailed` - Input type validation failed

**Example:**
```rust
use s_e_e_core::provide_user_input;

// Provide input for a waiting task
provide_user_input(
    "exec-123",
    "task-get-name",
    "John Doe".to_string(),
).await?;
```

#### `get_pending_inputs()`

Get all pending input requests for a workflow execution.

**Location:** `core/src/api/input.rs`

**Signature:**
```rust
pub async fn get_pending_inputs(
    workflow_id: &str,
) -> Result<Vec<UserInputRequest>, CoreError>
```

**Parameters:**
- `workflow_id` - The workflow execution ID

**Returns:**
- `Ok(Vec<UserInputRequest>)` - List of pending input requests
- `Err(CoreError)` on failure

**Example:**
```rust
use s_e_e_core::get_pending_inputs;

let pending = get_pending_inputs("exec-123").await?;
for request in pending {
    println!("Task {} needs input: {}", request.task_execution_id, request.prompt_text);
}
```

#### `get_tasks_waiting_for_input()`

Get all tasks that are currently waiting for input in a workflow.

**Location:** `core/src/api/input.rs`

**Signature:**
```rust
pub async fn get_tasks_waiting_for_input(
    workflow_id: &str,
) -> Result<Vec<TaskExecution>, CoreError>
```

**Parameters:**
- `workflow_id` - The workflow execution ID

**Returns:**
- `Ok(Vec<TaskExecution>)` - List of tasks waiting for input
- `Err(CoreError)` on failure

**Example:**
```rust
use s_e_e_core::get_tasks_waiting_for_input;

let waiting_tasks = get_tasks_waiting_for_input("exec-123").await?;
println!("{} tasks waiting for input", waiting_tasks.len());
```

### Execution API Enhancements

#### `execute_workflow_by_id()`

Execute a workflow by ID with support for user input.

**Location:** `core/src/api/execution.rs`

**Signature:**
```rust
pub async fn execute_workflow_by_id(
    workflow_id: &str,
    callback: Option<OutputCallback>,
) -> Result<WorkflowResult, CoreError>
```

**Behavior:**
- Executes the workflow through the engine
- If any tasks require input, workflow pauses
- Returns `WorkflowResult` with `errors` containing "Waiting for user input"
- Caller can check for pending inputs and provide them
- Resume by providing input for each waiting task

**Workflow Result:**
```rust
pub struct WorkflowResult {
    pub success: bool,
    pub workflow_name: String,
    pub execution_id: String,
    pub tasks: Vec<engine::TaskInfo>,
    pub audit_trail: Vec<AuditEvent>,
    pub per_task_logs: HashMap<String, Vec<String>>,
    pub errors: Vec<String>,  // Contains "Waiting for user input" if paused
}
```

**Example:**
```rust
use s_e_e_core::execute_workflow_by_id;
use s_e_e_core::provide_user_input;
use s_e_e_core::get_tasks_waiting_for_input;

let result = execute_workflow_by_id("my-workflow", None).await?;

// Check if workflow is waiting for input
if result.errors.contains(&"Waiting for user input".to_string()) {
    // Get tasks waiting for input
    let waiting_tasks = get_tasks_waiting_for_input(&result.execution_id).await?;
    
    // Provide input for each task
    for task in waiting_tasks {
        let input = read_user_input(&task.name)?;
        provide_user_input(&result.execution_id, &task.id, input).await?;
    }
    
    // Continue execution
}
```

### Resume API Enhancements

#### `resume_task()`

Resume a task that was waiting for input.

**Location:** `core/src/api/resume.rs`

**Signature:**
```rust
pub async fn resume_task(
    execution_id: &str,
    task_id: &str,
) -> Result<(), CoreError>
```

**Behavior:**
- Checks if task has input value
- Validates task status
- Resumes task execution
- Continues workflow if all dependencies are met

**Example:**
```rust
use s_e_e_core::resume_task;

// Task should have input provided first
resume_task("exec-123", "task-get-name").await?;
```

## Engine API

### UserInputHandler

#### Handler Execution

**Location:** `engine/src/handlers/user_input.rs`

The `UserInputHandler` implements the `TaskHandler` trait:

```rust
impl TaskHandler for UserInputHandler {
    async fn execute(
        &self,
        context: &mut ExecutionContext,
        task: &EngineTask,
    ) -> Result<TaskResult, HandlerError>;
}
```

**Behavior:**
- Marks task as `WaitingForInput` status
- Returns `TaskResult` with special output indicating waiting state
- Workflow pauses execution
- Task can be resumed with input value

**TaskResult Output:**
```json
{
    "waiting_for_input": true,
    "prompt": "Please enter your name:",
    "input_type": "string",
    "required": true,
    "default": null
}
```

### Engine Execution

#### Resume with Input

**Location:** `engine/src/engine.rs`

**Signature:**
```rust
impl WorkflowEngine {
    pub async fn resume_task_with_input(
        &self,
        execution_id: &str,
        task_id: &str,
        input_value: String,
    ) -> Result<TaskResult, EngineError>
}
```

**Behavior:**
- Resumes a task that was waiting for input
- Uses provided input value
- Continues execution to next tasks
- Returns task result

## Error Types

### CoreError

All input-related errors are of type `CoreError`:

```rust
pub enum CoreError {
    // ... existing errors ...

    #[error("Invalid input type: {0}")]
    InvalidInputType(String),

    #[error("Input value required but not provided")]
    InputRequired,

    #[error("Input validation failed: {0}")]
    InputValidationFailed(String),

    #[error("Task is not waiting for input")]
    TaskNotWaitingForInput,

    #[error("Workflow is waiting for user input")]
    WorkflowWaitingForInput,
}
```

## Input Validation

### Supported Types

#### String
- **Examples:** "hello", "world", "John Doe"
- **Validation:** Any string is valid
- **Accepted:** Any non-empty string

#### Number
- **Examples:** "123", "3.14", "-42"
- **Validation:** Must parse as f64
- **Accepted:** Integer or floating point numbers

#### Boolean
- **Examples:** "true", "false", "yes", "no", "1", "0"
- **Validation:** Case-insensitive boolean values
- **Accepted:** "true", "false", "1", "0", "yes", "no"

### Validation Function

```rust
fn validate_input_value(
    value: &str,
    input_type: &persistence::InputType,
) -> Result<(), CoreError>
```

**Usage:**
```rust
use s_e_e_core::validate_input_value;
use persistence::InputType;

validate_input_value("123", &InputType::Number)?;  // Ok
validate_input_value("abc", &InputType::Number)?;   // Error
```

## Logging

All API methods log their operations:

**Levels:**
- `info!` - Major operations (provide input, resume task)
- `debug!` - Detailed state information
- `trace!` - Fine-grained execution details
- `error!` - Error conditions

**Example Logs:**
```rust
info!("Providing user input for task");
debug!("Input validation passed");
trace!("Input value: {}", value);
error!("Input validation failed");
```

## Workflow State Transitions

### User Input Flow

```
┌─────────────┐
│   Pending  │
└──────┬──────┘
       │ execute_workflow_by_id()
       ▼
┌─────────────┐
│ In Progress │
└──────┬──────┘
       │ reach user_input task
       ▼
┌─────────────────────┐
│ Waiting For Input   │ ◄─── provide_user_input()
└──────┬──────────────┘
       │ resume_task()
       ▼
┌─────────────┐
│ In Progress │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Complete   │
└─────────────┘
```

## Best Practices

### 1. Error Handling
Always handle all possible errors from API calls:
```rust
match provide_user_input(exec_id, task_id, input).await {
    Ok(_) => println!("Input provided successfully"),
    Err(CoreError::TaskNotWaitingForInput) => {
        eprintln!("Task is not waiting for input");
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

### 2. Input Validation
Validate input on the client side before calling the API:
```rust
fn validate_number(input: &str) -> bool {
    input.parse::<f64>().is_ok()
}
```

### 3. State Checking
Always check workflow state before providing input:
```rust
let waiting_tasks = get_tasks_waiting_for_input(exec_id).await?;
if waiting_tasks.is_empty() {
    return Ok(()); // No input needed
}
```

### 4. Sequential Input
For multiple inputs, provide them in order:
```rust
for task in waiting_tasks {
    let input = read_user_input(&task.name)?;
    provide_user_input(exec_id, &task.id, input).await?;
}
```

## Examples

### Complete Workflow with Input

```rust
use s_e_e_core::*;

async fn run_workflow_with_input(workflow_id: &str) -> Result<(), CoreError> {
    // Start workflow
    let result = execute_workflow_by_id(workflow_id, None).await?;
    
    // Check if waiting for input
    if result.errors.contains(&"Waiting for user input".to_string()) {
        // Get pending inputs
        let waiting = get_tasks_waiting_for_input(&result.execution_id).await?;
        
        // Provide input for each task
        for task in waiting {
            let input = prompt_user(&task.name)?;
            provide_user_input(&result.execution_id, &task.id, input).await?;
        }
        
        // Resume workflow
        // (Would need additional API for continuing)
    }
    
    Ok(())
}
```

## See Also

- [Architecture Documentation](./ARCHITECTURE.md)
- [Implementation Steps](./IMPLEMENTATION_STEPS.md)
- [Testing Strategy](./TESTING_STRATEGY.md)

