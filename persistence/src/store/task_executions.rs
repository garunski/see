//! Task execution store for CRUD operations

use chrono::Utc;
use rusqlite::params;
use serde_json;
use std::sync::Arc;

use crate::db::DatabasePool;
use crate::error::PersistenceError;
use crate::models::TaskExecution;
use tracing::{debug, info, warn};

/// Store for task execution operations
pub struct TaskExecutionStore {
    pool: Arc<DatabasePool>,
}

impl TaskExecutionStore {
    /// Create a new task execution store
    pub fn new(pool: Arc<DatabasePool>) -> Self {
        Self { pool }
    }

    /// Save or update a task execution (upsert)
    pub async fn save(&self, task: &TaskExecution) -> Result<(), PersistenceError> {
        debug!("Saving task execution: {}", task.id);

        let conn = self.pool.get_connection()?;
        let now = Utc::now().to_rfc3339();
        let data = serde_json::to_string(task).map_err(|e| {
            PersistenceError::Serialization(format!("Failed to serialize task execution: {}", e))
        })?;

        conn.execute(
            "INSERT OR REPLACE INTO task_executions (id, workflow_execution_id, data, instance_id, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                task.id,
                task.workflow_execution_id,
                data,
                task.metadata.get("instance_id").and_then(|v| v.as_str()),
                now,
                now
            ],
        )?;

        info!("Task execution saved successfully: {}", task.id);
        Ok(())
    }

    /// Get a task execution by ID
    pub async fn get(&self, id: &str) -> Result<Option<TaskExecution>, PersistenceError> {
        debug!("Getting task execution: {}", id);

        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare("SELECT data FROM task_executions WHERE id = ?1")?;
        let mut rows = stmt.query_map([id], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;

        if let Some(row) = rows.next() {
            let data = row?;
            let task: TaskExecution = serde_json::from_str(&data).map_err(|e| {
                PersistenceError::Serialization(format!(
                    "Failed to deserialize task execution: {}",
                    e
                ))
            })?;
            debug!("Task execution found: {}", task.id);
            Ok(Some(task))
        } else {
            debug!("Task execution not found: {}", id);
            Ok(None)
        }
    }

    /// List all task executions with pagination
    pub async fn list(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<TaskExecution>, PersistenceError> {
        debug!(
            "Listing task executions (limit: {}, offset: {})",
            limit, offset
        );

        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT data FROM task_executions ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
        )?;
        let rows = stmt.query_map(params![limit as i64, offset as i64], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;

        let mut tasks = Vec::new();
        for row in rows {
            let data = row?;
            let task: TaskExecution = serde_json::from_str(&data).map_err(|e| {
                PersistenceError::Serialization(format!(
                    "Failed to deserialize task execution: {}",
                    e
                ))
            })?;
            tasks.push(task);
        }

        debug!("Found {} task executions", tasks.len());
        Ok(tasks)
    }

    /// List task executions by workflow execution ID
    pub async fn list_by_workflow_execution(
        &self,
        workflow_execution_id: &str,
    ) -> Result<Vec<TaskExecution>, PersistenceError> {
        debug!(
            "Listing task executions for workflow execution: {}",
            workflow_execution_id
        );

        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT data FROM task_executions WHERE workflow_execution_id = ?1 ORDER BY created_at ASC"
        )?;
        let rows = stmt.query_map([workflow_execution_id], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;

        let mut tasks = Vec::new();
        for row in rows {
            let data = row?;
            let task: TaskExecution = serde_json::from_str(&data).map_err(|e| {
                PersistenceError::Serialization(format!(
                    "Failed to deserialize task execution: {}",
                    e
                ))
            })?;
            tasks.push(task);
        }

        debug!(
            "Found {} tasks for workflow execution {}",
            tasks.len(),
            workflow_execution_id
        );
        Ok(tasks)
    }

    /// List task executions by status
    pub async fn list_by_status(
        &self,
        status: &str,
    ) -> Result<Vec<TaskExecution>, PersistenceError> {
        debug!("Listing task executions by status: {}", status);

        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT data FROM task_executions WHERE json_extract(data, '$.status') = ?1 ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map([status], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;

        let mut tasks = Vec::new();
        for row in rows {
            let data = row?;
            let task: TaskExecution = serde_json::from_str(&data).map_err(|e| {
                PersistenceError::Serialization(format!(
                    "Failed to deserialize task execution: {}",
                    e
                ))
            })?;
            tasks.push(task);
        }

        debug!("Found {} tasks with status '{}'", tasks.len(), status);
        Ok(tasks)
    }

    /// List task executions by instance
    pub async fn list_by_instance(
        &self,
        instance_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<TaskExecution>, PersistenceError> {
        debug!(
            "Listing task executions for instance: {} (limit: {}, offset: {})",
            instance_id, limit, offset
        );

        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT data FROM task_executions WHERE instance_id = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3"
        )?;
        let rows = stmt.query_map(params![instance_id, limit as i64, offset as i64], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;

        let mut tasks = Vec::new();
        for row in rows {
            let data = row?;
            let task: TaskExecution = serde_json::from_str(&data).map_err(|e| {
                PersistenceError::Serialization(format!(
                    "Failed to deserialize task execution: {}",
                    e
                ))
            })?;
            tasks.push(task);
        }

        debug!("Found {} tasks for instance {}", tasks.len(), instance_id);
        Ok(tasks)
    }

    /// Delete a task execution
    pub async fn delete(&self, id: &str) -> Result<(), PersistenceError> {
        debug!("Deleting task execution: {}", id);

        let conn = self.pool.get_connection()?;
        let changes = conn.execute("DELETE FROM task_executions WHERE id = ?1", [id])?;

        if changes > 0 {
            info!("Task execution deleted successfully: {}", id);
        } else {
            warn!("Task execution not found for deletion: {}", id);
        }

        Ok(())
    }

    /// Count total task executions
    pub async fn count(&self) -> Result<usize, PersistenceError> {
        debug!("Counting total task executions");

        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM task_executions")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;

        debug!("Total task executions: {}", count);
        Ok(count as usize)
    }

    /// Count task executions by status
    pub async fn count_by_status(&self, status: &str) -> Result<usize, PersistenceError> {
        debug!("Counting task executions by status: {}", status);

        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM task_executions WHERE json_extract(data, '$.status') = ?1",
        )?;
        let count: i64 = stmt.query_row([status], |row| row.get(0))?;

        debug!("Task executions with status '{}': {}", status, count);
        Ok(count as usize)
    }

    /// Count task executions by workflow execution
    pub async fn count_by_workflow_execution(
        &self,
        workflow_execution_id: &str,
    ) -> Result<usize, PersistenceError> {
        debug!(
            "Counting task executions for workflow execution: {}",
            workflow_execution_id
        );

        let conn = self.pool.get_connection()?;
        let mut stmt =
            conn.prepare("SELECT COUNT(*) FROM task_executions WHERE workflow_execution_id = ?1")?;
        let count: i64 = stmt.query_row([workflow_execution_id], |row| row.get(0))?;

        debug!(
            "Task executions for workflow execution {}: {}",
            workflow_execution_id, count
        );
        Ok(count as usize)
    }
}
