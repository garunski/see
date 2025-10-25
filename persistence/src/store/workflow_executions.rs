//! Workflow execution store for CRUD operations

use std::sync::Arc;
use rusqlite::params;
use chrono::Utc;
use serde_json;

use crate::db::DatabasePool;
use crate::error::PersistenceError;
use crate::models::WorkflowExecution;
use tracing::{debug, info, warn};

/// Store for workflow execution operations
pub struct WorkflowExecutionStore {
    pool: Arc<DatabasePool>,
}

impl WorkflowExecutionStore {
    /// Create a new workflow execution store
    pub fn new(pool: Arc<DatabasePool>) -> Self {
        Self { pool }
    }
    
    /// Save or update a workflow execution (upsert)
    pub async fn save(&self, execution: &WorkflowExecution) -> Result<(), PersistenceError> {
        debug!("Saving workflow execution: {}", execution.id);
        
        let conn = self.pool.get_connection()?;
        let now = Utc::now().to_rfc3339();
        let data = serde_json::to_string(execution)
            .map_err(|e| PersistenceError::Serialization(format!("Failed to serialize workflow execution: {}", e)))?;
        
        conn.execute(
            "INSERT OR REPLACE INTO workflow_executions (id, workflow_id, data, instance_id, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                execution.id,
                execution.workflow_id,
                data,
                execution.metadata.get("instance_id").and_then(|v| v.as_str()),
                now,
                now
            ],
        )?;
        
        info!("Workflow execution saved successfully: {}", execution.id);
        Ok(())
    }
    
    /// Get a workflow execution by ID
    pub async fn get(&self, id: &str) -> Result<Option<WorkflowExecution>, PersistenceError> {
        debug!("Getting workflow execution: {}", id);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare("SELECT data FROM workflow_executions WHERE id = ?1")?;
        let mut rows = stmt.query_map([id], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;
        
        if let Some(row) = rows.next() {
            let data = row?;
            let execution: WorkflowExecution = serde_json::from_str(&data)
                .map_err(|e| PersistenceError::Serialization(format!("Failed to deserialize workflow execution: {}", e)))?;
            debug!("Workflow execution found: {}", execution.id);
            Ok(Some(execution))
        } else {
            debug!("Workflow execution not found: {}", id);
            Ok(None)
        }
    }
    
    /// List all workflow executions with pagination
    pub async fn list(&self, limit: usize, offset: usize) -> Result<Vec<WorkflowExecution>, PersistenceError> {
        debug!("Listing workflow executions (limit: {}, offset: {})", limit, offset);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT data FROM workflow_executions ORDER BY created_at DESC LIMIT ?1 OFFSET ?2"
        )?;
        let rows = stmt.query_map(params![limit as i64, offset as i64], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;
        
        let mut executions = Vec::new();
        for row in rows {
            let data = row?;
            let execution: WorkflowExecution = serde_json::from_str(&data)
                .map_err(|e| PersistenceError::Serialization(format!("Failed to deserialize workflow execution: {}", e)))?;
            executions.push(execution);
        }
        
        debug!("Found {} workflow executions", executions.len());
        Ok(executions)
    }
    
    /// List workflow executions by workflow ID
    pub async fn list_by_workflow(&self, workflow_id: &str) -> Result<Vec<WorkflowExecution>, PersistenceError> {
        debug!("Listing workflow executions for workflow: {}", workflow_id);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT data FROM workflow_executions WHERE workflow_id = ?1 ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map([workflow_id], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;
        
        let mut executions = Vec::new();
        for row in rows {
            let data = row?;
            let execution: WorkflowExecution = serde_json::from_str(&data)
                .map_err(|e| PersistenceError::Serialization(format!("Failed to deserialize workflow execution: {}", e)))?;
            executions.push(execution);
        }
        
        debug!("Found {} executions for workflow {}", executions.len(), workflow_id);
        Ok(executions)
    }
    
    /// List workflow executions by status
    pub async fn list_by_status(&self, status: &str) -> Result<Vec<WorkflowExecution>, PersistenceError> {
        debug!("Listing workflow executions by status: {}", status);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT data FROM workflow_executions WHERE json_extract(data, '$.status') = ?1 ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map([status], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;
        
        let mut executions = Vec::new();
        for row in rows {
            let data = row?;
            let execution: WorkflowExecution = serde_json::from_str(&data)
                .map_err(|e| PersistenceError::Serialization(format!("Failed to deserialize workflow execution: {}", e)))?;
            executions.push(execution);
        }
        
        debug!("Found {} executions with status '{}'", executions.len(), status);
        Ok(executions)
    }
    
    /// List workflow executions by instance
    pub async fn list_by_instance(&self, instance_id: &str, limit: usize, offset: usize) -> Result<Vec<WorkflowExecution>, PersistenceError> {
        debug!("Listing workflow executions for instance: {} (limit: {}, offset: {})", instance_id, limit, offset);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT data FROM workflow_executions WHERE instance_id = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3"
        )?;
        let rows = stmt.query_map(params![instance_id, limit as i64, offset as i64], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;
        
        let mut executions = Vec::new();
        for row in rows {
            let data = row?;
            let execution: WorkflowExecution = serde_json::from_str(&data)
                .map_err(|e| PersistenceError::Serialization(format!("Failed to deserialize workflow execution: {}", e)))?;
            executions.push(execution);
        }
        
        debug!("Found {} executions for instance {}", executions.len(), instance_id);
        Ok(executions)
    }
    
    /// Delete a workflow execution
    pub async fn delete(&self, id: &str) -> Result<(), PersistenceError> {
        debug!("Deleting workflow execution: {}", id);
        
        let conn = self.pool.get_connection()?;
        let changes = conn.execute("DELETE FROM workflow_executions WHERE id = ?1", [id])?;
        
        if changes > 0 {
            info!("Workflow execution deleted successfully: {}", id);
        } else {
            warn!("Workflow execution not found for deletion: {}", id);
        }
        
        Ok(())
    }
    
    /// Count total workflow executions
    pub async fn count(&self) -> Result<usize, PersistenceError> {
        debug!("Counting total workflow executions");
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM workflow_executions")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        
        debug!("Total workflow executions: {}", count);
        Ok(count as usize)
    }
    
    /// Count workflow executions by status
    pub async fn count_by_status(&self, status: &str) -> Result<usize, PersistenceError> {
        debug!("Counting workflow executions by status: {}", status);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM workflow_executions WHERE json_extract(data, '$.status') = ?1")?;
        let count: i64 = stmt.query_row([status], |row| row.get(0))?;
        
        debug!("Workflow executions with status '{}': {}", status, count);
        Ok(count as usize)
    }
}
