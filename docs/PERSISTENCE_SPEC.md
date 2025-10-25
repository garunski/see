# Persistence Specification

## Overview
SQLx with SQLite database schema and Store implementation for multi-process concurrent access using JSON columns for key/value storage.

## Database Schema

### Table Definitions

Each table follows a simple key/value pattern with JSON storage:

#### Workflows Table
```sql
CREATE TABLE workflows (
    id TEXT PRIMARY KEY,
    data JSON NOT NULL
);
```

#### Workflow Executions Table
```sql
CREATE TABLE workflow_executions (
    id TEXT PRIMARY KEY,
    data JSON NOT NULL
);
```

#### Task Executions Table
```sql
CREATE TABLE task_executions (
    id TEXT PRIMARY KEY,
    data JSON NOT NULL
);
```

#### User Prompts Table
```sql
CREATE TABLE user_prompts (
    id TEXT PRIMARY KEY,
    data JSON NOT NULL
);
```

#### Audit Events Table
```sql
CREATE TABLE audit_events (
    id TEXT PRIMARY KEY,
    data JSON NOT NULL
);
```

#### Settings Table
```sql
CREATE TABLE settings (
    id TEXT PRIMARY KEY,
    data JSON NOT NULL
);
```

## File Organization and Single Responsibility Principle (SRP)

### File Structure Requirements
- **Each file must have a SINGLE, CLEAR responsibility**
- **NO mixing of concerns** - separate models, store operations, errors, etc.
- **Keep files small and focused** - aim for <200 lines per file
- **Logical grouping** - related functionality in same file, unrelated in separate files

### Required File Structure
```
persistence/
├── Cargo.toml
└── src/
    ├── lib.rs              # Public API exports only
    ├── models/
    │   ├── mod.rs          # Module declarations
    │   ├── workflow.rs     # WorkflowDefinition only
    │   ├── execution.rs    # WorkflowExecution, WorkflowExecutionSummary, WorkflowMetadata only
    │   ├── task.rs         # TaskExecution only
    │   ├── prompt.rs       # UserPrompt only
    │   ├── audit.rs        # AuditEvent only
    │   ├── settings.rs     # AppSettings only
    │   └── enums.rs        # All enums (WorkflowStatus, Theme, etc.)
    ├── store/
    │   ├── mod.rs          # Module declarations
    │   ├── lib.rs          # Store struct and initialization
    │   ├── workflow.rs     # Workflow CRUD operations only
    │   ├── execution.rs    # Execution CRUD operations only
    │   ├── task.rs         # Task operations only
    │   ├── prompt.rs       # Prompt operations only
    │   ├── settings.rs     # Settings operations only
    │   ├── audit.rs        # Audit operations only
    │   └── utils.rs        # Utility functions (clear_all_data) only
    ├── errors.rs           # PersistenceError only
    └── logging.rs          # Logging configuration and helpers only
```

### File Responsibility Guidelines

**models/workflow.rs** - ONLY WorkflowDefinition:
```rust
// ONLY WorkflowDefinition struct and its impl blocks
// NO other models, NO store operations, NO errors
```

**models/execution.rs** - ONLY execution-related models:
```rust
// ONLY WorkflowExecution, WorkflowExecutionSummary, WorkflowMetadata
// NO other models, NO store operations, NO errors
```

**store/workflow.rs** - ONLY workflow store operations:
```rust
// ONLY save_workflow, get_workflow, list_workflows, delete_workflow
// NO other operations, NO models, NO error definitions
```

**errors.rs** - ONLY error types:
```rust
// ONLY PersistenceError enum and its implementations
// NO models, NO store operations, NO business logic
```

### SRP Violations to Avoid
❌ **DON'T** put all models in one file
❌ **DON'T** put all store operations in one file  
❌ **DON'T** mix models with store operations
❌ **DON'T** put error types with business logic
❌ **DON'T** put logging setup with models
❌ **DON'T** put utility functions with core operations

### Benefits of SRP Compliance
- **Easier to find code** - know exactly where to look
- **Easier to test** - test one responsibility at a time
- **Easier to maintain** - changes affect minimal files
- **Easier to review** - smaller, focused files
- **Better reusability** - focused modules can be reused

## Store Implementation

### Store Struct
```rust
use sqlx::{SqlitePool, Row};
use std::sync::Arc;

pub struct Store {
    pool: Arc<SqlitePool>,
}

impl Store {
    pub async fn new(db_path: &str) -> Result<Self, PersistenceError> {
        // Enable WAL mode for better concurrency
        let pool = SqlitePool::connect_with(
            format!("sqlite:{}", db_path)
                .parse()
                .map_err(|e| PersistenceError::Database(e.to_string()))?
        ).await
        .map_err(|e| PersistenceError::Database(e.to_string()))?;

        // Enable WAL mode
        sqlx::query("PRAGMA journal_mode=WAL")
            .execute(&pool)
            .await
            .map_err(|e| PersistenceError::Database(e.to_string()))?;

        // Create tables
        Self::create_tables(&pool).await?;

        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    async fn create_tables(pool: &SqlitePool) -> Result<(), PersistenceError> {
        let tables = [
            "CREATE TABLE IF NOT EXISTS workflows (id TEXT PRIMARY KEY, data JSON NOT NULL)",
            "CREATE TABLE IF NOT EXISTS workflow_executions (id TEXT PRIMARY KEY, data JSON NOT NULL)",
            "CREATE TABLE IF NOT EXISTS task_executions (id TEXT PRIMARY KEY, data JSON NOT NULL)",
            "CREATE TABLE IF NOT EXISTS user_prompts (id TEXT PRIMARY KEY, data JSON NOT NULL)",
            "CREATE TABLE IF NOT EXISTS audit_events (id TEXT PRIMARY KEY, data JSON NOT NULL)",
            "CREATE TABLE IF NOT EXISTS settings (id TEXT PRIMARY KEY, data JSON NOT NULL)",
        ];

        for table_sql in &tables {
            sqlx::query(table_sql)
                .execute(pool)
                .await
                .map_err(|e| PersistenceError::Database(e.to_string()))?;
        }

        Ok(())
    }
}
```

### Concurrency Strategy

#### Multi-Process Reader Support
- SQLite with WAL mode enables multiple concurrent readers
- Multiple processes can read simultaneously without blocking
- Readers never block other readers
- Writers don't block readers (WAL mode provides snapshot isolation)
- Connection pooling handles concurrent access efficiently

#### Transaction Patterns
```rust
// Read operation (non-blocking)
let row = sqlx::query("SELECT data FROM workflows WHERE id = ?")
    .bind(&workflow_id)
    .fetch_optional(&self.pool)
    .await?;

// Write operation (IMMEDIATE transaction for better concurrency)
let mut tx = self.pool.begin().await?;
sqlx::query("INSERT OR REPLACE INTO workflows (id, data) VALUES (?, ?)")
    .bind(&workflow_id)
    .bind(&serde_json::to_string(&workflow)?)
    .execute(&mut *tx)
    .await?;
tx.commit().await?;
```

## Store API Methods

### Workflow Management
```rust
impl Store {
    pub async fn save_workflow(&self, workflow: &WorkflowDefinition) -> Result<(), String> {
        let json_data = serde_json::to_string(workflow)
            .map_err(|e| format!("Serialization error: {}", e))?;
        
        sqlx::query("INSERT OR REPLACE INTO workflows (id, data) VALUES (?, ?)")
            .bind(&workflow.id)
            .bind(&json_data)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        
        Ok(())
    }

    pub async fn get_workflow(&self, id: &str) -> Result<Option<WorkflowDefinition>, String> {
        let row = sqlx::query("SELECT data FROM workflows WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        match row {
            Some(row) => {
                let json_data: String = row.get("data");
                let workflow = serde_json::from_str(&json_data)
                    .map_err(|e| format!("Deserialization error: {}", e))?;
                Ok(Some(workflow))
            }
            None => Ok(None),
        }
    }

    pub async fn list_workflows(&self) -> Result<Vec<WorkflowDefinition>, String> {
        let rows = sqlx::query("SELECT data FROM workflows ORDER BY id")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let mut workflows = Vec::new();
        for row in rows {
            let json_data: String = row.get("data");
            let workflow = serde_json::from_str(&json_data)
                .map_err(|e| format!("Deserialization error: {}", e))?;
            workflows.push(workflow);
        }
        
        Ok(workflows)
    }

    pub async fn delete_workflow(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM workflows WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        
        Ok(())
    }
}
```

### Execution Management
```rust
impl Store {
    pub async fn save_workflow_execution(&self, execution: WorkflowExecution) -> Result<(), String> {
        let json_data = serde_json::to_string(&execution)
            .map_err(|e| format!("Serialization error: {}", e))?;
        
        sqlx::query("INSERT OR REPLACE INTO workflow_executions (id, data) VALUES (?, ?)")
            .bind(&execution.id)
            .bind(&json_data)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        
        Ok(())
    }

    pub async fn get_workflow_execution(&self, id: &str) -> Result<Option<WorkflowExecution>, String> {
        let row = sqlx::query("SELECT data FROM workflow_executions WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        match row {
            Some(row) => {
                let json_data: String = row.get("data");
                let execution = serde_json::from_str(&json_data)
                    .map_err(|e| format!("Deserialization error: {}", e))?;
                Ok(Some(execution))
            }
            None => Ok(None),
        }
    }

    pub async fn list_workflow_executions(&self) -> Result<Vec<WorkflowExecution>, String> {
        let rows = sqlx::query("SELECT data FROM workflow_executions ORDER BY id")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let mut executions = Vec::new();
        for row in rows {
            let json_data: String = row.get("data");
            let execution = serde_json::from_str(&json_data)
                .map_err(|e| format!("Deserialization error: {}", e))?;
            executions.push(execution);
        }
        
        Ok(executions)
    }

    pub async fn delete_workflow_execution(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM workflow_executions WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        
        Ok(())
    }

    pub async fn list_workflow_metadata(&self) -> Result<Vec<WorkflowMetadata>, String> {
        let executions = self.list_workflow_executions().await?;
        let metadata = executions.into_iter().map(|exec| WorkflowMetadata {
            id: exec.id,
            name: exec.workflow_name,
            status: exec.status.to_string(),
        }).collect();
        Ok(metadata)
    }

    pub async fn delete_workflow_metadata_and_tasks(&self, id: &str) -> Result<(), String> {
        // Delete execution
        self.delete_workflow_execution(id).await?;
        
        // Delete associated tasks
        sqlx::query("DELETE FROM task_executions WHERE workflow_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        
        Ok(())
    }

    pub async fn get_workflow_with_tasks(&self, id: &str) -> Result<WorkflowExecution, String> {
        let mut execution = self.get_workflow_execution(id).await?
            .ok_or_else(|| format!("Workflow execution not found: {}", id))?;
        
        // Load associated tasks
        let tasks = self.get_tasks_for_workflow(id).await?;
        execution.tasks = tasks;
        
        Ok(execution)
    }
}
```

### Task Management
```rust
impl Store {
    pub async fn save_task_execution(&self, task: TaskExecution) -> Result<(), String> {
        let json_data = serde_json::to_string(&task)
            .map_err(|e| format!("Serialization error: {}", e))?;
        
        sqlx::query("INSERT OR REPLACE INTO task_executions (id, data) VALUES (?, ?)")
            .bind(&task.id)
            .bind(&json_data)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        
        Ok(())
    }

    pub async fn get_tasks_for_workflow(&self, workflow_id: &str) -> Result<Vec<TaskExecution>, String> {
        let rows = sqlx::query("SELECT data FROM task_executions")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let mut tasks = Vec::new();
        for row in rows {
            let json_data: String = row.get("data");
            let task: TaskExecution = serde_json::from_str(&json_data)
                .map_err(|e| format!("Deserialization error: {}", e))?;
            
            if task.workflow_id == workflow_id {
                tasks.push(task);
            }
        }
        
        Ok(tasks)
    }
}
```

### Prompt Management
```rust
impl Store {
    pub async fn save_prompt(&self, prompt: &UserPrompt) -> Result<(), String> {
        let json_data = serde_json::to_string(prompt)
            .map_err(|e| format!("Serialization error: {}", e))?;
        
        sqlx::query("INSERT OR REPLACE INTO user_prompts (id, data) VALUES (?, ?)")
            .bind(&prompt.id)
            .bind(&json_data)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        
        Ok(())
    }

    pub async fn list_prompts(&self) -> Result<Vec<UserPrompt>, String> {
        let rows = sqlx::query("SELECT data FROM user_prompts ORDER BY id")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let mut prompts = Vec::new();
        for row in rows {
            let json_data: String = row.get("data");
            let prompt = serde_json::from_str(&json_data)
                .map_err(|e| format!("Deserialization error: {}", e))?;
            prompts.push(prompt);
        }
        
        Ok(prompts)
    }

    pub async fn delete_prompt(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM user_prompts WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        
        Ok(())
    }
}
```

### Settings Management
```rust
impl Store {
    pub async fn load_settings(&self) -> Result<Option<AppSettings>, String> {
        let row = sqlx::query("SELECT data FROM settings WHERE id = 'app_settings'")
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        match row {
            Some(row) => {
                let json_data: String = row.get("data");
                let settings = serde_json::from_str(&json_data)
                    .map_err(|e| format!("Deserialization error: {}", e))?;
                Ok(Some(settings))
            }
            None => Ok(None),
        }
    }

    pub async fn save_settings(&self, settings: &AppSettings) -> Result<(), String> {
        let json_data = serde_json::to_string(settings)
            .map_err(|e| format!("Serialization error: {}", e))?;
        
        sqlx::query("INSERT OR REPLACE INTO settings (id, data) VALUES ('app_settings', ?)")
            .bind(&json_data)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        
        Ok(())
    }
}
```

### Audit Management
```rust
impl Store {
    pub async fn log_audit_event(&self, event: AuditEvent) -> Result<(), String> {
        let json_data = serde_json::to_string(&event)
            .map_err(|e| format!("Serialization error: {}", e))?;
        
        sqlx::query("INSERT OR REPLACE INTO audit_events (id, data) VALUES (?, ?)")
            .bind(&event.id)
            .bind(&json_data)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        
        Ok(())
    }
}
```

### Utility Functions
```rust
impl Store {
    pub async fn clear_all_data(&self) -> Result<(), String> {
        let tables = [
            "workflows",
            "workflow_executions", 
            "task_executions",
            "user_prompts",
            "audit_events",
            "settings"
        ];

        for table in &tables {
            sqlx::query(&format!("DELETE FROM {}", table))
                .execute(&self.pool)
                .await
                .map_err(|e| format!("Database error: {}", e))?;
        }
        
        Ok(())
    }
}
```

## Logging and Instrumentation

### Logging Levels and Usage
- **TRACE** - Detailed execution flow, SQL query parameters, serialization/deserialization details
- **DEBUG** - Method entry/exit, data transformations, connection pool status
- **INFO** - Major operations (save/get/delete), database initialization, table creation
- **WARN** - Recoverable errors, fallback operations, deprecated usage
- **ERROR** - Unrecoverable errors, database failures, serialization failures

### Required Logging Points

**Store Initialization:**
- INFO: Database connection established
- INFO: WAL mode enabled
- INFO: Tables created successfully
- DEBUG: Connection pool configuration
- ERROR: Database connection failures

**Workflow Operations:**
- INFO: save_workflow() called with workflow_id
- DEBUG: Serialization of WorkflowDefinition
- TRACE: SQL query execution with parameters
- INFO: Workflow saved successfully
- ERROR: Serialization failures, database errors

**Execution Operations:**
- INFO: save_workflow_execution() called with execution_id
- DEBUG: Execution status and task count
- INFO: Execution saved successfully
- ERROR: Execution save failures

**Task Operations:**
- INFO: save_task_execution() called with task_id
- DEBUG: Task status and workflow_id
- INFO: Task saved successfully
- ERROR: Task save failures

**Query Operations:**
- INFO: get_workflow() called with workflow_id
- DEBUG: Query result (found/not found)
- TRACE: Deserialization process
- ERROR: Deserialization failures

**List Operations:**
- INFO: list_workflows() called
- DEBUG: Number of workflows returned
- TRACE: Deserialization of each workflow
- ERROR: Query failures

**Delete Operations:**
- INFO: delete_workflow() called with workflow_id
- DEBUG: Deletion result (rows affected)
- ERROR: Deletion failures

**Settings Operations:**
- INFO: save_settings() called
- DEBUG: Settings serialization
- INFO: Settings saved successfully
- ERROR: Settings save failures

**Audit Operations:**
- INFO: log_audit_event() called with event_id
- DEBUG: Audit event details
- INFO: Audit event logged successfully
- ERROR: Audit logging failures

**Utility Operations:**
- INFO: clear_all_data() called
- DEBUG: Tables cleared
- INFO: All data cleared successfully
- ERROR: Clear operation failures

### Logging Implementation Requirements

**Dependencies:**
- Add `tracing` crate for structured logging
- Add `tracing-sqlx` for SQL query logging
- Add `tracing-appender` for log file output

**Logging Setup:**
```rust
use tracing::{info, debug, trace, warn, error, instrument};

#[instrument(skip(self))]
pub async fn save_workflow(&self, workflow: &WorkflowDefinition) -> Result<(), String> {
    info!("Saving workflow: {}", workflow.id);
    // Implementation with logging
}
```

**Structured Logging:**
- Use structured fields for IDs, counts, durations
- Include correlation IDs for tracing operations
- Log performance metrics (query duration, serialization time)

**Error Context:**
- Always include relevant context in error logs
- Log stack traces for debugging
- Include operation parameters in error messages

### Logging Test Requirements
- All logging must be testable
- Log output must be deterministic
- Log levels must be configurable
- Logs must not contain sensitive data

## Error Types

### PersistenceError
```rust
#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Transaction error: {0}")]
    Transaction(String),
    
    #[error("Connection pool error: {0}")]
    ConnectionPool(String),
}
```

## Dependencies

### Cargo.toml
```toml
[dependencies]
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-rustls", "json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
thiserror = "1.0"
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
tracing-sqlx = "0.1"
tracing-appender = "0.2"
```

## Performance Characteristics

### Read Performance
- Connection pooling enables concurrent reads
- SQLite with WAL mode provides snapshot isolation
- JSON deserialization overhead (acceptable for key/value use case)
- Indexed primary key lookups are fast

### Write Performance
- Connection pooling handles concurrent writes
- IMMEDIATE transactions for better concurrency
- JSON serialization overhead (acceptable for key/value use case)
- WAL mode enables non-blocking reads during writes

### Multi-Process Support
- Multiple GUI processes can read simultaneously
- WAL mode eliminates file locking issues
- Process-safe concurrent access via SQLite
- Automatic conflict resolution

### Memory Usage
- Connection pool manages database connections efficiently
- JSON serialization uses temporary buffers
- No zero-copy reads (trade-off for simplicity)
- Reasonable memory footprint for embedded use case

## Testing Strategy

### Test Organization
- **All tests MUST be in separate test files** in `/tests` directory
- NO tests in the same files as implementation code
- Tests must be organized by feature/module

### Test Structure
```
persistence/
├── src/
│   ├── lib.rs
│   ├── models/
│   │   ├── workflow.rs
│   │   ├── execution.rs
│   │   ├── task.rs
│   │   ├── prompt.rs
│   │   ├── audit.rs
│   │   ├── settings.rs
│   │   └── enums.rs
│   ├── store/
│   │   ├── lib.rs
│   │   ├── workflow.rs
│   │   ├── execution.rs
│   │   ├── task.rs
│   │   ├── prompt.rs
│   │   ├── settings.rs
│   │   ├── audit.rs
│   │   └── utils.rs
│   ├── errors.rs
│   └── logging.rs
└── tests/
    ├── models/
    │   ├── workflow_tests.rs
    │   ├── execution_tests.rs
    │   ├── task_tests.rs
    │   ├── prompt_tests.rs
    │   ├── audit_tests.rs
    │   ├── settings_tests.rs
    │   └── enums_tests.rs
    ├── store/
    │   ├── workflow_tests.rs
    │   ├── execution_tests.rs
    │   ├── task_tests.rs
    │   ├── prompt_tests.rs
    │   ├── settings_tests.rs
    │   ├── audit_tests.rs
    │   └── utils_tests.rs
    ├── errors_tests.rs
    ├── logging_tests.rs
    ├── concurrency_tests.rs
    └── integration_tests.rs
```

### Test Coverage Requirements

**Model Tests** - Test each model individually:
- `tests/models/workflow_tests.rs` - WorkflowDefinition serialization, validation, defaults
- `tests/models/execution_tests.rs` - WorkflowExecution, WorkflowExecutionSummary, WorkflowMetadata
- `tests/models/task_tests.rs` - TaskExecution serialization, validation
- `tests/models/prompt_tests.rs` - UserPrompt serialization, validation
- `tests/models/settings_tests.rs` - AppSettings serialization, defaults
- `tests/models/audit_tests.rs` - AuditEvent serialization, validation
- `tests/models/enums_tests.rs` - All enum serialization, variants

**Store Tests** - Test each store module individually:
- `tests/store/workflow_tests.rs` - save_workflow, get_workflow, list_workflows, delete_workflow
- `tests/store/execution_tests.rs` - save/get/list/delete workflow executions, list_workflow_metadata, delete_workflow_metadata_and_tasks, get_workflow_with_tasks
- `tests/store/task_tests.rs` - save_task_execution, get_tasks_for_workflow
- `tests/store/prompt_tests.rs` - save_prompt, list_prompts, delete_prompt
- `tests/store/settings_tests.rs` - load_settings, save_settings (including defaults)
- `tests/store/audit_tests.rs` - log_audit_event
- `tests/store/utils_tests.rs` - clear_all_data

**Error Tests** - Test error handling:
- `tests/errors_tests.rs` - PersistenceError variants, error conversions, error messages

**Logging Tests** - Test logging functionality:
- `tests/logging_tests.rs` - Log level configuration, structured logging, error logging

**Integration Tests** - Test real-world scenarios:
- `tests/integration_tests.rs` - Complete workflow execution flow, multi-table operations, transaction rollback

**Concurrency Tests** - Test multi-process access:
- `tests/concurrency_tests.rs` - Multiple concurrent readers, write during read, connection pool limits

### Test Requirements Per Section

Each section in the spec should include:
1. **Implementation code example**
2. **Corresponding test examples** showing how to test the feature
3. **Edge cases** to test (empty results, not found, invalid data, etc.)

### Example Test Documentation Format

For each Store method, include:
```rust
// In Store API Methods section - show implementation
// In Testing Strategy section - show corresponding test

#[tokio::test]
async fn test_save_and_get_workflow() {
    let store = create_test_store().await;
    let workflow = create_test_workflow();
    
    // Test save
    store.save_workflow(&workflow).await.unwrap();
    
    // Test get
    let retrieved = store.get_workflow(&workflow.id).await.unwrap();
    assert_eq!(retrieved, Some(workflow));
}

#[tokio::test] 
async fn test_workflow_not_found() {
    let store = create_test_store().await;
    let result = store.get_workflow("nonexistent").await.unwrap();
    assert_eq!(result, None);
}
```

### Test Execution Requirements
- All tests must pass with `cargo test -p persistence`
- Tests must be isolated (each test creates its own temporary database)
- Tests must clean up resources (temp files deleted after test)
- Tests must be deterministic (no flaky tests)
- Tests must verify logging output where appropriate
- Tests must cover error conditions and edge cases

### Test Helper Functions
```rust
// In test files
async fn create_test_store() -> Store {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    Store::new(&db_path.to_string_lossy()).await.unwrap()
}

fn create_test_workflow() -> WorkflowDefinition {
    WorkflowDefinition {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Test Workflow".to_string(),
        description: Some("Test description".to_string()),
        content: r#"{"id":"test","name":"Test","tasks":[]}"#.to_string(),
        is_default: false,
        is_edited: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}
```
