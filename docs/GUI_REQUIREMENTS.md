# GUI Requirements Analysis

## Types Imported from `s_e_e_core`

### Core Data Types (ACTUAL from grep analysis)
- `WorkflowDefinition` - Workflow templates with metadata
- `WorkflowExecution` - Full execution records  
- `WorkflowExecutionSummary` - Lightweight execution summaries
- `WorkflowMetadata` - Basic workflow metadata
- `TaskExecution` - Individual task execution records
- `TaskInfo` - Task information for UI display
- `UserPrompt` - User-defined prompts
- `AuditEvent` - Audit trail entries
- `AppSettings` - Application configuration
- `WorkflowJson` - Raw workflow JSON content
- `WorkflowResult` - Execution result from core
- `WorkflowStatus` - Execution status enum
- `Theme` - UI theme enum (Light, Dark, System)
- `AuditStatus` - Audit entry status (Success, Failure)

### Function Types
- `OutputCallback` - Callback for streaming execution output
- `CoreError` - Core error type

## Global Store API Requirements

### Initialization Functions
- `init_tracing(Option<String>) -> Result<TracingGuard, String>` - Initialize logging
- `init_global_store() -> Result<(), String>` - Initialize persistence layer
- `get_global_store() -> Result<Arc<Store>, String>` - Get global store instance

### Store Methods Required (ACTUAL from file analysis)

#### Workflow Management
- `save_workflow(workflow: &WorkflowDefinition) -> Result<(), String>`
- `get_workflow(id: &str) -> Result<WorkflowDefinition, String>`
- `list_workflows() -> Result<Vec<WorkflowDefinition>, String>`
- `delete_workflow(id: &str) -> Result<(), String>`

#### Execution Management
- `save_workflow_execution(workflow: WorkflowExecution) -> Result<(), String>`
- `get_workflow_execution(id: &str) -> Result<WorkflowExecution, String>`
- `list_workflow_executions() -> Result<Vec<WorkflowExecution>, String>`
- `delete_workflow_execution(id: &str) -> Result<(), String>`
- `list_workflow_metadata() -> Result<Vec<WorkflowMetadata>, String>`
- `delete_workflow_metadata_and_tasks(id: &str) -> Result<(), String>`
- `get_workflow_with_tasks(id: &str) -> Result<WorkflowExecution, String>`

#### Task Management
- `save_task_execution(task: TaskExecution) -> Result<(), String>`
- `get_tasks_for_workflow(workflow_id: &str) -> Result<Vec<TaskExecution>, String>`

#### Prompt Management
- `save_prompt(prompt: &UserPrompt) -> Result<(), String>`
- `list_prompts() -> Result<Vec<UserPrompt>, String>`
- `delete_prompt(id: &str) -> Result<(), String>`

#### Settings Management
- `load_settings() -> Result<Option<AppSettings>, String>`
- `save_settings(settings: &AppSettings) -> Result<(), String>`

#### Audit Management
- `log_audit_event(event: AuditEvent) -> Result<(), String>`

#### Utility Functions
- `clear_all_data() -> Result<(), String>`

## Core API Functions Required (ACTUAL from file analysis)

### Workflow Execution
- `execute_workflow_by_id(workflow_id: &str, callback: Option<OutputCallback>) -> Result<WorkflowResult, CoreError>`
- `resume_task(execution_id: &str, task_id: &str) -> Result<(), CoreError>`

### Workflow Definition Methods
- `WorkflowDefinition::get_default_workflows() -> Vec<WorkflowDefinition>`

## Data Structure Requirements

### WorkflowDefinition Fields
```rust
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub content: String,  // JSON string
    pub is_default: bool,
    pub is_edited: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### WorkflowExecution Fields
```rust
pub struct WorkflowExecution {
    pub id: String,
    pub workflow_name: String,
    pub status: WorkflowStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub success: bool,
    pub tasks: Vec<TaskExecution>,
    pub timestamp: DateTime<Utc>,
}
```

### WorkflowExecutionSummary Fields
```rust
pub struct WorkflowExecutionSummary {
    pub id: String,
    pub workflow_name: String,
    pub status: WorkflowStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub success: bool,
    pub task_count: usize,
    pub timestamp: DateTime<Utc>,
}
```

### WorkflowMetadata Fields
```rust
pub struct WorkflowMetadata {
    pub id: String,
    pub name: String,
    pub status: String,  // "running" or other status
}
```

### TaskExecution Fields
```rust
pub struct TaskExecution {
    pub id: String,
    pub workflow_id: String,
    pub name: String,
    pub status: TaskStatus,
    pub output: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
```

### TaskInfo Fields (from Engine)
```rust
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    pub status: TaskStatus,
}
```

### UserPrompt Fields
```rust
pub struct UserPrompt {
    pub id: String,
    pub name: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### AuditEvent Fields
```rust
pub struct AuditEvent {
    pub id: String,
    pub task_id: String,
    pub status: AuditStatus,
    pub timestamp: DateTime<Utc>,
    pub changes_count: usize,
    pub message: String,
}
```

### AppSettings Fields
```rust
pub struct AppSettings {
    pub theme: Theme,
    pub auto_save: bool,
    pub notifications: bool,
    pub default_workflow: Option<String>,
}
```

### WorkflowResult Fields
```rust
pub struct WorkflowResult {
    pub success: bool,
    pub workflow_name: String,
    pub execution_id: String,
    pub tasks: Vec<TaskInfo>,
    pub audit_trail: Vec<AuditEntry>,
    pub per_task_logs: HashMap<String, Vec<String>>,
    pub errors: Vec<String>,
}
```

## Enums Required

### WorkflowStatus
```rust
pub enum WorkflowStatus {
    Pending,
    Running,
    Complete,
    Failed,
}
```

### TaskStatus (from Engine)
```rust
pub enum TaskStatus {
    Pending,
    InProgress,
    Complete,
    Failed,
    WaitingForInput,
}
```

### Theme
```rust
pub enum Theme {
    Light,
    Dark,
    System,
}
```

### AuditStatus
```rust
pub enum AuditStatus {
    Success,
    Failure,
}
```

## Error Handling Requirements

### CoreError
- Must implement `std::error::Error`
- Used for workflow execution errors

### Store Errors
- All store methods return `Result<T, String>` for simplicity
- GUI services wrap store errors in their own error types

## Concurrency Requirements

- Multiple GUI processes must be able to read simultaneously
- No blocking between readers
- Writers should not block readers
- Global store must be thread-safe and accessible from multiple processes
