# Bridge Mappings Specification

## Overview
This document defines field-by-field type conversions between engine types, persistence types, and core bridge types. These conversions are essential for the orchestration layer to coordinate between the workflow engine and the persistence layer.

## Type Conversion Overview

```
┌──────────────────────┐
│ WorkflowDefinition   │ (Persistence)
│ - content: String    │
└──────────┬───────────┘
           │ parse_workflow()
           ▼
┌──────────────────────┐
│ EngineWorkflow       │ (Engine)
│ - tasks: Vec<Task>   │
└──────────┬───────────┘
           │ execute_workflow()
           ▼
┌──────────────────────┐
│ WorkflowResult       │ (Engine)
│ - tasks: TaskInfo[]  │
└──────────┬───────────┘
           │ convert
           ▼
┌──────────────────────┐
│ WorkflowExecution    │ (Persistence)
│ - tasks: TaskExec[]  │
└──────────────────────┘
```

---

## Conversion 1: WorkflowDefinition → EngineWorkflow

### Purpose
Convert a stored workflow definition into an executable engine workflow.

### Source Type: `WorkflowDefinition` (Persistence)
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

### Target Type: `EngineWorkflow` (Engine)
```rust
pub struct EngineWorkflow {
    pub id: String,
    pub name: String,
    pub tasks: Vec<EngineTask>,
}
```

### Conversion Function
```rust
pub fn workflow_definition_to_engine(
    workflow: &WorkflowDefinition
) -> Result<EngineWorkflow, CoreError> {
    // Parse JSON content to get tasks
    let parsed = engine::parse_workflow(&workflow.content)
        .map_err(|e| CoreError::Engine(e))?;
    
    // EngineWorkflow is created by parse_workflow with:
    // - id: extracted from JSON
    // - name: extracted from JSON
    // - tasks: parsed from JSON tasks array
    
    Ok(parsed)
}
```

### Field Mapping

| WorkflowDefinition Field | EngineWorkflow Field | Conversion |
|-------------------------|---------------------|------------|
| `content` (String)      | `id`, `name`, `tasks` | Parse JSON using `engine::parse_workflow()` |
| `id` (String)           | Not used | Metadata only, not passed to engine |
| `name` (String)         | Not used | JSON content contains its own name |
| `description`           | Not used | Metadata only |
| `is_default`            | Not used | Metadata only |
| `is_edited`             | Not used | Metadata only |
| `created_at`            | Not used | Metadata only |
| `updated_at`            | Not used | Metadata only |

### JSON Content Format
The `content` field must contain valid JSON matching this structure:
```json
{
  "id": "workflow-1",
  "name": "My Workflow",
  "tasks": [
    {
      "id": "task-1",
      "name": "Task 1",
      "function": {
        "cli_command": {
          "command": "echo",
          "args": ["hello"]
        }
      },
      "next_tasks": []
    }
  ]
}
```

### Validation Rules
1. Content must be valid JSON
2. JSON must have `id`, `name`, and `tasks` fields
3. Tasks must be an array
4. Each task must have valid structure
5. No circular dependencies in task graph
6. No duplicate task IDs

### Error Cases

| Error | CoreError Variant | When |
|-------|------------------|------|
| Invalid JSON syntax | `CoreError::Engine(ParserError)` | Content is not valid JSON |
| Missing required fields | `CoreError::Engine(ParserError)` | JSON missing id/name/tasks |
| Circular dependencies | `CoreError::Engine(GraphError)` | Tasks have circular dependency |
| Duplicate task IDs | `CoreError::Engine(ParserError)` | Multiple tasks with same ID |

---

## Conversion 2: WorkflowResult → WorkflowExecution

### Purpose
Convert engine execution results into a persistent execution record.

### Source Type: `WorkflowResult` (Engine)
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

### Target Type: `WorkflowExecution` (Persistence)
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

### Conversion Function
```rust
pub fn workflow_result_to_execution(
    result: WorkflowResult,
    execution_id: String,
    created_at: DateTime<Utc>,
) -> WorkflowExecution {
    let now = chrono::Utc::now();
    
    // Convert tasks
    let task_executions = result.tasks.iter()
        .map(|task| task_info_to_execution(
            task,
            &execution_id,
            &result.per_task_logs,
            &result.errors,
            created_at,
            now,
        ))
        .collect();
    
    WorkflowExecution {
        id: execution_id,
        workflow_name: result.workflow_name,
        status: if result.success {
            WorkflowStatus::Complete
        } else {
            WorkflowStatus::Failed
        },
        created_at,
        completed_at: Some(now),
        success: result.success,
        tasks: task_executions,
        timestamp: now,
    }
}
```

### Field Mapping

| WorkflowResult Field | WorkflowExecution Field | Conversion |
|---------------------|------------------------|------------|
| N/A | `id` | New UUID generated by orchestrator |
| `workflow_name` | `workflow_name` | Direct copy |
| `success` | `status` | `true` → `Complete`, `false` → `Failed` |
| N/A | `created_at` | Timestamp saved before execution started |
| N/A | `completed_at` | Current timestamp when conversion happens |
| `success` | `success` | Direct copy |
| `tasks` | `tasks` | Convert each `TaskInfo` → `TaskExecution` |
| N/A | `timestamp` | Current timestamp |
| `audit_trail` | N/A | Saved separately as `AuditEvent` records |
| `per_task_logs` | N/A | Merged into task `output` fields |
| `errors` | N/A | Merged into task `error` fields |

### Status Conversion Logic

```rust
fn success_to_status(success: bool) -> WorkflowStatus {
    if success {
        WorkflowStatus::Complete
    } else {
        WorkflowStatus::Failed
    }
}
```

| success | status |
|---------|--------|
| `true` | `WorkflowStatus::Complete` |
| `false` | `WorkflowStatus::Failed` |

### Timestamp Generation

- `created_at`: Passed from orchestrator (set before execution)
- `completed_at`: Current time when conversion happens
- `timestamp`: Current time (same as `completed_at`)

**Note:** `timestamp` and `completed_at` are redundant in current design but both present in GUI requirements.

---

## Conversion 3: TaskInfo → TaskExecution

### Purpose
Convert engine task result into a persistent task execution record.

### Source Type: `TaskInfo` (Engine)
```rust
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    pub status: TaskStatus,
}
```

### Target Type: `TaskExecution` (Persistence)
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

### Conversion Function
```rust
pub fn task_info_to_execution(
    task: &TaskInfo,
    workflow_id: &str,
    per_task_logs: &HashMap<String, Vec<String>>,
    errors: &Vec<String>,
    workflow_created_at: DateTime<Utc>,
    workflow_completed_at: DateTime<Utc>,
) -> TaskExecution {
    // Extract output from per_task_logs
    let output = per_task_logs.get(&task.id)
        .map(|logs| logs.join("\n"))
        .filter(|s| !s.is_empty());
    
    // Extract error if task failed
    let error = if matches!(task.status, TaskStatus::Failed) {
        // Try to find task-specific error in errors vec
        errors.iter()
            .find(|e| e.contains(&task.id))
            .cloned()
            .or_else(|| Some("Task failed".to_string()))
    } else {
        None
    };
    
    // Estimate timestamps (engine doesn't provide per-task timestamps)
    let completed_at = match task.status {
        TaskStatus::Complete | TaskStatus::Failed => Some(workflow_completed_at),
        TaskStatus::WaitingForInput => None,
        _ => None,
    };
    
    TaskExecution {
        id: task.id.clone(),
        workflow_id: workflow_id.to_string(),
        name: task.name.clone(),
        status: task.status.clone(),
        output,
        error,
        created_at: workflow_created_at,
        completed_at,
    }
}
```

### Field Mapping

| TaskInfo Field | TaskExecution Field | Conversion |
|----------------|---------------------|------------|
| `id` | `id` | Direct copy |
| N/A | `workflow_id` | Execution ID from orchestrator |
| `name` | `name` | Direct copy |
| `status` | `status` | Direct copy (same enum) |
| N/A | `output` | Extract from `per_task_logs[task.id]` |
| N/A | `error` | Extract from `errors` if failed |
| N/A | `created_at` | Workflow start time (estimate) |
| N/A | `completed_at` | Workflow end time if complete (estimate) |

### Output Extraction

```rust
fn extract_output(
    task_id: &str,
    per_task_logs: &HashMap<String, Vec<String>>,
) -> Option<String> {
    per_task_logs.get(task_id)
        .map(|logs| logs.join("\n"))
        .filter(|s| !s.is_empty())
}
```

**Logic:**
1. Look up task ID in `per_task_logs` map
2. If found, join log lines with newlines
3. If empty string, return None
4. Otherwise return Some(joined_logs)

### Error Extraction

```rust
fn extract_error(
    task_id: &str,
    task_status: &TaskStatus,
    errors: &Vec<String>,
) -> Option<String> {
    if !matches!(task_status, TaskStatus::Failed) {
        return None;
    }
    
    // Try to find task-specific error
    errors.iter()
        .find(|e| e.contains(task_id))
        .cloned()
        .or_else(|| Some("Task failed".to_string()))
}
```

**Logic:**
1. If task not failed, return None
2. Search errors vec for message containing task ID
3. If found, return that error message
4. Otherwise return generic "Task failed" message

### Timestamp Estimation

**Problem:** Engine doesn't provide per-task timestamps, only overall workflow timing.

**Solution:** Estimate based on workflow timestamps and task status.

```rust
fn estimate_task_timestamps(
    status: &TaskStatus,
    workflow_created_at: DateTime<Utc>,
    workflow_completed_at: DateTime<Utc>,
) -> (DateTime<Utc>, Option<DateTime<Utc>>) {
    let created_at = workflow_created_at;
    
    let completed_at = match status {
        TaskStatus::Complete | TaskStatus::Failed => {
            Some(workflow_completed_at)
        },
        TaskStatus::WaitingForInput => None,
        TaskStatus::Pending | TaskStatus::InProgress => None,
    };
    
    (created_at, completed_at)
}
```

**Estimation Rules:**

| Task Status | created_at | completed_at |
|------------|------------|--------------|
| Complete | Workflow start | Workflow end |
| Failed | Workflow start | Workflow end |
| WaitingForInput | Workflow start | None |
| Pending | Workflow start | None |
| InProgress | Workflow start | None |

**Limitations:**
- Cannot determine actual task start time
- Cannot determine task duration
- All completed tasks show same completion time
- Future enhancement: Engine should track per-task timing

### Status Conversion

TaskStatus enum is shared between engine and persistence, so no conversion needed.

```rust
// Same enum in both crates
pub enum TaskStatus {
    Pending,
    InProgress,
    Complete,
    Failed,
    WaitingForInput,
}
```

**Direct copy:** `task.status` → `task_execution.status`

---

## Conversion 4: AuditEntry → AuditEvent

### Purpose
Convert engine audit trail entry into a persistent audit event record.

### Source Type: `AuditEntry` (Engine)
```rust
pub struct AuditEntry {
    pub task_id: String,
    pub status: AuditStatus,
    pub timestamp: String,  // RFC3339 format
    pub changes_count: usize,
    pub message: String,
}
```

### Target Type: `AuditEvent` (Persistence)
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

### Conversion Function
```rust
pub fn audit_entry_to_event(
    entry: &AuditEntry,
) -> Result<AuditEvent, CoreError> {
    // Parse RFC3339 timestamp
    let timestamp = DateTime::parse_from_rfc3339(&entry.timestamp)
        .map_err(|e| CoreError::Execution(format!("Invalid timestamp: {}", e)))?
        .with_timezone(&chrono::Utc);
    
    Ok(AuditEvent {
        id: uuid::Uuid::new_v4().to_string(),
        task_id: entry.task_id.clone(),
        status: entry.status.clone(),
        timestamp,
        changes_count: entry.changes_count,
        message: entry.message.clone(),
    })
}
```

### Field Mapping

| AuditEntry Field | AuditEvent Field | Conversion |
|------------------|------------------|------------|
| N/A | `id` | New UUID generated |
| `task_id` | `task_id` | Direct copy |
| `status` | `status` | Direct copy (same enum) |
| `timestamp` (String) | `timestamp` (DateTime) | Parse RFC3339 |
| `changes_count` | `changes_count` | Direct copy |
| `message` | `message` | Direct copy |

### Timestamp Parsing

```rust
fn parse_rfc3339_timestamp(
    timestamp_str: &str,
) -> Result<DateTime<Utc>, CoreError> {
    DateTime::parse_from_rfc3339(timestamp_str)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|e| CoreError::Execution(format!("Invalid timestamp: {}", e)))
}
```

**Format:** RFC3339 string (e.g., "2024-01-15T10:30:45Z")
**Output:** `DateTime<Utc>`

**Error Handling:**
- If parse fails, return `CoreError::Execution`
- Should not happen if engine generates valid timestamps

### Status Conversion

AuditStatus enum is shared between engine and persistence, so no conversion needed.

```rust
// Same enum in both crates
pub enum AuditStatus {
    Success,
    Failure,
}
```

**Direct copy:** `entry.status` → `event.status`

### ID Generation

Each audit event gets a new UUID to ensure uniqueness in database.

```rust
let id = uuid::Uuid::new_v4().to_string();
```

**Format:** UUID v4 as string (e.g., "550e8400-e29b-41d4-a716-446655440000")

---

## Data Mapping Tables

### Status Enum Mappings

#### WorkflowStatus

| String Representation | Enum Variant | Used When |
|----------------------|--------------|-----------|
| "pending" | `WorkflowStatus::Pending` | Queued, not started |
| "running" | `WorkflowStatus::Running` | Currently executing |
| "complete" | `WorkflowStatus::Complete` | Finished successfully |
| "failed" | `WorkflowStatus::Failed` | Finished with errors |

**Serialization:**
```rust
#[derive(Serialize, Deserialize)]
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

---

#### TaskStatus (Shared)

| String Representation | Enum Variant | Used When |
|----------------------|--------------|-----------|
| "pending" | `TaskStatus::Pending` | Queued, waiting for dependencies |
| "in_progress" | `TaskStatus::InProgress` | Currently executing |
| "complete" | `TaskStatus::Complete` | Finished successfully |
| "failed" | `TaskStatus::Failed` | Finished with error |
| "waiting_for_input" | `TaskStatus::WaitingForInput` | Paused, needs user input |

**Serialization:**
```rust
#[derive(Serialize, Deserialize)]
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

---

#### AuditStatus (Shared)

| String Representation | Enum Variant | Used When |
|----------------------|--------------|-----------|
| "success" | `AuditStatus::Success` | Audit check passed |
| "failure" | `AuditStatus::Failure` | Audit check failed |

**Serialization:**
```rust
#[derive(Serialize, Deserialize)]
pub enum AuditStatus {
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "failure")]
    Failure,
}
```

---

### Timestamp Format Conversions

#### RFC3339 String ↔ DateTime<Utc>

**String → DateTime:**
```rust
use chrono::{DateTime, Utc};

fn parse_timestamp(s: &str) -> Result<DateTime<Utc>, String> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| e.to_string())
}
```

**Examples:**
- `"2024-01-15T10:30:45Z"` → `DateTime<Utc>`
- `"2024-01-15T10:30:45.123Z"` → `DateTime<Utc>` (with milliseconds)
- `"2024-01-15T10:30:45+00:00"` → `DateTime<Utc>` (explicit timezone)

**DateTime → String:**
```rust
fn format_timestamp(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339()
}
```

**Examples:**
- `DateTime<Utc>` → `"2024-01-15T10:30:45.123456789Z"`

---

### Optional Field Handling

#### Rules for Option<T>

1. **None → None:** Preserve None values
2. **Some(empty) → None:** Empty strings become None
3. **Some(value) → Some(value):** Preserve non-empty values

**Example: Task Output**
```rust
fn normalize_optional_string(s: Option<String>) -> Option<String> {
    s.filter(|s| !s.is_empty())
}
```

**Cases:**
- `None` → `None` (no output)
- `Some("")` → `None` (empty output)
- `Some("hello")` → `Some("hello")` (has output)

---

### Collection Transformations

#### Vec<String> → Option<String>

**Use Case:** Join log lines into single output field

```rust
fn join_logs(logs: &[String]) -> Option<String> {
    if logs.is_empty() {
        None
    } else {
        Some(logs.join("\n"))
    }
}
```

**Examples:**
- `vec![]` → `None`
- `vec!["line1", "line2"]` → `Some("line1\nline2")`

---

#### HashMap<String, Vec<String>> → Vec<Option<String>>

**Use Case:** Extract per-task logs into task execution outputs

```rust
fn extract_task_outputs(
    task_ids: &[String],
    per_task_logs: &HashMap<String, Vec<String>>,
) -> Vec<Option<String>> {
    task_ids.iter()
        .map(|id| {
            per_task_logs.get(id)
                .map(|logs| logs.join("\n"))
                .filter(|s| !s.is_empty())
        })
        .collect()
}
```

**Example:**
```rust
task_ids: ["task-1", "task-2"]
per_task_logs: {
    "task-1": ["log line 1", "log line 2"],
    "task-2": []
}
→
[Some("log line 1\nlog line 2"), None]
```

---

## Transformation Rules

### 1. ID Generation Strategies

#### Execution ID
```rust
let execution_id = uuid::Uuid::new_v4().to_string();
```
- **When:** Creating new WorkflowExecution
- **Format:** UUID v4 as string
- **Example:** `"550e8400-e29b-41d4-a716-446655440000"`

#### Audit Event ID
```rust
let audit_id = uuid::Uuid::new_v4().to_string();
```
- **When:** Converting AuditEntry to AuditEvent
- **Format:** UUID v4 as string
- **Example:** `"6ba7b810-9dad-11d1-80b4-00c04fd430c8"`

#### Task IDs
- **Source:** Defined in workflow JSON
- **Strategy:** Direct copy from engine
- **No generation:** Task IDs are not generated, only copied

#### Workflow IDs
- **Source:** WorkflowDefinition.id
- **Strategy:** Direct copy
- **No generation:** Workflow IDs created when workflow saved, not during execution

---

### 2. Timestamp Conversion Rules

#### Current Time
```rust
let now = chrono::Utc::now();
```
- **When:** Setting completion timestamps
- **Format:** `DateTime<Utc>`
- **Used For:** `completed_at`, `timestamp`

#### Preserved Time
```rust
let created_at = initial_execution.created_at;
```
- **When:** Maintaining execution start time
- **Format:** `DateTime<Utc>`
- **Used For:** `created_at` in final execution

#### Parsed Time
```rust
let timestamp = DateTime::parse_from_rfc3339(s)?.with_timezone(&Utc);
```
- **When:** Converting engine RFC3339 strings
- **Format:** `DateTime<Utc>`
- **Used For:** Audit event timestamps

---

### 3. JSON Parsing Strategy

#### Workflow Content Parsing
```rust
fn parse_workflow_content(content: &str) -> Result<EngineWorkflow, CoreError> {
    engine::parse_workflow(content)
        .map_err(|e| CoreError::Engine(e.into()))
}
```

**Steps:**
1. Validate JSON syntax
2. Extract required fields (id, name, tasks)
3. Parse task structure recursively
4. Validate no circular dependencies
5. Validate no duplicate task IDs
6. Build EngineWorkflow struct

**Errors:**
- Invalid JSON → `CoreError::Engine(ParserError)`
- Missing fields → `CoreError::Engine(ParserError)`
- Invalid structure → `CoreError::Engine(ParserError)`

---

### 4. Log Extraction Rules

#### Per-Task Log Extraction
```rust
fn extract_task_log(
    task_id: &str,
    per_task_logs: &HashMap<String, Vec<String>>,
) -> Option<String> {
    per_task_logs.get(task_id)
        .filter(|logs| !logs.is_empty())
        .map(|logs| logs.join("\n"))
}
```

**Rules:**
1. Look up task ID in map
2. If not found → return None
3. If found but empty vector → return None
4. If found with logs → join with newlines and return Some

**Example:**
```rust
per_task_logs: {
    "task-1": ["Starting task", "Task complete"],
    "task-2": [],
}

extract_task_log("task-1", &per_task_logs)
→ Some("Starting task\nTask complete")

extract_task_log("task-2", &per_task_logs)
→ None

extract_task_log("task-3", &per_task_logs)
→ None
```

---

### 5. Error Aggregation Rules

#### Global Errors → Task Errors

```rust
fn find_task_error(
    task_id: &str,
    task_status: &TaskStatus,
    errors: &[String],
) -> Option<String> {
    if !matches!(task_status, TaskStatus::Failed) {
        return None;
    }
    
    // Try to find task-specific error
    errors.iter()
        .find(|e| e.contains(task_id))
        .cloned()
        .or_else(|| Some("Task failed".to_string()))
}
```

**Rules:**
1. Only extract error if task status is Failed
2. Search errors array for message containing task ID
3. If found, use specific error message
4. If not found, use generic "Task failed" message

**Example:**
```rust
task_id: "task-1"
task_status: TaskStatus::Failed
errors: [
    "Task task-1 failed: command not found",
    "Workflow has errors"
]

→ Some("Task task-1 failed: command not found")
```

```rust
task_id: "task-2"
task_status: TaskStatus::Failed
errors: [
    "Task task-1 failed: command not found",
    "Workflow has errors"
]

→ Some("Task failed")  // No task-2 specific error
```

---

## Edge Cases

### 1. Missing Optional Fields

#### Scenario: WorkflowDefinition.description is None
```rust
let workflow = WorkflowDefinition {
    description: None,
    ..Default::default()
};
```

**Handling:** 
- Engine doesn't use description field
- No impact on conversion
- Field preserved in persistence

---

#### Scenario: TaskExecution.output is None
```rust
let task = TaskExecution {
    output: None,
    ..Default::default()
};
```

**Handling:**
- GUI displays "No output" or empty
- Valid state for tasks without output
- Not an error condition

---

#### Scenario: TaskExecution.error is None
```rust
let task = TaskExecution {
    error: None,
    status: TaskStatus::Complete,
    ..Default::default()
};
```

**Handling:**
- Normal for successful tasks
- GUI doesn't display error field
- Consistent with task status

---

### 2. Empty Collections

#### Scenario: WorkflowResult.tasks is empty
```rust
let result = WorkflowResult {
    tasks: vec![],
    ..Default::default()
};
```

**Handling:**
- Valid for workflow with no tasks (unusual but allowed)
- WorkflowExecution.tasks will be empty vector
- No tasks saved to database
- Execution still marked Complete or Failed

---

#### Scenario: WorkflowResult.errors is empty
```rust
let result = WorkflowResult {
    errors: vec![],
    success: true,
    ..Default::default()
};
```

**Handling:**
- Normal for successful workflow
- No errors to display in GUI
- All task errors will be None

---

#### Scenario: WorkflowResult.per_task_logs has no entries
```rust
let result = WorkflowResult {
    per_task_logs: HashMap::new(),
    ..Default::default()
};
```

**Handling:**
- All task outputs will be None
- Valid state if tasks produce no output
- GUI displays "No output" for each task

---

### 3. Invalid JSON Content

#### Scenario: WorkflowDefinition.content is not valid JSON
```rust
let workflow = WorkflowDefinition {
    content: "not json",
    ..Default::default()
};
```

**Handling:**
```rust
let result = engine::parse_workflow(&workflow.content);
// Returns: Err(ParserError::InvalidJson(...))
// Converted to: CoreError::Engine(...)
// Execution fails before engine runs
```

**Result:**
- Error returned to caller
- No execution record created (failure before Step 5)
- User sees parse error in GUI

---

#### Scenario: WorkflowDefinition.content is empty
```rust
let workflow = WorkflowDefinition {
    content: "",
    ..Default::default()
};
```

**Handling:**
```rust
if workflow.content.is_empty() {
    return Err(CoreError::Execution("Workflow content is empty".to_string()));
}
```

**Result:**
- Early validation catches this
- Fails at Step 2 (validation)
- Clear error message to user

---

#### Scenario: JSON missing required fields
```rust
let content = r#"{"name": "test"}"#;  // Missing id and tasks
```

**Handling:**
```rust
let result = engine::parse_workflow(content);
// Returns: Err(ParserError::MissingField("id"))
// Converted to: CoreError::Engine(...)
```

**Result:**
- Parse error at Step 3
- Execution never starts
- User sees "Missing field: id" error

---

### 4. Timestamp Parsing Failures

#### Scenario: AuditEntry.timestamp is invalid format
```rust
let entry = AuditEntry {
    timestamp: "invalid",
    ..Default::default()
};
```

**Handling:**
```rust
let result = DateTime::parse_from_rfc3339(&entry.timestamp);
// Returns: Err(ParseError)
// Converted to: CoreError::Execution("Invalid timestamp: ...")
```

**Result:**
- Conversion fails
- Audit event not saved
- Error logged but doesn't stop workflow save
- Partial success: execution saved, some audit events might be lost

---

#### Scenario: Timestamp in wrong timezone
```rust
let entry = AuditEntry {
    timestamp: "2024-01-15T10:30:45+05:30",  // IST timezone
    ..Default::default()
};
```

**Handling:**
```rust
let dt = DateTime::parse_from_rfc3339(&entry.timestamp)?;
let utc_dt = dt.with_timezone(&Utc);
// Correctly converts to UTC
```

**Result:**
- Automatically converted to UTC
- Time adjusted for timezone offset
- Stored consistently in database

---

### 5. Status String Mismatches

#### Scenario: Serialized status doesn't match enum
```rust
let json = r#"{"status": "unknown_status"}"#;
let result: Result<WorkflowExecution, _> = serde_json::from_str(json);
// Returns: Err(serde error)
```

**Handling:**
- Serde deserialization fails
- Database read error
- Returns CoreError::Persistence
- Should not happen if data written by our code

**Prevention:**
- Always use enum types, never raw strings
- Serde ensures type safety
- #[serde(rename)] ensures consistent strings

---

## Testing Strategy

### Conversion Tests

```rust
#[test]
fn test_workflow_definition_to_engine() {
    let workflow = WorkflowDefinition {
        content: r#"{"id":"1","name":"test","tasks":[]}"#.to_string(),
        ..Default::default()
    };
    
    let engine_wf = workflow_definition_to_engine(&workflow).unwrap();
    assert_eq!(engine_wf.id, "1");
    assert_eq!(engine_wf.name, "test");
    assert_eq!(engine_wf.tasks.len(), 0);
}

#[test]
fn test_workflow_result_to_execution() {
    let result = WorkflowResult {
        success: true,
        workflow_name: "test".to_string(),
        tasks: vec![],
        audit_trail: vec![],
        per_task_logs: HashMap::new(),
        errors: vec![],
    };
    
    let execution = workflow_result_to_execution(
        result,
        "exec-1".to_string(),
        Utc::now(),
    );
    
    assert_eq!(execution.id, "exec-1");
    assert_eq!(execution.status, WorkflowStatus::Complete);
    assert!(execution.success);
}

#[test]
fn test_task_info_to_execution() {
    let task = TaskInfo {
        id: "task-1".to_string(),
        name: "Test Task".to_string(),
        status: TaskStatus::Complete,
    };
    
    let mut logs = HashMap::new();
    logs.insert("task-1".to_string(), vec!["output line".to_string()]);
    
    let task_exec = task_info_to_execution(
        &task,
        "workflow-1",
        &logs,
        &vec![],
        Utc::now(),
        Utc::now(),
    );
    
    assert_eq!(task_exec.workflow_id, "workflow-1");
    assert_eq!(task_exec.output, Some("output line".to_string()));
    assert_eq!(task_exec.error, None);
}
```

---

### Edge Case Tests

```rust
#[test]
fn test_empty_collections() {
    let result = WorkflowResult {
        tasks: vec![],
        errors: vec![],
        per_task_logs: HashMap::new(),
        ..Default::default()
    };
    
    let execution = workflow_result_to_execution(result, "id".into(), Utc::now());
    assert_eq!(execution.tasks.len(), 0);
}

#[test]
fn test_invalid_json_content() {
    let workflow = WorkflowDefinition {
        content: "invalid".to_string(),
        ..Default::default()
    };
    
    let result = workflow_definition_to_engine(&workflow);
    assert!(result.is_err());
}

#[test]
fn test_timestamp_parsing() {
    let entry = AuditEntry {
        timestamp: "2024-01-15T10:30:45Z".to_string(),
        ..Default::default()
    };
    
    let event = audit_entry_to_event(&entry).unwrap();
    assert!(event.timestamp.year() == 2024);
}
```

---

## Success Criteria

✓ All type conversions are field-by-field complete
✓ Data mapping tables cover all enum variants
✓ Transformation rules are explicit and unambiguous
✓ Edge cases are identified and handled
✓ Error cases are documented
✓ Optional field handling is clear
✓ Collection transformations are specified
✓ Testing strategy covers all conversions
✓ Examples demonstrate actual usage
✓ No ambiguity in implementation

