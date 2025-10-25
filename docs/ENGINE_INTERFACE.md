# Engine Interface Analysis

## Public API Exports

### Main Functions
- `execute_workflow_from_json(json: &str) -> Result<WorkflowResult, EngineError>`
- `parse_workflow(json: &str) -> Result<EngineWorkflow, ParserError>`

### Main Types
- `WorkflowEngine` - Main execution engine struct
- `EngineWorkflow` - Workflow structure for execution
- `EngineTask` - Task structure with recursive next_tasks
- `TaskFunction` - Task function types
- `TaskResult` - Result of task execution
- `WorkflowResult` - Result of workflow execution
- `TaskInfo` - Task information for results
- `AuditEntry` - Audit trail entry
- `ExecutionContext` - Execution context for handlers
- `TaskStatus` - Task execution status enum
- `AuditStatus` - Audit entry status enum

### Error Types
- `EngineError` - Main error type
- `ParserError` - JSON parsing errors
- `GraphError` - Dependency graph errors
- `HandlerError` - Task handler errors

## Input Types

### EngineWorkflow
```rust
pub struct EngineWorkflow {
    pub id: String,
    pub name: String,
    pub tasks: Vec<EngineTask>,
}
```

### EngineTask
```rust
pub struct EngineTask {
    pub id: String,
    pub name: String,
    pub function: TaskFunction,
    pub next_tasks: Vec<EngineTask>,  // Recursive structure
    pub status: TaskStatus,
}
```

### TaskFunction
```rust
pub enum TaskFunction {
    #[serde(rename = "cli_command")]
    CliCommand { command: String, args: Vec<String> },
    #[serde(rename = "cursor_agent")]
    CursorAgent { prompt: String, config: Value },
    #[serde(rename = "custom")]
    Custom { name: String, input: Value },
}
```

## Output Types

### WorkflowResult
```rust
pub struct WorkflowResult {
    pub success: bool,
    pub workflow_name: String,
    pub tasks: Vec<TaskInfo>,
    pub audit_trail: Vec<AuditEntry>,
    pub per_task_logs: HashMap<String, Vec<String>>,
    pub errors: Vec<String>,
}
```

### TaskResult
```rust
pub struct TaskResult {
    pub success: bool,
    pub output: Value,
    pub error: Option<String>,
}
```

### TaskInfo
```rust
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    pub status: TaskStatus,
}
```

### AuditEntry
```rust
pub struct AuditEntry {
    pub task_id: String,
    pub status: AuditStatus,
    pub timestamp: String,  // RFC3339 string
    pub changes_count: usize,
    pub message: String,
}
```

## Enums

### TaskStatus
```rust
pub enum TaskStatus {
    Pending,
    InProgress,
    Complete,
    Failed,
    WaitingForInput,
}
```

### AuditStatus
```rust
pub enum AuditStatus {
    Success,
    Failure,
}
```

## Execution Context

### ExecutionContext
```rust
pub struct ExecutionContext {
    pub execution_id: String,
    pub workflow_name: String,
    pub output_logs: Vec<String>,
    pub per_task_logs: HashMap<String, Vec<String>>,
    pub tasks: HashMap<String, EngineTask>,
}
```

## Key Characteristics

### JSON Parsing
- Expects JSON with `id`, `name`, and `tasks` fields
- Supports recursive `next_tasks` structure
- Validates no duplicate task IDs
- Uses serde rename attributes for enum variants

### Execution Model
- Parallel execution of independent tasks
- Sequential execution based on dependencies
- Uses dependency graph for execution order
- Supports task resumption (WaitingForInput status)

### Error Handling
- Comprehensive error types for different failure modes
- Parser errors for malformed JSON
- Graph errors for circular dependencies
- Handler errors for task execution failures

### Concurrency
- Uses tokio for async execution
- Spawns parallel tasks for independent execution
- Thread-safe execution context
- No built-in persistence (expects external store integration)
