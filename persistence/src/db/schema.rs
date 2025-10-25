//! Database schema management

use rusqlite::Connection;
use crate::error::PersistenceError;
use tracing::{info, debug};

/// Initialize the database schema with 5 document-based tables
pub fn initialize_schema(conn: &Connection) -> Result<(), PersistenceError> {
    info!("Initializing database schema with document-based tables");
    
    // Enable WAL mode for better concurrency
    conn.execute("PRAGMA journal_mode=WAL", [])?;
    
    // Set performance pragmas
    conn.execute("PRAGMA synchronous=NORMAL", [])?;
    conn.execute("PRAGMA cache_size=10000", [])?;
    conn.execute("PRAGMA temp_store=MEMORY", [])?;
    
    // Create tables
    create_workflows_table(conn)?;
    create_workflow_executions_table(conn)?;
    create_task_executions_table(conn)?;
    create_user_prompts_table(conn)?;
    create_ai_prompts_table(conn)?;
    create_settings_table(conn)?;
    
    // Create indexes
    create_indexes(conn)?;
    
    info!("Database schema initialized successfully");
    Ok(())
}

/// Create the workflows table (workflow definitions)
fn create_workflows_table(conn: &Connection) -> Result<(), PersistenceError> {
    debug!("Creating workflows table");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS workflows (
            id TEXT PRIMARY KEY,
            data TEXT NOT NULL,
            instance_id TEXT,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    Ok(())
}

/// Create the workflow_executions table (runtime executions)
fn create_workflow_executions_table(conn: &Connection) -> Result<(), PersistenceError> {
    debug!("Creating workflow_executions table");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS workflow_executions (
            id TEXT PRIMARY KEY,
            workflow_id TEXT NOT NULL,
            data TEXT NOT NULL,
            instance_id TEXT,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    Ok(())
}

/// Create the task_executions table (individual task executions)
fn create_task_executions_table(conn: &Connection) -> Result<(), PersistenceError> {
    debug!("Creating task_executions table");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS task_executions (
            id TEXT PRIMARY KEY,
            workflow_execution_id TEXT NOT NULL,
            data TEXT NOT NULL,
            instance_id TEXT,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    Ok(())
}

/// Create the user_prompts table (user-defined prompts)
fn create_user_prompts_table(conn: &Connection) -> Result<(), PersistenceError> {
    debug!("Creating user_prompts table");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS user_prompts (
            id TEXT PRIMARY KEY,
            data TEXT NOT NULL,
            instance_id TEXT,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    Ok(())
}

/// Create the ai_prompts table (AI model prompts)
fn create_ai_prompts_table(conn: &Connection) -> Result<(), PersistenceError> {
    debug!("Creating ai_prompts table");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ai_prompts (
            id TEXT PRIMARY KEY,
            data TEXT NOT NULL,
            instance_id TEXT,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    Ok(())
}

/// Create the settings table (app configuration)
fn create_settings_table(conn: &Connection) -> Result<(), PersistenceError> {
    debug!("Creating settings table");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            data TEXT NOT NULL,
            instance_id TEXT,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    Ok(())
}

/// Create all indexes for performance
fn create_indexes(conn: &Connection) -> Result<(), PersistenceError> {
    debug!("Creating database indexes");
    
    // Workflows indexes
    conn.execute("CREATE INDEX IF NOT EXISTS idx_workflows_instance ON workflows(instance_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_workflows_name ON workflows(json_extract(data, '$.name'))", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_workflows_created ON workflows(created_at)", [])?;
    
    // Workflow executions indexes
    conn.execute("CREATE INDEX IF NOT EXISTS idx_wf_exec_workflow ON workflow_executions(workflow_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_wf_exec_instance ON workflow_executions(instance_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_wf_exec_status ON workflow_executions(json_extract(data, '$.status'))", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_wf_exec_created ON workflow_executions(created_at)", [])?;
    
    // Task executions indexes
    conn.execute("CREATE INDEX IF NOT EXISTS idx_task_exec_workflow ON task_executions(workflow_execution_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_task_exec_instance ON task_executions(instance_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_task_exec_status ON task_executions(json_extract(data, '$.status'))", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_task_exec_created ON task_executions(created_at)", [])?;
    
    // User prompts indexes
    conn.execute("CREATE INDEX IF NOT EXISTS idx_user_prompts_instance ON user_prompts(instance_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_user_prompts_name ON user_prompts(json_extract(data, '$.name'))", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_user_prompts_created ON user_prompts(created_at)", [])?;
    
    // AI prompts indexes
    conn.execute("CREATE INDEX IF NOT EXISTS idx_ai_prompts_instance ON ai_prompts(instance_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_ai_prompts_name ON ai_prompts(json_extract(data, '$.name'))", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_ai_prompts_model ON ai_prompts(json_extract(data, '$.model'))", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_ai_prompts_created ON ai_prompts(created_at)", [])?;
    
    // Settings indexes
    conn.execute("CREATE INDEX IF NOT EXISTS idx_settings_instance ON settings(instance_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_settings_created ON settings(created_at)", [])?;
    
    debug!("All indexes created successfully");
    Ok(())
}

/// Get the current schema version
pub fn get_schema_version(_conn: &Connection) -> Result<i32, PersistenceError> {
    // For now, we don't use schema versioning since we're using document-based storage
    // This could be extended in the future if needed
    Ok(1)
}

/// Upgrade schema (placeholder for future migrations)
pub fn upgrade_schema(_conn: &Connection, _from_version: i32, _to_version: i32) -> Result<(), PersistenceError> {
    // Document-based storage doesn't need schema migrations
    // New fields can be added to JSON documents without schema changes
    debug!("Schema upgrade not needed for document-based storage");
    Ok(())
}