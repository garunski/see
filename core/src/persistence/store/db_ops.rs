// Database operations wrapper with transaction helpers and retry logic

use crate::errors::CoreError;
use redb::Database;
use std::sync::Arc;
use tokio::task;
use tokio::time::{sleep, Duration};

/// Wrapper around redb Database with helper methods for common operations
#[derive(Debug, Clone)]
pub struct DatabaseOperations {
    db: Arc<Database>,
}

impl DatabaseOperations {
    /// Create a new DatabaseOperations instance
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Execute a read operation in a separate thread
    pub async fn execute_read<F, T>(&self, operation: F) -> Result<T, CoreError>
    where
        F: FnOnce(&Database) -> Result<T, CoreError> + Send + 'static,
        T: Send + 'static,
    {
        let db = Arc::clone(&self.db);
        task::spawn_blocking(move || operation(&db))
            .await
            .map_err(|e| CoreError::Dataflow(format!("task join error: {}", e)))?
    }

    /// Execute a write operation in a separate thread
    pub async fn execute_write<F, T>(&self, operation: F) -> Result<T, CoreError>
    where
        F: FnOnce(&Database) -> Result<T, CoreError> + Send + 'static,
        T: Send + 'static,
    {
        let db = Arc::clone(&self.db);
        task::spawn_blocking(move || operation(&db))
            .await
            .map_err(|e| CoreError::Dataflow(format!("task join error: {}", e)))?
    }

    /// Execute a write operation with retry logic
    pub async fn execute_write_with_retry<F, T>(&self, operation: F) -> Result<T, CoreError>
    where
        F: Fn() -> Result<T, CoreError> + Send + Sync + Clone + 'static,
        T: Send + 'static,
    {
        let mut last_error = None;

        for attempt in 0..3 {
            let operation = operation.clone();
            let result = task::spawn_blocking(operation)
                .await
                .map_err(|e| CoreError::Dataflow(format!("task join error: {}", e)))?;

            match result {
                Ok(value) => return Ok(value),
                Err(error) => {
                    last_error = Some(error);
                    if attempt == 2 {
                        break;
                    }
                    let delay_ms = 100 * (2_u64.pow(attempt));
                    sleep(Duration::from_millis(delay_ms)).await;
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// Get a reference to the underlying database
    pub fn database(&self) -> &Arc<Database> {
        &self.db
    }
}
