//! SQLite Document-Based Persistence Layer
//!
//! High-performance, thread-safe SQLite persistence layer for S_E_E workflow engine.
//! Uses SQLite as a document database with JSON storage for maximum flexibility.

pub mod error;
pub mod models;
pub mod types;
pub mod db;
pub mod store;
pub mod instance;

// Re-export public types
pub use error::PersistenceError;
pub use models::*;
pub use types::*;
pub use db::{DatabasePool, initialize_schema};
pub use store::*;
pub use instance::{InstanceManager, MultiInstanceCoordinator, InstanceStats};

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, warn};

// Global singleton for thread-safe access
lazy_static::lazy_static! {
    static ref GLOBAL_DATABASE: Arc<RwLock<Option<Arc<DatabasePool>>>> = 
        Arc::new(RwLock::new(None));
    static ref GLOBAL_INSTANCE: Arc<RwLock<Option<Arc<InstanceManager>>>> = 
        Arc::new(RwLock::new(None));
}

/// Initialize the global database at the specified path.
/// 
/// This function:
/// - Creates the ~/.s_e_e/ directory if it doesn't exist
/// - Initializes the database schema with 5 document tables
/// - Sets up connection pooling with WAL mode
/// - Stores the database in a global singleton
/// 
/// # Arguments
/// * `path` - Path to the SQLite database file
/// 
/// # Errors
/// Returns `PersistenceError` if initialization fails
/// 
/// # Example
/// ```rust
/// use persistence::initialize_database;
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     initialize_database("~/.s_e_e/workflows.db").await?;
///     Ok(())
/// }
/// ```
pub async fn initialize_database(path: &str) -> Result<(), PersistenceError> {
    info!("Initializing global database at: {}", path);
    
    // Expand tilde to home directory
    let expanded_path = if path.starts_with("~/") {
        let home = dirs::home_dir()
            .ok_or_else(|| PersistenceError::Connection("Could not find home directory".to_string()))?;
        home.join(&path[2..])
    } else {
        std::path::PathBuf::from(path)
    };
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = expanded_path.parent() {
        debug!("Creating directory: {:?}", parent);
        std::fs::create_dir_all(parent)
            .map_err(|e| PersistenceError::Connection(format!("Failed to create directory: {}", e)))?;
    }
    
    let path_str = expanded_path.to_str()
        .ok_or_else(|| PersistenceError::Connection("Invalid database path".to_string()))?;
    
    // Initialize database pool
    let pool = DatabasePool::new(path_str, 10)?; // 10 max connections
    info!("Database pool created with 10 max connections");
    
    // Initialize schema
    let conn = pool.get_connection()?;
    initialize_schema(&conn)?;
    drop(conn);
    
    // Store in global singleton
    let mut db_guard = GLOBAL_DATABASE.write().await;
    *db_guard = Some(Arc::new(pool));
    
    info!("Global database initialized successfully");
    Ok(())
}

/// Get the global database instance.
/// 
/// # Errors
/// Returns `PersistenceError` if database hasn't been initialized
pub async fn get_database() -> Result<Arc<DatabasePool>, PersistenceError> {
    let db_guard = GLOBAL_DATABASE.read().await;
    db_guard.as_ref()
        .ok_or_else(|| {
            warn!("Database not initialized");
            PersistenceError::Connection("Database not initialized".to_string())
        })
        .map(|db| db.clone())
}

/// Get or create the global instance manager.
/// 
/// Creates a new instance manager if one doesn't exist.
/// 
/// # Errors
/// Returns `PersistenceError` if database hasn't been initialized
pub async fn get_instance_manager() -> Result<Arc<InstanceManager>, PersistenceError> {
    let mut instance_guard = GLOBAL_INSTANCE.write().await;
    
    if let Some(instance) = instance_guard.as_ref() {
        debug!("Returning existing instance manager");
        return Ok(instance.clone());
    }
    
    let db = get_database().await?;
    let instance = Arc::new(InstanceManager::new(db));
    info!("Created new instance manager");
    
    *instance_guard = Some(instance.clone());
    Ok(instance)
}

/// Create a new multi-instance coordinator.
/// 
/// # Errors
/// Returns `PersistenceError` if database hasn't been initialized
pub async fn create_coordinator() -> Result<MultiInstanceCoordinator, PersistenceError> {
    debug!("Creating coordinator");
    let db = get_database().await?;
    Ok(MultiInstanceCoordinator::new(db))
}

/// Create a workflow store.
/// 
/// # Errors
/// Returns `PersistenceError` if database hasn't been initialized
pub async fn create_workflow_store() -> Result<WorkflowStore, PersistenceError> {
    debug!("Creating workflow store");
    let db = get_database().await?;
    Ok(WorkflowStore::new(db))
}

/// Create a workflow execution store.
/// 
/// # Errors
/// Returns `PersistenceError` if database hasn't been initialized
pub async fn create_workflow_execution_store() -> Result<WorkflowExecutionStore, PersistenceError> {
    debug!("Creating workflow execution store");
    let db = get_database().await?;
    Ok(WorkflowExecutionStore::new(db))
}

/// Create a task execution store.
/// 
/// # Errors
/// Returns `PersistenceError` if database hasn't been initialized
pub async fn create_task_execution_store() -> Result<TaskExecutionStore, PersistenceError> {
    debug!("Creating task execution store");
    let db = get_database().await?;
    Ok(TaskExecutionStore::new(db))
}

/// Create a user prompt store.
/// 
/// # Errors
/// Returns `PersistenceError` if database hasn't been initialized
pub async fn create_user_prompt_store() -> Result<UserPromptStore, PersistenceError> {
    debug!("Creating user prompt store");
    let db = get_database().await?;
    Ok(UserPromptStore::new(db))
}

/// Create an AI prompt store.
/// 
/// # Errors
/// Returns `PersistenceError` if database hasn't been initialized
pub async fn create_ai_prompt_store() -> Result<AiPromptStore, PersistenceError> {
    debug!("Creating AI prompt store");
    let db = get_database().await?;
    Ok(AiPromptStore::new(db))
}

/// Create a settings store.
/// 
/// # Errors
/// Returns `PersistenceError` if database hasn't been initialized
pub async fn create_settings_store() -> Result<SettingsStore, PersistenceError> {
    debug!("Creating settings store");
    let db = get_database().await?;
    Ok(SettingsStore::new(db))
}