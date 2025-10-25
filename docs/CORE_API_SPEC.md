# Core API Specification

## Overview
The `core` crate provides the public API that the GUI and CLI consume. It coordinates between the persistence layer and the engine, managing the global store singleton and orchestrating workflow execution.

## Public API Functions

### Initialization Functions

#### `init_tracing`
Initialize the tracing/logging system.

```rust
pub fn init_tracing(log_file: Option<String>) -> Result<TracingGuard, String>
```

**Parameters:**
- `log_file: Option<String>` - Optional path to log file. If None, logs to stdout.

**Returns:**
- `Result<TracingGuard, String>` - Guard that must be kept alive for logging to work, or error message.

**Usage:**
```rust
let _guard = s_e_e_core::init_tracing(None)
    .expect("Failed to initialize tracing");
```

**Implementation Notes:**
- Uses `tracing_subscriber` with JSON formatting
- Returns `tracing_appender::non_blocking::WorkerGuard` 
- Guard must be kept alive for duration of application
- Should be called once at application startup

---

#### `init_global_store`
Initialize the global persistence store singleton.

```rust
pub async fn init_global_store() -> Result<(), String>
```

**Returns:**
- `Result<(), String>` - Success or error message.

**Usage:**
```rust
s_e_e_core::init_global_store().await
    .expect("Failed to initialize store");
```

**Implementation Notes:**
- Creates database at `~/.s_e_e/data.redb`
- Uses `OnceLock` for thread-safe singleton
- Returns error if already initialized
- Must be called before `get_global_store()`
- Safe to call from multiple processes (redb handles multi-process access)

---

#### `get_global_store`
Get reference to the global persistence store.

```rust
pub fn get_global_store() -> Result<Arc<Store>, String>
```

**Returns:**
- `Result<Arc<Store>, String>` - Shared reference to store, or error if not initialized.

**Usage:**
```rust
let store = s_e_e_core::get_global_store()?;
let workflows = store.list_workflows().await?;
```

**Implementation Notes:**
- Returns error if `init_global_store()` hasn't been called
- Returns `Arc<Store>` for cheap cloning
- Thread-safe and can be called from any thread
- Multiple GUI processes can each call this safely

---

### Workflow Execution Functions

#### `execute_workflow_by_id`
Execute a workflow by loading it from persistence and running it through the engine.

```rust
pub async fn execute_workflow_by_id(
    workflow_id: &str,
    callback: Option<OutputCallback>,
) -> Result<WorkflowResult, CoreError>
```

**Parameters:**
- `workflow_id: &str` - ID of workflow to execute
- `callback: Option<OutputCallback>` - Optional callback for streaming output during execution

**Returns:**
- `Result<WorkflowResult, CoreError>` - Execution result with tasks, audit trail, and errors.

**Errors:**
- `CoreError::WorkflowNotFound` - Workflow ID doesn't exist in persistence
- `CoreError::Persistence` - Database error loading workflow
- `CoreError::Engine` - Workflow execution failed
- `CoreError::Execution` - Invalid workflow JSON or other execution errors

**Usage:**
```rust
use s_e_e_core::{execute_workflow_by_id, OutputCallback};
use std::sync::Arc;

// With callback for streaming output
let callback: OutputCallback = Arc::new(|msg: String| {
    println!("Progress: {}", msg);
});

let result = execute_workflow_by_id("workflow-123", Some(callback)).await?;
println!("Success: {}, Tasks: {}", result.success, result.tasks.len());

// Without callback
let result = execute_workflow_by_id("workflow-123", None).await?;
```

**Execution Flow:**
1. Load WorkflowDefinition from persistence
2. Parse workflow JSON content to EngineWorkflow
3. Create initial WorkflowExecution record (status: Running)
4. Execute workflow through engine
5. Convert results to persistence types
6. Save execution results to database
7. Return WorkflowResult with execution_id

**Implementation Notes:**
- Automatically persists execution results
- Creates execution record before starting
- Updates execution record after completion
- Generates UUID for execution_id
- Callback is called during engine execution for progress updates

---

#### `resume_task`
Resume a paused task that's waiting for input.

```rust
pub async fn resume_task(
    execution_id: &str,
    task_id: &str,
) -> Result<(), CoreError>
```

**Parameters:**
- `execution_id: &str` - ID of workflow execution containing the task
- `task_id: &str` - ID of task to resume

**Returns:**
- `Result<(), CoreError>` - Success or error.

**Errors:**
- `CoreError::WorkflowNotFound` - Execution ID doesn't exist
- `CoreError::TaskNotFound` - Task ID doesn't exist in execution
- `CoreError::Execution` - Task is not in WaitingForInput status
- `CoreError::Engine` - Task resumption failed

**Usage:**
```rust
s_e_e_core::resume_task("exec-456", "task-789").await?;
```

**Implementation Notes:**
- Validates task is in `WaitingForInput` status
- Loads execution state from persistence
- Resumes task through engine
- Updates task status in persistence
- Not yet implemented in engine - placeholder for future feature

---

## WorkflowDefinition Methods

The `WorkflowDefinition` type has additional methods beyond its struct fields.

### `get_default_workflows`
Get default workflow templates.

```rust
impl WorkflowDefinition {
    pub fn get_default_workflows() -> Vec<WorkflowDefinition>
}
```

**Returns:**
- `Vec<WorkflowDefinition>` - List of built-in workflow templates.

**Usage:**
```rust
let defaults = WorkflowDefinition::get_default_workflows();
for workflow in defaults {
    store.save_workflow(&workflow).await?;
}
```

**Implementation Notes:**
- Returns pre-defined workflow templates
- Includes simple, parallel, and nested example workflows
- Each has `is_default: true` flag
- IDs are stable (e.g., "default-simple")
- Can be saved to persistence for user access

---

### `get_name`
Get the display name for a workflow.

```rust
impl WorkflowDefinition {
    pub fn get_name(&self) -> &str
}
```

**Returns:**
- `&str` - Workflow name.

**Usage:**
```rust
let name = workflow.get_name();
println!("Executing: {}", name);
```

**Implementation Notes:**
- Simply returns reference to `self.name` field
- Convenience method for GUI display

---

## Type Definitions

### `OutputCallback`
Callback function type for streaming execution output.

```rust
pub type OutputCallback = Arc<dyn Fn(String) + Send + Sync>;
```

**Usage:**
```rust
use std::sync::Arc;

let callback: OutputCallback = Arc::new(|msg: String| {
    tracing::info!("Engine output: {}", msg);
});
```

**Notes:**
- Must be `Send + Sync` for async execution
- Wrapped in `Arc` for cheap cloning
- Called during engine execution for progress updates
- Optional parameter to `execute_workflow_by_id`

---

### `TracingGuard`
Type alias for tracing worker guard.

```rust
pub type TracingGuard = tracing_appender::non_blocking::WorkerGuard;
```

**Notes:**
- Must be kept alive for logging to work
- Dropping the guard stops background logging thread
- Store in a variable prefixed with `_` to indicate intentionally unused

---

## Re-exported Types

The core crate re-exports types from persistence and engine for convenience.

### From Persistence Crate
```rust
// Data models
pub use persistence::{
    WorkflowDefinition,
    WorkflowExecution,
    WorkflowExecutionSummary,
    WorkflowMetadata,
    TaskExecution,
    UserPrompt,
    AuditEvent,
    AppSettings,
    WorkflowJson,
};

// Enums
pub use persistence::{
    WorkflowStatus,
    Theme,
};

// Store
pub use persistence::Store;
```

### From Engine Crate
```rust
pub use engine::{
    TaskInfo,
    TaskStatus,
    AuditStatus,
};
```

### Core Bridge Types
```rust
pub use crate::bridge::WorkflowResult;
pub use crate::errors::CoreError;
```

---

## Error Types

### `CoreError`
Main error type for core crate operations.

```rust
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Engine error: {0}")]
    Engine(#[from] engine::EngineError),
    
    #[error("Persistence error: {0}")]
    Persistence(String),
    
    #[error("Workflow not found: {0}")]
    WorkflowNotFound(String),
    
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    
    #[error("Execution error: {0}")]
    Execution(String),
}
```

**Variants:**

- `Engine(EngineError)` - Error from workflow engine
  - Wraps engine execution, parsing, and handler errors
  - Automatically converted via `From` trait

- `Persistence(String)` - Error from persistence layer
  - Database errors, serialization errors, I/O errors
  - User-friendly error message

- `WorkflowNotFound(String)` - Workflow ID not found in database
  - Contains workflow ID that wasn't found
  - Returned by `execute_workflow_by_id`

- `TaskNotFound(String)` - Task ID not found in execution
  - Contains task ID that wasn't found
  - Returned by `resume_task`

- `Execution(String)` - General execution error
  - Invalid workflow JSON
  - Invalid task status for operation
  - Other execution-related errors

**Error Conversions:**
```rust
impl From<engine::EngineError> for CoreError {
    fn from(err: engine::EngineError) -> Self {
        CoreError::Engine(err)
    }
}

impl From<String> for CoreError {
    fn from(err: String) -> Self {
        CoreError::Persistence(err)
    }
}
```

---

## Global Store Singleton Pattern

### Implementation Strategy

The global store uses `OnceLock` for thread-safe singleton initialization.

```rust
use std::sync::{Arc, OnceLock};
use persistence::Store;

static GLOBAL_STORE: OnceLock<Arc<Store>> = OnceLock::new();

pub async fn init_global_store() -> Result<(), String> {
    let db_path = get_database_path()?;
    let store = Store::new(&db_path).await?;
    
    GLOBAL_STORE
        .set(Arc::new(store))
        .map_err(|_| "Store already initialized".to_string())?;
    
    Ok(())
}

pub fn get_global_store() -> Result<Arc<Store>, String> {
    GLOBAL_STORE
        .get()
        .cloned()
        .ok_or_else(|| "Store not initialized. Call init_global_store() first.".to_string())
}

fn get_database_path() -> Result<String, String> {
    let home = std::env::var("HOME")
        .map_err(|_| "HOME environment variable not set")?;
    let data_dir = format!("{}/.s_e_e", home);
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;
    Ok(format!("{}/data.redb", data_dir))
}
```

### Thread Safety Guarantees

- `OnceLock` ensures initialization happens exactly once
- Multiple threads can safely call `init_global_store()` concurrently
- Only first call succeeds; others return error
- `Arc<Store>` allows cheap cloning for multiple references
- Store methods use async Tokio RwLock for internal synchronization

### Multi-Process Safety

- `redb` database supports multi-process concurrent access
- Multiple GUI processes can read simultaneously
- Writers don't block readers (MVCC)
- Each process has its own `Arc<Store>` instance
- All processes share the same underlying database file

### Initialization Order

1. Application startup
2. Call `init_tracing()` - save guard
3. Call `init_global_store().await` - initialize database
4. Call `get_global_store()` - get store reference
5. Use store throughout application

### Error Handling

**Store Not Initialized:**
```rust
match get_global_store() {
    Ok(store) => { /* use store */ },
    Err(e) => {
        eprintln!("Store not initialized: {}", e);
        std::process::exit(1);
    }
}
```

**Already Initialized:**
```rust
if let Err(e) = init_global_store().await {
    if e.contains("already initialized") {
        // This is fine, store is ready
    } else {
        // Real error
        return Err(e);
    }
}
```

---

## Usage Examples

### Complete Initialization
```rust
#[tokio::main]
async fn main() {
    // Initialize tracing
    let _guard = s_e_e_core::init_tracing(None)
        .expect("Failed to initialize tracing");
    
    // Initialize persistence
    s_e_e_core::init_global_store().await
        .expect("Failed to initialize store");
    
    // Application ready
    tracing::info!("Application initialized");
}
```

### Execute Workflow
```rust
use s_e_e_core::{execute_workflow_by_id, get_global_store};

async fn run_workflow(workflow_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Execute workflow
    let result = execute_workflow_by_id(workflow_id, None).await?;
    
    println!("Workflow completed: {}", result.success);
    println!("Tasks: {}", result.tasks.len());
    println!("Errors: {:?}", result.errors);
    
    Ok(())
}
```

### Load and Save Data
```rust
use s_e_e_core::{get_global_store, WorkflowDefinition};

async fn manage_workflows() -> Result<(), String> {
    let store = get_global_store()?;
    
    // Load defaults
    let defaults = WorkflowDefinition::get_default_workflows();
    for workflow in defaults {
        store.save_workflow(&workflow).await?;
    }
    
    // List all workflows
    let workflows = store.list_workflows().await?;
    for wf in workflows {
        println!("{}: {}", wf.id, wf.name);
    }
    
    Ok(())
}
```

### Stream Execution Output
```rust
use s_e_e_core::{execute_workflow_by_id, OutputCallback};
use std::sync::Arc;

async fn run_with_progress(workflow_id: &str) {
    let callback: OutputCallback = Arc::new(|msg: String| {
        println!("[ENGINE] {}", msg);
    });
    
    match execute_workflow_by_id(workflow_id, Some(callback)).await {
        Ok(result) => println!("Complete: {}", result.success),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

---

## Testing Strategy

### Unit Tests
- Test singleton initialization
- Test error conversions
- Test type re-exports
- Test default workflows

### Integration Tests
- Test complete workflow execution
- Test persistence integration
- Test engine integration
- Test multi-threaded access
- Test error propagation

### Multi-Process Tests
- Test concurrent readers
- Test database locking
- Test process-safe initialization

---

## Dependencies

### Cargo.toml
```toml
[dependencies]
persistence = { path = "../persistence" }
engine = { path = "../engine" }
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json"] }
tracing-appender = "0.2"
thiserror = "1.0"
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
```

---

## Module Structure

```
core/
├── Cargo.toml
└── src/
    ├── lib.rs           # Public API exports
    ├── api.rs           # Workflow execution functions
    ├── store_singleton.rs  # Global store singleton
    ├── bridge.rs        # Type conversions
    ├── errors.rs        # CoreError definition
    └── tracing.rs       # Tracing initialization
```

---

## Success Criteria

✓ All GUI-expected functions are defined
✓ Type re-exports match GUI imports
✓ Error types support all failure modes
✓ Singleton pattern is thread-safe and multi-process safe
✓ API is ergonomic and easy to use
✓ Documentation is complete with examples
✓ Async/await support throughout

