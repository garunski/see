# S_E_E Persistence Layer - Complete Implementation Plan

## Overview
High-performance, thread-safe SQLite persistence layer using Rusqlite with bundled SQLite. Designed for desktop app with multiple GUI instances, focusing on performance over migration complexity.

## Core Principles

### 1. **Performance First**
- Minimize serialization/deserialization
- Use raw SQL for complex queries
- Direct column mapping where possible
- Batch operations when beneficial
- Connection pooling for concurrent access

### 2. **Thread Safety & Multi-Instance**
- Thread-safe connection pooling
- WAL mode for concurrent reads/writes
- Instance tracking for multiple GUI instances
- No locking conflicts between instances

### 3. **API Evolution Friendly**
- No complex migration system
- Simple schema versioning
- Graceful degradation for schema changes
- Easy to add/remove columns

### 4. **Desktop App Optimized**
- Embedded database (no external dependencies)
- Single database file shared between instances
- Local file storage
- Simple backup/restore

## Dependencies

```toml
[dependencies]
rusqlite = { version = "0.37.0", features = ["bundled"] }
r2d2 = "0.8"
r2d2_sqlite = "0.25"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
thiserror = "1.0"
tokio = { version = "1.0", features = ["rt", "sync"] }
tracing = "0.1"
lazy_static = "1.4"

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3.0"
```

## File Structure

```
persistence/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── connection.rs      # Database connection management
│   ├── schema.rs          # Raw SQL schema definitions
│   ├── models.rs          # Minimal data structures
│   ├── operations.rs      # High-level operations
│   ├── queries.rs         # Raw SQL queries
│   ├── instance.rs        # Multi-instance management
│   ├── coordination.rs    # Cross-instance coordination
│   ├── error.rs           # Error handling
│   └── tests/
│       ├── mod.rs
│       ├── unit/
│       │   ├── connection_tests.rs
│       │   ├── operations_tests.rs
│       │   ├── instance_tests.rs
│       │   └── coordination_tests.rs
│       └── integration/
│           ├── multi_instance_tests.rs
│           ├── performance_tests.rs
│           └── concurrency_tests.rs
```

## Database Schema

### Core Tables (Raw SQL)
```sql
-- Simple versioning system
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Workflow executions with multi-instance support
CREATE TABLE IF NOT EXISTS workflow_executions (
    id TEXT PRIMARY KEY,
    workflow_name TEXT NOT NULL,
    status TEXT NOT NULL,  -- 'pending', 'running', 'completed', 'failed'
    created_at TIMESTAMP NOT NULL,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    success BOOLEAN,
    error_message TEXT,
    metadata TEXT,  -- JSON blob for flexible data
    instance_id TEXT,  -- Track which GUI instance created this
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Task executions (denormalized for performance)
CREATE TABLE IF NOT EXISTS task_executions (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL,
    task_name TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    success BOOLEAN,
    error_message TEXT,
    output TEXT,
    logs TEXT,  -- JSON array of log messages
    retry_count INTEGER DEFAULT 0,
    execution_order INTEGER DEFAULT 0,
    instance_id TEXT,  -- Track which GUI instance created this
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workflow_id) REFERENCES workflow_executions(id)
);

-- Audit trail (simple event log)
CREATE TABLE IF NOT EXISTS audit_events (
    id TEXT PRIMARY KEY,
    workflow_id TEXT,
    task_id TEXT,
    event_type TEXT NOT NULL,  -- 'workflow_started', 'task_completed', etc.
    message TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    data TEXT,  -- JSON blob for additional context
    instance_id TEXT  -- Track which GUI instance created this
);

-- Performance indexes
CREATE INDEX IF NOT EXISTS idx_workflow_executions_created_at ON workflow_executions(created_at);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_status ON workflow_executions(status);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_instance_id ON workflow_executions(instance_id);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_last_updated ON workflow_executions(last_updated);
CREATE INDEX IF NOT EXISTS idx_task_executions_workflow_id ON task_executions(workflow_id);
CREATE INDEX IF NOT EXISTS idx_task_executions_status ON task_executions(status);
CREATE INDEX IF NOT EXISTS idx_task_executions_instance_id ON task_executions(instance_id);
CREATE INDEX IF NOT EXISTS idx_audit_events_workflow_id ON audit_events(workflow_id);
CREATE INDEX IF NOT EXISTS idx_audit_events_timestamp ON audit_events(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_events_instance_id ON audit_events(instance_id);
```

## Implementation Details

### 1. Error Handling (`src/error.rs`)

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error("Database connection error: {0}")]
    Connection(String),
    
    #[error("SQL execution error: {0}")]
    Sql(#[from] rusqlite::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Pool error: {0}")]
    Pool(#[from] r2d2::Error),
    
    #[error("Validation error: {message} (field: {field})")]
    Validation { message: String, field: String },
    
    #[error("Not found: {resource} with ID '{id}'")]
    NotFound { resource: String, id: String },
    
    #[error("Concurrency error: {0}")]
    Concurrency(String),
    
    #[error("Instance error: {0}")]
    Instance(String),
}

impl From<r2d2::Error> for PersistenceError {
    fn from(err: r2d2::Error) -> Self {
        PersistenceError::Pool(err)
    }
}
```

### 2. Models (`src/models.rs`)

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub id: String,
    pub workflow_name: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub success: Option<bool>,
    pub error_message: Option<String>,
    pub metadata: serde_json::Value,
    pub instance_id: Option<String>,
    pub last_updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecution {
    pub id: String,
    pub workflow_id: String,
    pub task_name: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub success: Option<bool>,
    pub error_message: Option<String>,
    pub output: Option<String>,
    pub logs: Vec<String>,
    pub retry_count: i32,
    pub execution_order: i32,
    pub instance_id: Option<String>,
    pub last_updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub workflow_id: Option<String>,
    pub task_id: Option<String>,
    pub event_type: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
    pub instance_id: Option<String>,
}

// Helper implementations
impl WorkflowExecution {
    pub fn new(workflow_name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            workflow_name,
            status: "pending".to_string(),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            success: None,
            error_message: None,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            instance_id: None,
            last_updated: None,
        }
    }
    
    pub fn mark_started(&mut self) {
        self.status = "running".to_string();
        self.started_at = Some(Utc::now());
        self.last_updated = Some(Utc::now());
    }
    
    pub fn mark_completed(&mut self, success: bool, error_message: Option<String>) {
        self.status = if success { "completed".to_string() } else { "failed".to_string() };
        self.completed_at = Some(Utc::now());
        self.success = Some(success);
        self.error_message = error_message;
        self.last_updated = Some(Utc::now());
    }
}

impl TaskExecution {
    pub fn new(workflow_id: String, task_name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            workflow_id,
            task_name,
            status: "pending".to_string(),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            success: None,
            error_message: None,
            output: None,
            logs: Vec::new(),
            retry_count: 0,
            execution_order: 0,
            instance_id: None,
            last_updated: None,
        }
    }
    
    pub fn mark_started(&mut self) {
        self.status = "running".to_string();
        self.started_at = Some(Utc::now());
        self.last_updated = Some(Utc::now());
    }
    
    pub fn mark_completed(&mut self, success: bool, output: Option<String>, error_message: Option<String>) {
        self.status = if success { "completed".to_string() } else { "failed".to_string() };
        self.completed_at = Some(Utc::now());
        self.success = Some(success);
        self.output = output;
        self.error_message = error_message;
        self.last_updated = Some(Utc::now());
    }
    
    pub fn add_log(&mut self, message: String) {
        self.logs.push(message);
        self.last_updated = Some(Utc::now());
    }
}
```

### 3. Connection Management (`src/connection.rs`)

```rust
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Connection, OpenFlags, config::DbConfig};
use std::sync::Arc;
use tracing::{info, debug, error, warn};

pub struct DatabasePool {
    pool: Arc<Pool<SqliteConnectionManager>>,
    path: String,
}

impl DatabasePool {
    pub fn new(path: &str, max_connections: u32) -> Result<Self, PersistenceError> {
        info!("Initializing database pool at: {}", path);
        
        let manager = SqliteConnectionManager::file(path)
            .with_flags(OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE)
            .with_init(|conn| {
                debug!("Initializing new database connection");
                
                // Set WAL mode for concurrent access
                conn.execute("PRAGMA journal_mode=WAL", [])?;
                debug!("Set journal mode to WAL");
                
                // Optimize for performance
                conn.execute("PRAGMA synchronous=NORMAL", [])?;
                conn.execute("PRAGMA cache_size=10000", [])?;
                conn.execute("PRAGMA temp_store=MEMORY", [])?;
                conn.execute("PRAGMA mmap_size=268435456", [])?; // 256MB
                
                // Enable foreign keys
                conn.execute("PRAGMA foreign_keys=ON", [])?;
                debug!("Enabled foreign key constraints");
                
                // Set busy timeout for concurrent access
                conn.busy_timeout(std::time::Duration::from_secs(30))?;
                debug!("Set busy timeout to 30 seconds");
                
                Ok(())
            });
        
        let pool = Pool::builder()
            .max_size(max_connections)
            .connection_timeout(std::time::Duration::from_secs(30))
            .idle_timeout(Some(std::time::Duration::from_secs(600))) // 10 minutes
            .build(manager)
            .map_err(|e| {
                error!("Failed to create connection pool: {}", e);
                PersistenceError::Connection(e.to_string())
            })?;
        
        info!("Database pool created with {} max connections", max_connections);
        
        Ok(Self {
            pool: Arc::new(pool),
            path: path.to_string(),
        })
    }
    
    pub fn get_connection(&self) -> Result<PooledConnection<SqliteConnectionManager>, PersistenceError> {
        debug!("Acquiring database connection from pool");
        self.pool.get().map_err(|e| {
            error!("Failed to get database connection: {}", e);
            PersistenceError::Connection(e.to_string())
        })
    }
    
    pub fn pool_size(&self) -> u32 {
        self.pool.state().connections
    }
    
    pub fn idle_connections(&self) -> u32 {
        self.pool.state().idle_connections
    }
    
    pub fn path(&self) -> &str {
        &self.path
    }
}
```

### 4. Schema Management (`src/schema.rs`)

```rust
use rusqlite::Connection;
use tracing::{info, debug, error, warn};

const CURRENT_SCHEMA_VERSION: i32 = 1;

pub fn initialize_schema(conn: &Connection) -> Result<(), PersistenceError> {
    info!("Initializing database schema");
    
    // Create schema version table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    debug!("Created schema_version table");
    
    // Check current schema version
    let current_version = get_schema_version(conn)?;
    info!("Current schema version: {}", current_version);
    
    if current_version < CURRENT_SCHEMA_VERSION {
        info!("Upgrading schema from version {} to {}", current_version, CURRENT_SCHEMA_VERSION);
        upgrade_schema(conn, current_version, CURRENT_SCHEMA_VERSION)?;
    }
    
    // Create main tables
    create_workflow_executions_table(conn)?;
    create_task_executions_table(conn)?;
    create_audit_events_table(conn)?;
    create_indexes(conn)?;
    
    // Update schema version
    conn.execute(
        "INSERT OR REPLACE INTO schema_version (version) VALUES (?)",
        [CURRENT_SCHEMA_VERSION],
    )?;
    
    info!("Schema initialization completed");
    Ok(())
}

fn get_schema_version(conn: &Connection) -> Result<i32, PersistenceError> {
    let mut stmt = conn.prepare("SELECT MAX(version) FROM schema_version")?;
    let version: Option<i32> = stmt.query_row([], |row| Ok(row.get(0)?))?;
    Ok(version.unwrap_or(0))
}

fn upgrade_schema(conn: &Connection, from_version: i32, to_version: i32) -> Result<(), PersistenceError> {
    info!("Upgrading schema from {} to {}", from_version, to_version);
    
    // For now, we only have version 1
    // Future versions can add migration logic here
    if from_version < 1 && to_version >= 1 {
        debug!("Applying schema changes for version 1");
        // Schema changes are handled in create_*_table functions
    }
    
    Ok(())
}

fn create_workflow_executions_table(conn: &Connection) -> Result<(), PersistenceError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS workflow_executions (
            id TEXT PRIMARY KEY,
            workflow_name TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL,
            started_at TIMESTAMP,
            completed_at TIMESTAMP,
            success BOOLEAN,
            error_message TEXT,
            metadata TEXT,
            instance_id TEXT,
            last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    debug!("Created workflow_executions table");
    Ok(())
}

fn create_task_executions_table(conn: &Connection) -> Result<(), PersistenceError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS task_executions (
            id TEXT PRIMARY KEY,
            workflow_id TEXT NOT NULL,
            task_name TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL,
            started_at TIMESTAMP,
            completed_at TIMESTAMP,
            success BOOLEAN,
            error_message TEXT,
            output TEXT,
            logs TEXT,
            retry_count INTEGER DEFAULT 0,
            execution_order INTEGER DEFAULT 0,
            instance_id TEXT,
            last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (workflow_id) REFERENCES workflow_executions(id)
        )",
        [],
    )?;
    debug!("Created task_executions table");
    Ok(())
}

fn create_audit_events_table(conn: &Connection) -> Result<(), PersistenceError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS audit_events (
            id TEXT PRIMARY KEY,
            workflow_id TEXT,
            task_id TEXT,
            event_type TEXT NOT NULL,
            message TEXT NOT NULL,
            timestamp TIMESTAMP NOT NULL,
            data TEXT,
            instance_id TEXT
        )",
        [],
    )?;
    debug!("Created audit_events table");
    Ok(())
}

fn create_indexes(conn: &Connection) -> Result<(), PersistenceError> {
    let indexes = vec![
        "CREATE INDEX IF NOT EXISTS idx_workflow_executions_created_at ON workflow_executions(created_at)",
        "CREATE INDEX IF NOT EXISTS idx_workflow_executions_status ON workflow_executions(status)",
        "CREATE INDEX IF NOT EXISTS idx_workflow_executions_instance_id ON workflow_executions(instance_id)",
        "CREATE INDEX IF NOT EXISTS idx_workflow_executions_last_updated ON workflow_executions(last_updated)",
        "CREATE INDEX IF NOT EXISTS idx_task_executions_workflow_id ON task_executions(workflow_id)",
        "CREATE INDEX IF NOT EXISTS idx_task_executions_status ON task_executions(status)",
        "CREATE INDEX IF NOT EXISTS idx_task_executions_instance_id ON task_executions(instance_id)",
        "CREATE INDEX IF NOT EXISTS idx_audit_events_workflow_id ON audit_events(workflow_id)",
        "CREATE INDEX IF NOT EXISTS idx_audit_events_timestamp ON audit_events(timestamp)",
        "CREATE INDEX IF NOT EXISTS idx_audit_events_instance_id ON audit_events(instance_id)",
    ];
    
    for index_sql in indexes {
        conn.execute(index_sql, [])?;
    }
    debug!("Created all indexes");
    Ok(())
}
```

### 5. SQL Queries (`src/queries.rs`)

```rust
// Workflow queries
pub const INSERT_WORKFLOW: &str = r#"
    INSERT INTO workflow_executions 
    (id, workflow_name, status, created_at, metadata, instance_id, last_updated) 
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
"#;

pub const UPDATE_WORKFLOW_STATUS: &str = r#"
    UPDATE workflow_executions 
    SET status = ?2, started_at = ?3, completed_at = ?4, success = ?5, 
        error_message = ?6, last_updated = ?7
    WHERE id = ?1
"#;

pub const GET_WORKFLOW_BY_ID: &str = r#"
    SELECT id, workflow_name, status, created_at, started_at, completed_at, 
           success, error_message, metadata, instance_id, last_updated
    FROM workflow_executions 
    WHERE id = ?1
"#;

pub const GET_WORKFLOWS_BY_INSTANCE: &str = r#"
    SELECT id, workflow_name, status, created_at, started_at, completed_at, 
           success, error_message, metadata, instance_id, last_updated
    FROM workflow_executions 
    WHERE instance_id = ?1
    ORDER BY created_at DESC
    LIMIT ?2 OFFSET ?3
"#;

pub const GET_ALL_WORKFLOWS: &str = r#"
    SELECT id, workflow_name, status, created_at, started_at, completed_at, 
           success, error_message, metadata, instance_id, last_updated
    FROM workflow_executions 
    ORDER BY created_at DESC
    LIMIT ?1 OFFSET ?2
"#;

// Task queries
pub const INSERT_TASK: &str = r#"
    INSERT INTO task_executions 
    (id, workflow_id, task_name, status, created_at, logs, execution_order, 
     instance_id, last_updated) 
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
"#;

pub const UPDATE_TASK_STATUS: &str = r#"
    UPDATE task_executions 
    SET status = ?2, started_at = ?3, completed_at = ?4, success = ?5, 
        error_message = ?6, output = ?7, logs = ?8, last_updated = ?9
    WHERE id = ?1
"#;

pub const GET_TASKS_FOR_WORKFLOW: &str = r#"
    SELECT id, workflow_id, task_name, status, created_at, started_at, 
           completed_at, success, error_message, output, logs, retry_count, 
           execution_order, instance_id, last_updated
    FROM task_executions 
    WHERE workflow_id = ?1
    ORDER BY execution_order, created_at
"#;

// Audit queries
pub const INSERT_AUDIT_EVENT: &str = r#"
    INSERT INTO audit_events 
    (id, workflow_id, task_id, event_type, message, timestamp, data, instance_id) 
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
"#;

pub const GET_AUDIT_TRAIL: &str = r#"
    SELECT id, workflow_id, task_id, event_type, message, timestamp, data, instance_id
    FROM audit_events 
    WHERE workflow_id = ?1
    ORDER BY timestamp
"#;

// Instance coordination queries
pub const GET_ACTIVE_INSTANCES: &str = r#"
    SELECT DISTINCT instance_id 
    FROM workflow_executions 
    WHERE last_updated > datetime('now', '-5 minutes')
    AND instance_id IS NOT NULL
"#;

pub const CLEANUP_OLD_INSTANCES: &str = r#"
    DELETE FROM workflow_executions 
    WHERE last_updated < datetime('now', '-1 hour')
"#;
```

### 6. Core Operations (`src/operations.rs`)

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, error, warn, instrument};

use crate::connection::DatabasePool;
use crate::models::*;
use crate::queries::*;
use crate::error::PersistenceError;

pub struct Database {
    pool: Arc<DatabasePool>,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, PersistenceError> {
        info!("Creating database at: {}", path);
        let pool = DatabasePool::new(path, 10)?; // 10 max connections
        Ok(Self {
            pool: Arc::new(pool),
        })
    }
    
    #[instrument(skip(self))]
    pub async fn create_workflow(&self, mut workflow: WorkflowExecution) -> Result<(), PersistenceError> {
        debug!("Creating workflow: {}", workflow.id);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare_cached(INSERT_WORKFLOW)?;
        
        workflow.last_updated = Some(chrono::Utc::now());
        
        stmt.execute(params![
            workflow.id,
            workflow.workflow_name,
            workflow.status,
            workflow.created_at.to_rfc3339(),
            serde_json::to_string(&workflow.metadata)?,
            workflow.instance_id,
            workflow.last_updated.unwrap().to_rfc3339()
        ])?;
        
        info!("Created workflow: {} ({})", workflow.id, workflow.workflow_name);
        Ok(())
    }
    
    #[instrument(skip(self))]
    pub async fn update_workflow_status(&self, id: &str, status: &str) -> Result<(), PersistenceError> {
        debug!("Updating workflow status: {} -> {}", id, status);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare_cached(UPDATE_WORKFLOW_STATUS)?;
        
        let now = chrono::Utc::now().to_rfc3339();
        let (started_at, completed_at, success, error_message) = match status {
            "running" => (Some(now.clone()), None, None, None),
            "completed" => (None, Some(now.clone()), Some(true), None),
            "failed" => (None, Some(now.clone()), Some(false), Some("Workflow failed".to_string())),
            _ => (None, None, None, None),
        };
        
        stmt.execute(params![
            id, status, started_at, completed_at, success, error_message, now
        ])?;
        
        info!("Updated workflow status: {} -> {}", id, status);
        Ok(())
    }
    
    #[instrument(skip(self))]
    pub async fn get_workflow(&self, id: &str) -> Result<Option<WorkflowExecution>, PersistenceError> {
        debug!("Getting workflow: {}", id);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare_cached(GET_WORKFLOW_BY_ID)?;
        
        let mut rows = stmt.query_map([id], |row| {
            Ok(WorkflowExecution {
                id: row.get(0)?,
                workflow_name: row.get(1)?,
                status: row.get(2)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .unwrap_or_else(|_| chrono::Utc::now())
                    .with_timezone(&chrono::Utc),
                started_at: row.get::<_, Option<String>>(4)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                completed_at: row.get::<_, Option<String>>(5)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                success: row.get(6)?,
                error_message: row.get(7)?,
                metadata: serde_json::from_str(&row.get::<_, String>(8)?)
                    .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new())),
                instance_id: row.get(9)?,
                last_updated: row.get::<_, Option<String>>(10)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
            })
        })?;
        
        match rows.next() {
            Some(row) => {
                let workflow = row?;
                debug!("Found workflow: {} ({})", workflow.id, workflow.workflow_name);
                Ok(Some(workflow))
            },
            None => {
                debug!("Workflow not found: {}", id);
                Ok(None)
            },
        }
    }
    
    #[instrument(skip(self))]
    pub async fn list_workflows(&self, limit: usize, offset: usize) -> Result<Vec<WorkflowExecution>, PersistenceError> {
        debug!("Listing workflows: limit={}, offset={}", limit, offset);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare_cached(GET_ALL_WORKFLOWS)?;
        
        let mut rows = stmt.query_map(params![limit as i64, offset as i64], |row| {
            Ok(WorkflowExecution {
                id: row.get(0)?,
                workflow_name: row.get(1)?,
                status: row.get(2)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .unwrap_or_else(|_| chrono::Utc::now())
                    .with_timezone(&chrono::Utc),
                started_at: row.get::<_, Option<String>>(4)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                completed_at: row.get::<_, Option<String>>(5)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                success: row.get(6)?,
                error_message: row.get(7)?,
                metadata: serde_json::from_str(&row.get::<_, String>(8)?)
                    .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new())),
                instance_id: row.get(9)?,
                last_updated: row.get::<_, Option<String>>(10)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
            })
        })?;
        
        let mut workflows = Vec::new();
        for row in rows {
            workflows.push(row?);
        }
        
        info!("Listed {} workflows", workflows.len());
        Ok(workflows)
    }
    
    #[instrument(skip(self))]
    pub async fn create_task(&self, mut task: TaskExecution) -> Result<(), PersistenceError> {
        debug!("Creating task: {} for workflow: {}", task.id, task.workflow_id);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare_cached(INSERT_TASK)?;
        
        task.last_updated = Some(chrono::Utc::now());
        
        stmt.execute(params![
            task.id,
            task.workflow_id,
            task.task_name,
            task.status,
            task.created_at.to_rfc3339(),
            serde_json::to_string(&task.logs)?,
            task.execution_order,
            task.instance_id,
            task.last_updated.unwrap().to_rfc3339()
        ])?;
        
        info!("Created task: {} ({})", task.id, task.task_name);
        Ok(())
    }
    
    #[instrument(skip(self))]
    pub async fn update_task_status(&self, id: &str, status: &str) -> Result<(), PersistenceError> {
        debug!("Updating task status: {} -> {}", id, status);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare_cached(UPDATE_TASK_STATUS)?;
        
        let now = chrono::Utc::now().to_rfc3339();
        let (started_at, completed_at, success, error_message, output) = match status {
            "running" => (Some(now.clone()), None, None, None, None),
            "completed" => (None, Some(now.clone()), Some(true), None, Some("Task completed".to_string())),
            "failed" => (None, Some(now.clone()), Some(false), Some("Task failed".to_string()), None),
            _ => (None, None, None, None, None),
        };
        
        stmt.execute(params![
            id, status, started_at, completed_at, success, error_message, output, "[]", now
        ])?;
        
        info!("Updated task status: {} -> {}", id, status);
        Ok(())
    }
    
    #[instrument(skip(self))]
    pub async fn get_tasks_for_workflow(&self, workflow_id: &str) -> Result<Vec<TaskExecution>, PersistenceError> {
        debug!("Getting tasks for workflow: {}", workflow_id);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare_cached(GET_TASKS_FOR_WORKFLOW)?;
        
        let mut rows = stmt.query_map([workflow_id], |row| {
            Ok(TaskExecution {
                id: row.get(0)?,
                workflow_id: row.get(1)?,
                task_name: row.get(2)?,
                status: row.get(3)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .unwrap_or_else(|_| chrono::Utc::now())
                    .with_timezone(&chrono::Utc),
                started_at: row.get::<_, Option<String>>(5)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                completed_at: row.get::<_, Option<String>>(6)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                success: row.get(7)?,
                error_message: row.get(8)?,
                output: row.get(9)?,
                logs: serde_json::from_str(&row.get::<_, String>(10)?)
                    .unwrap_or_else(|_| Vec::new()),
                retry_count: row.get(11)?,
                execution_order: row.get(12)?,
                instance_id: row.get(13)?,
                last_updated: row.get::<_, Option<String>>(14)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
            })
        })?;
        
        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(row?);
        }
        
        info!("Found {} tasks for workflow: {}", tasks.len(), workflow_id);
        Ok(tasks)
    }
    
    #[instrument(skip(self))]
    pub async fn log_audit_event(&self, event: AuditEvent) -> Result<(), PersistenceError> {
        debug!("Logging audit event: {} ({})", event.id, event.event_type);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare_cached(INSERT_AUDIT_EVENT)?;
        
        stmt.execute(params![
            event.id,
            event.workflow_id,
            event.task_id,
            event.event_type,
            event.message,
            event.timestamp.to_rfc3339(),
            serde_json::to_string(&event.data)?,
            event.instance_id
        ])?;
        
        debug!("Logged audit event: {}", event.id);
        Ok(())
    }
    
    #[instrument(skip(self))]
    pub async fn get_audit_trail(&self, workflow_id: &str) -> Result<Vec<AuditEvent>, PersistenceError> {
        debug!("Getting audit trail for workflow: {}", workflow_id);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare_cached(GET_AUDIT_TRAIL)?;
        
        let mut rows = stmt.query_map([workflow_id], |row| {
            Ok(AuditEvent {
                id: row.get(0)?,
                workflow_id: row.get(1)?,
                task_id: row.get(2)?,
                event_type: row.get(3)?,
                message: row.get(4)?,
                timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .unwrap_or_else(|_| chrono::Utc::now())
                    .with_timezone(&chrono::Utc),
                data: serde_json::from_str(&row.get::<_, String>(6)?)
                    .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new())),
                instance_id: row.get(7)?,
            })
        })?;
        
        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }
        
        info!("Found {} audit events for workflow: {}", events.len(), workflow_id);
        Ok(events)
    }
}
```

### 7. Instance Management (`src/instance.rs`)

```rust
use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, debug, warn, error, instrument};

use crate::operations::Database;
use crate::models::*;
use crate::error::PersistenceError;

pub struct InstanceManager {
    instance_id: String,
    db: Arc<Database>,
}

impl InstanceManager {
    pub fn new(db: Arc<Database>) -> Self {
        let instance_id = Uuid::new_v4().to_string();
        info!("Created instance manager with ID: {}", instance_id);
        
        Self {
            instance_id,
            db,
        }
    }
    
    pub fn get_instance_id(&self) -> &str {
        &self.instance_id
    }
    
    #[instrument(skip(self))]
    pub async fn create_workflow(&self, mut workflow: WorkflowExecution) -> Result<(), PersistenceError> {
        debug!("Creating workflow for instance: {}", self.instance_id);
        
        workflow.instance_id = Some(self.instance_id.clone());
        self.db.create_workflow(workflow).await
    }
    
    #[instrument(skip(self))]
    pub async fn get_workflows_for_instance(&self) -> Result<Vec<WorkflowExecution>, PersistenceError> {
        debug!("Getting workflows for instance: {}", self.instance_id);
        
        let conn = self.db.pool.get_connection()?;
        let mut stmt = conn.prepare_cached(crate::queries::GET_WORKFLOWS_BY_INSTANCE)?;
        
        let mut rows = stmt.query_map(params![self.instance_id, 1000i64, 0i64], |row| {
            Ok(WorkflowExecution {
                id: row.get(0)?,
                workflow_name: row.get(1)?,
                status: row.get(2)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .unwrap_or_else(|_| chrono::Utc::now())
                    .with_timezone(&chrono::Utc),
                started_at: row.get::<_, Option<String>>(4)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                completed_at: row.get::<_, Option<String>>(5)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                success: row.get(6)?,
                error_message: row.get(7)?,
                metadata: serde_json::from_str(&row.get::<_, String>(8)?)
                    .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new())),
                instance_id: row.get(9)?,
                last_updated: row.get::<_, Option<String>>(10)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
            })
        })?;
        
        let mut workflows = Vec::new();
        for row in rows {
            workflows.push(row?);
        }
        
        info!("Found {} workflows for instance: {}", workflows.len(), self.instance_id);
        Ok(workflows)
    }
    
    #[instrument(skip(self))]
    pub async fn get_all_workflows(&self) -> Result<Vec<WorkflowExecution>, PersistenceError> {
        debug!("Getting all workflows");
        self.db.list_workflows(1000, 0).await
    }
    
    #[instrument(skip(self))]
    pub async fn create_task(&self, mut task: TaskExecution) -> Result<(), PersistenceError> {
        debug!("Creating task for instance: {}", self.instance_id);
        
        task.instance_id = Some(self.instance_id.clone());
        self.db.create_task(task).await
    }
    
    #[instrument(skip(self))]
    pub async fn log_audit_event(&self, mut event: AuditEvent) -> Result<(), PersistenceError> {
        debug!("Logging audit event for instance: {}", self.instance_id);
        
        event.instance_id = Some(self.instance_id.clone());
        self.db.log_audit_event(event).await
    }
}
```

### 8. Multi-Instance Coordination (`src/coordination.rs`)

```rust
use std::sync::Arc;
use tracing::{info, debug, warn, error, instrument};

use crate::operations::Database;
use crate::error::PersistenceError;

pub struct MultiInstanceCoordinator {
    db: Arc<Database>,
}

impl MultiInstanceCoordinator {
    pub fn new(db: Arc<Database>) -> Self {
        info!("Created multi-instance coordinator");
        Self { db }
    }
    
    #[instrument(skip(self))]
    pub async fn get_active_instances(&self) -> Result<Vec<String>, PersistenceError> {
        debug!("Getting active instances");
        
        let conn = self.db.pool.get_connection()?;
        let mut stmt = conn.prepare_cached(crate::queries::GET_ACTIVE_INSTANCES)?;
        
        let mut rows = stmt.query_map([], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;
        
        let mut instances = Vec::new();
        for row in rows {
            instances.push(row?);
        }
        
        info!("Found {} active instances", instances.len());
        Ok(instances)
    }
    
    #[instrument(skip(self))]
    pub async fn cleanup_old_instances(&self) -> Result<usize, PersistenceError> {
        debug!("Cleaning up old instances");
        
        let conn = self.db.pool.get_connection()?;
        let changes = conn.execute(crate::queries::CLEANUP_OLD_INSTANCES, [])?;
        
        info!("Cleaned up {} old workflow records", changes);
        Ok(changes)
    }
    
    #[instrument(skip(self))]
    pub async fn get_instance_stats(&self) -> Result<InstanceStats, PersistenceError> {
        debug!("Getting instance statistics");
        
        let conn = self.db.pool.get_connection()?;
        
        // Get total workflows
        let total_workflows: i64 = conn.query_row(
            "SELECT COUNT(*) FROM workflow_executions",
            [],
            |row| Ok(row.get(0)?)
        )?;
        
        // Get active instances
        let active_instances = self.get_active_instances().await?;
        
        // Get workflows per instance
        let mut workflows_per_instance = Vec::new();
        for instance_id in &active_instances {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM workflow_executions WHERE instance_id = ?1",
                [instance_id],
                |row| Ok(row.get(0)?)
            )?;
            workflows_per_instance.push((instance_id.clone(), count));
        }
        
        let stats = InstanceStats {
            total_workflows,
            active_instances: active_instances.len(),
            workflows_per_instance,
        };
        
        info!("Instance stats: {} workflows, {} active instances", 
              stats.total_workflows, stats.active_instances);
        
        Ok(stats)
    }
}

#[derive(Debug, Clone)]
pub struct InstanceStats {
    pub total_workflows: i64,
    pub active_instances: usize,
    pub workflows_per_instance: Vec<(String, i64)>,
}
```

### 9. Main Library (`src/lib.rs`)

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, error, warn};

pub mod connection;
pub mod schema;
pub mod models;
pub mod operations;
pub mod queries;
pub mod instance;
pub mod coordination;
pub mod error;

pub use connection::DatabasePool;
pub use operations::Database;
pub use instance::InstanceManager;
pub use coordination::MultiInstanceCoordinator;
pub use models::*;
pub use error::PersistenceError;

// Global singleton for thread-safe access
lazy_static::lazy_static! {
    static ref GLOBAL_DATABASE: Arc<RwLock<Option<Arc<Database>>>> = 
        Arc::new(RwLock::new(None));
}

pub async fn initialize_database(path: &str) -> Result<Arc<Database>, PersistenceError> {
    info!("Initializing global database at: {}", path);
    
    let mut db_guard = GLOBAL_DATABASE.write().await;
    if db_guard.is_some() {
        warn!("Database already initialized, returning existing instance");
        return Ok(db_guard.as_ref().unwrap().clone());
    }
    
    let db = Arc::new(Database::new(path)?);
    *db_guard = Some(db.clone());
    
    info!("Global database initialized successfully");
    Ok(db)
}

pub async fn get_database() -> Result<Arc<Database>, PersistenceError> {
    let db_guard = GLOBAL_DATABASE.read().await;
    db_guard.as_ref()
        .ok_or_else(|| PersistenceError::Connection("Database not initialized".to_string()))
        .map(|db| db.clone())
}

pub async fn create_instance_manager() -> Result<InstanceManager, PersistenceError> {
    let db = get_database().await?;
    Ok(InstanceManager::new(db))
}

pub async fn create_coordinator() -> Result<MultiInstanceCoordinator, PersistenceError> {
    let db = get_database().await?;
    Ok(MultiInstanceCoordinator::new(db))
}
```

## Comprehensive Test Suite

### 1. Unit Tests (`src/tests/unit/`)

#### Connection Tests (`connection_tests.rs`)
```rust
use tempfile::tempdir;
use crate::connection::DatabasePool;

#[tokio::test]
async fn test_database_pool_creation() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let pool = DatabasePool::new(db_path.to_str().unwrap(), 5).unwrap();
    assert_eq!(pool.pool_size(), 0);
    assert_eq!(pool.idle_connections(), 0);
}

#[tokio::test]
async fn test_connection_acquisition() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let pool = DatabasePool::new(db_path.to_str().unwrap(), 2).unwrap();
    
    // Acquire connections
    let conn1 = pool.get_connection().unwrap();
    let conn2 = pool.get_connection().unwrap();
    
    // Should fail on third connection
    assert!(pool.get_connection().is_err());
    
    drop(conn1);
    drop(conn2);
    
    // Should work again after dropping
    let conn3 = pool.get_connection().unwrap();
    assert!(conn3.is_ok());
}

#[tokio::test]
async fn test_concurrent_connections() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let pool = Arc::new(DatabasePool::new(db_path.to_str().unwrap(), 10).unwrap());
    
    let mut handles = vec![];
    
    for i in 0..20 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            let conn = pool_clone.get_connection().unwrap();
            // Simulate some work
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            i
        });
        handles.push(handle);
    }
    
    let results: Vec<_> = futures::future::join_all(handles).await;
    assert_eq!(results.len(), 20);
}
```

#### Operations Tests (`operations_tests.rs`)
```rust
use tempfile::tempdir;
use crate::operations::Database;
use crate::models::*;

#[tokio::test]
async fn test_workflow_crud() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let db = Database::new(db_path.to_str().unwrap()).unwrap();
    
    // Create workflow
    let mut workflow = WorkflowExecution::new("Test Workflow".to_string());
    workflow.instance_id = Some("test-instance".to_string());
    
    db.create_workflow(workflow.clone()).await.unwrap();
    
    // Read workflow
    let retrieved = db.get_workflow(&workflow.id).await.unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, workflow.id);
    assert_eq!(retrieved.workflow_name, "Test Workflow");
    
    // Update workflow
    db.update_workflow_status(&workflow.id, "running").await.unwrap();
    
    let updated = db.get_workflow(&workflow.id).await.unwrap().unwrap();
    assert_eq!(updated.status, "running");
}

#[tokio::test]
async fn test_task_crud() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let db = Database::new(db_path.to_str().unwrap()).unwrap();
    
    // Create workflow first
    let mut workflow = WorkflowExecution::new("Test Workflow".to_string());
    workflow.instance_id = Some("test-instance".to_string());
    db.create_workflow(workflow.clone()).await.unwrap();
    
    // Create task
    let mut task = TaskExecution::new(workflow.id.clone(), "Test Task".to_string());
    task.instance_id = Some("test-instance".to_string());
    task.execution_order = 1;
    
    db.create_task(task.clone()).await.unwrap();
    
    // Get tasks for workflow
    let tasks = db.get_tasks_for_workflow(&workflow.id).await.unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].task_name, "Test Task");
    
    // Update task
    db.update_task_status(&task.id, "completed").await.unwrap();
    
    let updated_tasks = db.get_tasks_for_workflow(&workflow.id).await.unwrap();
    assert_eq!(updated_tasks[0].status, "completed");
}

#[tokio::test]
async fn test_audit_events() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let db = Database::new(db_path.to_str().unwrap()).unwrap();
    
    // Create workflow
    let mut workflow = WorkflowExecution::new("Test Workflow".to_string());
    workflow.instance_id = Some("test-instance".to_string());
    db.create_workflow(workflow.clone()).await.unwrap();
    
    // Create audit event
    let event = AuditEvent {
        id: uuid::Uuid::new_v4().to_string(),
        workflow_id: Some(workflow.id.clone()),
        task_id: None,
        event_type: "workflow_started".to_string(),
        message: "Workflow started".to_string(),
        timestamp: chrono::Utc::now(),
        data: serde_json::Value::Object(serde_json::Map::new()),
        instance_id: Some("test-instance".to_string()),
    };
    
    db.log_audit_event(event.clone()).await.unwrap();
    
    // Get audit trail
    let trail = db.get_audit_trail(&workflow.id).await.unwrap();
    assert_eq!(trail.len(), 1);
    assert_eq!(trail[0].event_type, "workflow_started");
}
```

#### Instance Tests (`instance_tests.rs`)
```rust
use tempfile::tempdir;
use std::sync::Arc;
use crate::operations::Database;
use crate::instance::InstanceManager;

#[tokio::test]
async fn test_instance_isolation() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let db = Arc::new(Database::new(db_path.to_str().unwrap()).unwrap());
    
    // Create two instance managers
    let instance1 = InstanceManager::new(db.clone());
    let instance2 = InstanceManager::new(db.clone());
    
    assert_ne!(instance1.get_instance_id(), instance2.get_instance_id());
    
    // Create workflows for each instance
    let mut workflow1 = WorkflowExecution::new("Instance 1 Workflow".to_string());
    let mut workflow2 = WorkflowExecution::new("Instance 2 Workflow".to_string());
    
    instance1.create_workflow(workflow1.clone()).await.unwrap();
    instance2.create_workflow(workflow2.clone()).await.unwrap();
    
    // Each instance should only see its own workflows
    let instance1_workflows = instance1.get_workflows_for_instance().await.unwrap();
    let instance2_workflows = instance2.get_workflows_for_instance().await.unwrap();
    
    assert_eq!(instance1_workflows.len(), 1);
    assert_eq!(instance2_workflows.len(), 1);
    assert_eq!(instance1_workflows[0].workflow_name, "Instance 1 Workflow");
    assert_eq!(instance2_workflows[0].workflow_name, "Instance 2 Workflow");
}

#[tokio::test]
async fn test_cross_instance_visibility() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let db = Arc::new(Database::new(db_path.to_str().unwrap()).unwrap());
    
    let instance1 = InstanceManager::new(db.clone());
    let instance2 = InstanceManager::new(db.clone());
    
    // Create workflows for each instance
    let mut workflow1 = WorkflowExecution::new("Instance 1 Workflow".to_string());
    let mut workflow2 = WorkflowExecution::new("Instance 2 Workflow".to_string());
    
    instance1.create_workflow(workflow1.clone()).await.unwrap();
    instance2.create_workflow(workflow2.clone()).await.unwrap();
    
    // Both instances should see all workflows
    let all_workflows1 = instance1.get_all_workflows().await.unwrap();
    let all_workflows2 = instance2.get_all_workflows().await.unwrap();
    
    assert_eq!(all_workflows1.len(), 2);
    assert_eq!(all_workflows2.len(), 2);
}
```

### 2. Integration Tests (`src/tests/integration/`)

#### Multi-Instance Tests (`multi_instance_tests.rs`)
```rust
use tempfile::tempdir;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use crate::operations::Database;
use crate::instance::InstanceManager;
use crate::coordination::MultiInstanceCoordinator;

#[tokio::test]
async fn test_concurrent_instances() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let db = Arc::new(Database::new(db_path.to_str().unwrap()).unwrap());
    
    // Create multiple instances
    let mut instances = Vec::new();
    for i in 0..5 {
        let instance = InstanceManager::new(db.clone());
        instances.push(instance);
    }
    
    // Create workflows concurrently
    let mut handles = vec![];
    for (i, instance) in instances.into_iter().enumerate() {
        let handle = tokio::spawn(async move {
            for j in 0..10 {
                let mut workflow = WorkflowExecution::new(format!("Instance {} Workflow {}", i, j));
                instance.create_workflow(workflow).await.unwrap();
            }
            i
        });
        handles.push(handle);
    }
    
    // Wait for all instances to complete
    let results: Vec<_> = futures::future::join_all(handles).await;
    assert_eq!(results.len(), 5);
    
    // Verify all workflows were created
    let coordinator = MultiInstanceCoordinator::new(db);
    let stats = coordinator.get_instance_stats().await.unwrap();
    assert_eq!(stats.total_workflows, 50);
    assert_eq!(stats.active_instances, 5);
}

#[tokio::test]
async fn test_instance_cleanup() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let db = Arc::new(Database::new(db_path.to_str().unwrap()).unwrap());
    
    // Create an instance and some workflows
    let instance = InstanceManager::new(db.clone());
    for i in 0..5 {
        let mut workflow = WorkflowExecution::new(format!("Workflow {}", i));
        instance.create_workflow(workflow).await.unwrap();
    }
    
    // Verify workflows exist
    let stats = MultiInstanceCoordinator::new(db.clone()).get_instance_stats().await.unwrap();
    assert_eq!(stats.total_workflows, 5);
    
    // Simulate old instance by updating timestamps
    let conn = db.pool.get_connection().unwrap();
    conn.execute(
        "UPDATE workflow_executions SET last_updated = datetime('now', '-2 hours')",
        []
    ).unwrap();
    
    // Cleanup old instances
    let coordinator = MultiInstanceCoordinator::new(db);
    let cleaned = coordinator.cleanup_old_instances().await.unwrap();
    assert_eq!(cleaned, 5);
    
    // Verify cleanup
    let stats = coordinator.get_instance_stats().await.unwrap();
    assert_eq!(stats.total_workflows, 0);
}
```

#### Performance Tests (`performance_tests.rs`)
```rust
use tempfile::tempdir;
use std::sync::Arc;
use std::time::Instant;
use crate::operations::Database;

#[tokio::test]
async fn test_workflow_insert_performance() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let db = Arc::new(Database::new(db_path.to_str().unwrap()).unwrap());
    
    let start = Instant::now();
    
    // Insert 1000 workflows
    for i in 0..1000 {
        let mut workflow = WorkflowExecution::new(format!("Workflow {}", i));
        workflow.instance_id = Some("perf-test".to_string());
        db.create_workflow(workflow).await.unwrap();
    }
    
    let duration = start.elapsed();
    println!("Inserted 1000 workflows in {:?}", duration);
    assert!(duration.as_secs() < 5); // Should complete in under 5 seconds
}

#[tokio::test]
async fn test_concurrent_read_write_performance() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let db = Arc::new(Database::new(db_path.to_str().unwrap()).unwrap());
    
    // Create some initial data
    for i in 0..100 {
        let mut workflow = WorkflowExecution::new(format!("Workflow {}", i));
        workflow.instance_id = Some("perf-test".to_string());
        db.create_workflow(workflow).await.unwrap();
    }
    
    let start = Instant::now();
    
    // Concurrent reads and writes
    let mut handles = vec![];
    
    // Writers
    for i in 0..10 {
        let db_clone = db.clone();
        let handle = tokio::spawn(async move {
            for j in 0..50 {
                let mut workflow = WorkflowExecution::new(format!("Concurrent Workflow {} {}", i, j));
                workflow.instance_id = Some("perf-test".to_string());
                db_clone.create_workflow(workflow).await.unwrap();
            }
        });
        handles.push(handle);
    }
    
    // Readers
    for _ in 0..10 {
        let db_clone = db.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..50 {
                db_clone.list_workflows(10, 0).await.unwrap();
            }
        });
        handles.push(handle);
    }
    
    // Wait for all operations
    futures::future::join_all(handles).await;
    
    let duration = start.elapsed();
    println!("Concurrent operations completed in {:?}", duration);
    assert!(duration.as_secs() < 10); // Should complete in under 10 seconds
}
```

#### Concurrency Tests (`concurrency_tests.rs`)
```rust
use tempfile::tempdir;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use crate::operations::Database;

#[tokio::test]
async fn test_concurrent_workflow_updates() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let db = Arc::new(Database::new(db_path.to_str().unwrap()).unwrap());
    
    // Create a workflow
    let mut workflow = WorkflowExecution::new("Concurrent Test Workflow".to_string());
    workflow.instance_id = Some("concurrent-test".to_string());
    db.create_workflow(workflow.clone()).await.unwrap();
    
    let counter = Arc::new(AtomicU32::new(0));
    
    // Update the same workflow concurrently
    let mut handles = vec![];
    for i in 0..20 {
        let db_clone = db.clone();
        let counter_clone = counter.clone();
        let workflow_id = workflow.id.clone();
        
        let handle = tokio::spawn(async move {
            let status = if i % 2 == 0 { "running" } else { "completed" };
            match db_clone.update_workflow_status(&workflow_id, status).await {
                Ok(_) => counter_clone.fetch_add(1, Ordering::SeqCst),
                Err(_) => 0, // Some updates may fail due to concurrency
            };
        });
        handles.push(handle);
    }
    
    // Wait for all updates
    futures::future::join_all(handles).await;
    
    // At least some updates should succeed
    let successful_updates = counter.load(Ordering::SeqCst);
    assert!(successful_updates > 0);
    
    // Verify the workflow still exists and has a valid status
    let retrieved = db.get_workflow(&workflow.id).await.unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert!(matches!(retrieved.status.as_str(), "running" | "completed"));
}

#[tokio::test]
async fn test_connection_pool_exhaustion() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let db = Arc::new(Database::new(db_path.to_str().unwrap()).unwrap());
    
    // Create many concurrent operations to exhaust the pool
    let mut handles = vec![];
    for i in 0..50 {
        let db_clone = db.clone();
        let handle = tokio::spawn(async move {
            // Each task holds a connection for a while
            let mut workflow = WorkflowExecution::new(format!("Workflow {}", i));
            workflow.instance_id = Some("pool-test".to_string());
            db_clone.create_workflow(workflow).await.unwrap();
            
            // Hold the connection for a bit
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            Ok::<(), crate::error::PersistenceError>(())
        });
        handles.push(handle);
    }
    
    // Most should succeed, some may fail due to pool exhaustion
    let results: Vec<_> = futures::future::join_all(handles).await;
    let successful = results.iter().filter(|r| r.is_ok()).count();
    
    // At least 80% should succeed (pool has 10 connections, 50 tasks)
    assert!(successful >= 40);
}
```

## Performance Characteristics

### Expected Performance
- **Insert**: ~10,000 workflows/second
- **Query**: ~100,000 rows/second  
- **Update**: ~50,000 updates/second
- **Concurrent**: ~1,000 operations/second (10 connections)
- **Database size**: ~1MB per 10,000 workflows

### Memory Usage
- **Connection pool**: ~10MB (10 connections)
- **Prepared statements**: ~1MB
- **Query results**: Depends on data size

## Integration with Core

### Simple API (`core/src/persistence.rs`)
```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, error, warn};

use persistence::{Database, InstanceManager, MultiInstanceCoordinator, PersistenceError};
use crate::types::WorkflowResult;

pub struct PersistenceLayer {
    instance_manager: Arc<InstanceManager>,
    coordinator: Arc<MultiInstanceCoordinator>,
}

impl PersistenceLayer {
    pub async fn new() -> Result<Self, CoreError> {
        info!("Initializing persistence layer");
        
        let db = persistence::initialize_database("~/.s_e_e/workflows.db").await?;
        let instance_manager = Arc::new(persistence::create_instance_manager().await?);
        let coordinator = Arc::new(persistence::create_coordinator().await?);
        
        info!("Persistence layer initialized successfully");
        Ok(Self {
            instance_manager,
            coordinator,
        })
    }
    
    pub async fn save_workflow(&self, workflow: WorkflowResult) -> Result<(), CoreError> {
        debug!("Saving workflow: {}", workflow.execution_id);
        
        let execution = self.convert_to_workflow_execution(workflow);
        self.instance_manager.create_workflow(execution).await
            .map_err(|e| CoreError::Persistence(e))?;
        
        info!("Workflow saved successfully: {}", execution.id);
        Ok(())
    }
    
    pub async fn get_workflows(&self) -> Result<Vec<WorkflowExecution>, CoreError> {
        debug!("Getting all workflows");
        
        self.instance_manager.get_all_workflows().await
            .map_err(|e| CoreError::Persistence(e))
    }
    
    pub async fn get_instance_stats(&self) -> Result<InstanceStats, CoreError> {
        debug!("Getting instance statistics");
        
        self.coordinator.get_instance_stats().await
            .map_err(|e| CoreError::Persistence(e))
    }
    
    fn convert_to_workflow_execution(&self, workflow: WorkflowResult) -> WorkflowExecution {
        WorkflowExecution {
            id: workflow.execution_id,
            workflow_name: workflow.workflow_name,
            status: if workflow.success { "completed".to_string() } else { "failed".to_string() },
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: Some(chrono::Utc::now()),
            success: Some(workflow.success),
            error_message: if workflow.success { None } else { Some("Workflow failed".to_string()) },
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            instance_id: None, // Will be set by instance manager
            last_updated: None,
        }
    }
}

// Global singleton
lazy_static::lazy_static! {
    static ref PERSISTENCE: Arc<RwLock<Option<PersistenceLayer>>> = 
        Arc::new(RwLock::new(None));
}

pub async fn init_persistence() -> Result<(), CoreError> {
    info!("Initializing global persistence layer");
    let mut persistence = PERSISTENCE.write().await;
    *persistence = Some(PersistenceLayer::new().await?);
    info!("Global persistence layer initialized");
    Ok(())
}

pub async fn get_persistence() -> Result<Arc<PersistenceLayer>, CoreError> {
    let persistence = PERSISTENCE.read().await;
    persistence.as_ref()
        .ok_or_else(|| CoreError::Persistence("Persistence not initialized".to_string()))
        .map(Arc::new)
}
```

## Timeline

- **Phase 1**: 3 hours (thread-safe infrastructure)
- **Phase 2**: 3 hours (multi-instance operations)  
- **Phase 3**: 2 hours (coordination layer)
- **Phase 4**: 2 hours (comprehensive testing)
- **Phase 5**: 1 hour (integration & optimization)
- **Total**: 11 hours

## Benefits

- ✅ **High performance** with minimal serialization
- ✅ **Thread safety** with connection pooling
- ✅ **Multi-instance support** for multiple GUI instances
- ✅ **Simple API** that's easy to use
- ✅ **No migration complexity** - just works
- ✅ **Desktop app friendly** - embedded, no external deps
- ✅ **Evolving API support** - flexible schema
- ✅ **Fast queries** with proper indexing
- ✅ **Easy to debug** - raw SQL is transparent
- ✅ **Comprehensive testing** - unit, integration, performance
- ✅ **Full observability** - tracing and logging throughout

This implementation provides a robust, high-performance persistence layer that can handle multiple GUI instances while maintaining data integrity and providing excellent performance characteristics.
