# Persistence Layer Specification - User Input

## Overview

This document specifies the persistence layer changes for user input support, including new models, extended models, store operations, and database schema.

## Database Schema

### New Table: `user_input_requests`

```sql
CREATE TABLE IF NOT EXISTS user_input_requests (
    id TEXT PRIMARY KEY,
    data JSON NOT NULL
)
```

### Modified Table: `task_executions`

**No schema change needed** - Using JSON data field. Changes are in the model only.

## Models

### 1. UserInputRequest Model

**File**: `persistence/src/models/user_input_request.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Runtime input request for a task execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserInputRequest {
    pub id: String,
    pub task_execution_id: String,
    pub workflow_execution_id: String,
    pub prompt_text: String,
    pub input_type: InputType,
    pub required: bool,
    pub default_value: Option<Value>,
    pub validation_rules: Value,
    pub status: InputRequestStatus,
    pub created_at: DateTime<Utc>,
    pub fulfilled_at: Option<DateTime<Utc>>,
    pub fulfilled_value: Option<String>,
}
```

**InputType Enum**:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputType {
    String,
    Number,
    Boolean,
}
```

**InputRequestStatus Enum**:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputRequestStatus {
    Pending,
    Fulfilled,
}
```

**Validation Methods**:
```rust
impl UserInputRequest {
    pub fn validate(&self) -> Result<(), String>;
    pub fn is_fulfilled(&self) -> bool;
    pub fn is_pending(&self) -> bool;
}
```

### 2. Extended TaskExecution Model

**File**: `persistence/src/models/task.rs` (extend existing)

**New Fields Added**:
```rust
impl TaskExecution {
    pub user_input: Option<String>,
    pub input_request_id: Option<String>,
    pub prompt_id: Option<String>,
}
```

**Field Descriptions**:
- `user_input`: The actual value provided by the user
- `input_request_id`: Reference to the UserInputRequest that triggered this input
- `prompt_id`: Optional reference to a UserPrompt template (if applicable)

**Additional Methods**:
```rust
impl TaskExecution {
    pub fn has_user_input(&self) -> bool;
    pub fn get_input_request_id(&self) -> Option<&str>;
}
```

### 3. User Input Enums

**File**: `persistence/src/models/enums.rs` (extend existing)

**Added Enums**:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputType {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "number")]
    Number,
    #[serde(rename = "boolean")]
    Boolean,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputRequestStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "fulfilled")]
    Fulfilled,
}
```

## Store Operations

### User Input Request Operations

**File**: `persistence/src/store/user_input.rs`

```rust
impl Store {
    /// Save a user input request
    pub async fn save_input_request(&self, request: &UserInputRequest) -> Result<(), String>;

    /// Get an input request by ID
    pub async fn get_input_request(&self, id: &str) -> Result<Option<UserInputRequest>, String>;

    /// Get input request by task execution ID
    pub async fn get_input_request_by_task(&self, task_id: &str) -> Result<Option<UserInputRequest>, String>;

    /// Get all pending input requests for a workflow execution
    pub async fn get_pending_inputs_for_workflow(&self, workflow_id: &str) -> Result<Vec<UserInputRequest>, String>;

    /// Get all pending input requests
    pub async fn get_all_pending_inputs(&self) -> Result<Vec<UserInputRequest>, String>;

    /// Mark input request as fulfilled
    pub async fn fulfill_input_request(&self, id: &str, value: String) -> Result<(), String>;

    /// Delete an input request
    pub async fn delete_input_request(&self, id: &str) -> Result<(), String>;
}
```

**Implementation Notes**:
- Use JSON serialization like other models
- Table name: `user_input_requests`
- Index on `task_execution_id` for fast lookups
- Index on `workflow_execution_id` for batch queries

### Task Execution Operations

**File**: `persistence/src/store/task.rs` (extend existing)

**New Methods**:
```rust
impl Store {
    /// Save task with user input
    pub async fn save_task_with_input(&self, task: &TaskExecution) -> Result<(), String>;

    /// Get all tasks waiting for input
    pub async fn get_tasks_waiting_for_input(&self) -> Result<Vec<TaskExecution>, String>;

    /// Get tasks waiting for input in a specific workflow
    pub async fn get_tasks_waiting_for_input_in_workflow(&self, workflow_id: &str) -> Result<Vec<TaskExecution>, String>;

    /// Get task with input request
    pub async fn get_task_with_input_request(&self, task_id: &str) -> Result<Option<TaskExecution>, String>;
}
```

### Integration with Existing Operations

**File**: `persistence/src/store/task.rs`

**Modified Methods**:
- `save_task_execution()` - Already handles JSON serialization, no changes needed
- `get_task_execution()` - Returns tasks with new fields populated
- `get_tasks_by_workflow()` - Includes new fields

## Database Migration

**File**: `persistence/src/store/lib.rs`

**In `create_tables()` method**:
```rust
let tables = [
    "CREATE TABLE IF NOT EXISTS workflows (id TEXT PRIMARY KEY, data JSON NOT NULL)",
    "CREATE TABLE IF NOT EXISTS workflow_executions (id TEXT PRIMARY KEY, data JSON NOT NULL)",
    "CREATE TABLE IF NOT EXISTS task_executions (id TEXT PRIMARY KEY, data JSON NOT NULL)",
    "CREATE TABLE IF NOT EXISTS user_input_requests (id TEXT PRIMARY KEY, data JSON NOT NULL)",
    "CREATE TABLE IF NOT EXISTS user_prompts (id TEXT PRIMARY KEY, data JSON NOT NULL)",
    "CREATE TABLE IF NOT EXISTS audit_events (id TEXT PRIMARY KEY, data JSON NOT NULL)",
    "CREATE TABLE IF NOT EXISTS settings (id TEXT PRIMARY KEY, data JSON NOT NULL)",
];
```

**Migration Notes**:
- Non-destructive migration (no data loss)
- Existing task_executions automatically support new fields
- JSON schema allows optional fields
- No need to migrate existing data

## Model Registration

**File**: `persistence/src/models/mod.rs`

```rust
pub mod audit;
pub mod enums;
pub mod execution;
pub mod prompt;
pub mod settings;
pub mod task;
pub mod user_input_request; // NEW
pub mod workflow;
```

## Store Registration

**File**: `persistence/src/store/mod.rs`

```rust
pub mod audit;
pub mod execution;
pub mod prompt;
pub mod settings;
pub mod task;
pub mod user_input; // NEW
pub mod utils;
pub mod workflow;
```

## Serialization Format

### UserInputRequest JSON

```json
{
    "id": "uuid-string",
    "task_execution_id": "task-uuid",
    "workflow_execution_id": "workflow-uuid",
    "prompt_text": "Please enter your name",
    "input_type": "string",
    "required": true,
    "default_value": null,
    "validation_rules": {},
    "status": "pending",
    "created_at": "2024-01-01T00:00:00Z",
    "fulfilled_at": null,
    "fulfilled_value": null
}
```

### TaskExecution with Input JSON

```json
{
    "id": "task-uuid",
    "workflow_id": "workflow-uuid",
    "name": "Get User Input",
    "status": "waiting_for_input",
    "output": null,
    "error": null,
    "created_at": "2024-01-01T00:00:00Z",
    "completed_at": null,
    "user_input": null,
    "input_request_id": "request-uuid",
    "prompt_id": "optional-prompt-uuid"
}
```

## Error Handling

### Persistence Errors

```rust
// In persistence/src/errors.rs
pub enum PersistenceError {
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Input request not found: {0}")]
    InputRequestNotFound(String),
    
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    
    #[error("Invalid input type: {0}")]
    InvalidInputType(String),
}
```

## Testing Requirements

### Test Files

1. **`persistence/tests/models/user_input_request_tests.rs`**
   - Model validation
   - Serialization/deserialization
   - Status transitions

2. **`persistence/tests/models/task_input_tests.rs`**
   - Task execution with input fields
   - Validation of input fields
   - Edge cases (null values, etc.)

3. **`persistence/tests/store/user_input_tests.rs`**
   - CRUD operations
   - Query operations
   - Status updates

4. **`persistence/tests/task_input_operations_tests.rs`**
   - Task with input save/load
   - Query tasks waiting for input
   - Integration with input requests

### Test Coverage

- Happy path: Create, read, update, delete
- Error cases: Not found, invalid data
- Edge cases: Null values, empty strings
- Concurrent operations: Multiple simultaneous requests
- Data integrity: Foreign key relationships

## Logging Requirements

### All Operations Must Log

```rust
// Example for save_input_request
log_db_operation_start("save_input_request", "user_input_requests");
log_serialization("UserInputRequest", json_data.len());
log_db_operation_success("save_input_request", "user_input_requests", 0);
```

**Log Levels**:
- `INFO`: Successful operations, status changes
- `DEBUG`: Query details, data transformations
- `TRACE`: Serialization details, parameter values
- `WARN`: Retries, degraded performance
- `ERROR`: Failures, data corruption

## Performance Considerations

### Indexes

```sql
CREATE INDEX IF NOT EXISTS idx_input_request_task_id 
ON user_input_requests(data->>'$.task_execution_id');

CREATE INDEX IF NOT EXISTS idx_input_request_workflow_id 
ON user_input_requests(data->>'$.workflow_execution_id');

CREATE INDEX IF NOT EXISTS idx_input_request_status 
ON user_input_requests(data->>'$.status');
```

### Query Optimization

- Batch queries for multiple pending inputs
- Use indexes for task/workflow lookups
- Cache fulfilled requests (future enhancement)

## Backward Compatibility

### Existing Behavior Preserved

- All existing task operations continue to work
- New fields are Optional<String> - no breaking changes
- JSON serialization handles optional fields gracefully
- Old task executions load without errors

### Migration Path

1. Run database migration (adds new table)
2. Update models with new fields (all optional)
3. Code gracefully handles missing fields
4. Old data remains valid

