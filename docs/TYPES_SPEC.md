# Type Specifications

## Overview
Complete type definitions for persistence and core crates based on GUI requirements and Engine interface analysis.

## Persistence Types

### WorkflowDefinition
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

### WorkflowExecution
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

### WorkflowExecutionSummary
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

### WorkflowMetadata
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    pub id: String,
    pub name: String,
    pub status: String,  // "running" or other status
}
```

### TaskExecution
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

### UserPrompt
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserPrompt {
    pub id: String,
    pub name: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### AuditEvent
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub task_id: String,
    pub status: AuditStatus,
    pub timestamp: DateTime<Utc>,
    pub changes_count: usize,
    pub message: String,
}
```

### AppSettings
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: Theme,
    pub auto_save: bool,
    pub notifications: bool,
    pub default_workflow: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            auto_save: true,
            notifications: true,
            default_workflow: None,
        }
    }
}
```

## Enums

### WorkflowStatus
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "complete")]
    Complete,
    #[serde(rename = "failed")]
    Failed,
}
```

### TaskStatus (re-exported from Engine)
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "complete")]
    Complete,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "waiting_for_input")]
    WaitingForInput,
}
```

### Theme
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
    #[serde(rename = "system")]
    System,
}
```

### AuditStatus (re-exported from Engine)
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuditStatus {
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "failure")]
    Failure,
}
```

## Core Bridge Types

### WorkflowResult (Enhanced)
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub success: bool,
    pub workflow_name: String,
    pub execution_id: String,  // Added for GUI tracking
    pub tasks: Vec<TaskInfo>,
    pub audit_trail: Vec<AuditEntry>,
    pub per_task_logs: HashMap<String, Vec<String>>,
    pub errors: Vec<String>,
}
```

### OutputCallback
```rust
pub type OutputCallback = Arc<dyn Fn(String) + Send + Sync>;
```

### CoreError
```rust
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Engine error: {0}")]
    Engine(#[from] engine::EngineError),
    
    #[error("Persistence error: {0}")]
    Persistence(#[from] persistence::PersistenceError),
    
    #[error("Workflow not found: {0}")]
    WorkflowNotFound(String),
    
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    
    #[error("Execution error: {0}")]
    Execution(String),
}
```

## Re-exports from Engine

### TaskInfo
```rust
// Re-exported from engine crate
pub use engine::TaskInfo;
```

### AuditEntry
```rust
// Re-exported from engine crate  
pub use engine::AuditEntry;
```

## Additional Types

### WorkflowJson
```rust
pub type WorkflowJson = String;  // Raw JSON content
```

### TracingGuard
```rust
pub type TracingGuard = tracing_appender::non_blocking::WorkerGuard;
```

## Serialization Requirements

### Required Traits
All types must implement:
- `Debug` - For debugging and logging
- `Clone` - For GUI state management
- `PartialEq` - For testing and comparisons
- `Serialize` - For JSON serialization
- `Deserialize` - For JSON deserialization

### Serialization Format
- Use `serde_json` for JSON serialization
- Use `chrono::DateTime<Utc>` for timestamps
- Use `serde(rename = "...")` for enum variants to match existing conventions
- Use `Option<T>` for optional fields
- Use `Vec<T>` for collections

### Key Constraints
- All IDs must be unique strings
- Timestamps must be UTC
- JSON content must be valid JSON strings
- Status strings must match enum variants exactly
- Error messages must be human-readable strings
