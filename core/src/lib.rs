pub mod engine;
pub mod errors;
pub mod execution;
pub mod json_parser;
pub mod store;
pub mod task_executor;
pub mod tracing;
pub mod types;
pub mod utils;

pub use types::*;

// Re-export tracing functionality
pub use tracing::{init_tracing, TracingGuard};

// Re-export persistence functionality
pub use persistence::{initialize_database, PersistenceError};
pub use persistence::{
    Workflow, WorkflowExecution, TaskExecution, UserPrompt, AiPrompt, Setting,
    WorkflowDefinition, AppSettings, Theme, WorkflowExecutionSummary, WorkflowMetadata,
    WorkflowJson, TaskInfo, WorkflowStatus, AuditEvent
};

pub use crate::engine::execute::{
    execute_workflow, execute_workflow_by_id, execute_workflow_from_content, pause_workflow,
    resume_task, resume_workflow,
};
pub use crate::store::{SimpleStore, Store, PersistenceStore};

use std::sync::Arc;
use crate::errors::CoreError;
// Tracing macros are available globally when tracing is enabled

// Global singleton for store access
lazy_static::lazy_static! {
    static ref GLOBAL_STORE: Arc<std::sync::RwLock<Option<Arc<dyn Store>>>> = 
        Arc::new(std::sync::RwLock::new(None));
}

/// Initialize the global store with persistence layer
pub async fn init_global_store() -> Result<(), CoreError> {
    println!("Initializing global store with persistence layer");
    
    // Initialize persistence database
    initialize_database("~/.s_e_e/workflows.db").await?;
    
    // Get instance manager
    let instance_manager = persistence::get_instance_manager().await?;
    
    // Create persistence store
    let store = Arc::new(PersistenceStore::new(instance_manager));
    
    // Store in global singleton
    let mut store_guard = GLOBAL_STORE.write()
        .map_err(|e| CoreError::MutexLock(format!("Failed to lock global store: {}", e)))?;
    *store_guard = Some(store);
    
    println!("Global store initialized successfully");
    Ok(())
}

/// Get the global store instance
pub fn get_global_store() -> Result<Arc<dyn Store>, CoreError> {
    let store_guard = GLOBAL_STORE.read()
        .map_err(|e| CoreError::MutexLock(format!("Failed to lock global store: {}", e)))?;
    
    store_guard.as_ref()
        .ok_or_else(|| {
            eprintln!("Global store not initialized");
            CoreError::Persistence(persistence::PersistenceError::Connection("Global store not initialized".to_string()))
        })
        .map(|store| store.clone())
}
