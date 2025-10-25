//! Workflow store for CRUD operations

use std::sync::Arc;
use rusqlite::params;
use chrono::Utc;
use serde_json;

use crate::db::DatabasePool;
use crate::error::PersistenceError;
use crate::models::Workflow;
use tracing::{debug, info, warn};

/// Store for workflow definition operations
pub struct WorkflowStore {
    pool: Arc<DatabasePool>,
}

impl WorkflowStore {
    /// Create a new workflow store
    pub fn new(pool: Arc<DatabasePool>) -> Self {
        Self { pool }
    }
    
    /// Save or update a workflow (upsert)
    pub async fn save(&self, workflow: &Workflow) -> Result<(), PersistenceError> {
        debug!("Saving workflow: {}", workflow.name);
        
        let conn = self.pool.get_connection()?;
        let now = Utc::now().to_rfc3339();
        let data = serde_json::to_string(workflow)
            .map_err(|e| PersistenceError::Serialization(format!("Failed to serialize workflow: {}", e)))?;
        
        conn.execute(
            "INSERT OR REPLACE INTO workflows (id, data, instance_id, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                workflow.id,
                data,
                workflow.metadata.get("instance_id").and_then(|v| v.as_str()),
                now,
                now
            ],
        )?;
        
        info!("Workflow saved successfully: {}", workflow.id);
        Ok(())
    }
    
    /// Get a workflow by ID
    pub async fn get(&self, id: &str) -> Result<Option<Workflow>, PersistenceError> {
        debug!("Getting workflow: {}", id);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare("SELECT data FROM workflows WHERE id = ?1")?;
        let mut rows = stmt.query_map([id], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;
        
        if let Some(row) = rows.next() {
            let data = row?;
            let workflow: Workflow = serde_json::from_str(&data)
                .map_err(|e| PersistenceError::Serialization(format!("Failed to deserialize workflow: {}", e)))?;
            debug!("Workflow found: {}", workflow.name);
            Ok(Some(workflow))
        } else {
            debug!("Workflow not found: {}", id);
            Ok(None)
        }
    }
    
    /// List all workflows with pagination
    pub async fn list(&self, limit: usize, offset: usize) -> Result<Vec<Workflow>, PersistenceError> {
        debug!("Listing workflows (limit: {}, offset: {})", limit, offset);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT data FROM workflows ORDER BY created_at DESC LIMIT ?1 OFFSET ?2"
        )?;
        let rows = stmt.query_map(params![limit as i64, offset as i64], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;
        
        let mut workflows = Vec::new();
        for row in rows {
            let data = row?;
            let workflow: Workflow = serde_json::from_str(&data)
                .map_err(|e| PersistenceError::Serialization(format!("Failed to deserialize workflow: {}", e)))?;
            workflows.push(workflow);
        }
        
        debug!("Found {} workflows", workflows.len());
        Ok(workflows)
    }
    
    /// List workflows by instance
    pub async fn list_by_instance(&self, instance_id: &str, limit: usize, offset: usize) -> Result<Vec<Workflow>, PersistenceError> {
        debug!("Listing workflows for instance: {} (limit: {}, offset: {})", instance_id, limit, offset);
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT data FROM workflows WHERE instance_id = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3"
        )?;
        let rows = stmt.query_map(params![instance_id, limit as i64, offset as i64], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;
        
        let mut workflows = Vec::new();
        for row in rows {
            let data = row?;
            let workflow: Workflow = serde_json::from_str(&data)
                .map_err(|e| PersistenceError::Serialization(format!("Failed to deserialize workflow: {}", e)))?;
            workflows.push(workflow);
        }
        
        debug!("Found {} workflows for instance {}", workflows.len(), instance_id);
        Ok(workflows)
    }
    
    /// Delete a workflow
    pub async fn delete(&self, id: &str) -> Result<(), PersistenceError> {
        debug!("Deleting workflow: {}", id);
        
        let conn = self.pool.get_connection()?;
        let changes = conn.execute("DELETE FROM workflows WHERE id = ?1", [id])?;
        
        if changes > 0 {
            info!("Workflow deleted successfully: {}", id);
        } else {
            warn!("Workflow not found for deletion: {}", id);
        }
        
        Ok(())
    }
    
    /// Search workflows by name
    pub async fn search_by_name(&self, name_pattern: &str) -> Result<Vec<Workflow>, PersistenceError> {
        debug!("Searching workflows by name pattern: {}", name_pattern);
        
        let conn = self.pool.get_connection()?;
        let pattern = format!("%{}%", name_pattern);
        let mut stmt = conn.prepare(
            "SELECT data FROM workflows WHERE json_extract(data, '$.name') LIKE ?1 ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map([&pattern], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })?;
        
        let mut workflows = Vec::new();
        for row in rows {
            let data = row?;
            let workflow: Workflow = serde_json::from_str(&data)
                .map_err(|e| PersistenceError::Serialization(format!("Failed to deserialize workflow: {}", e)))?;
            workflows.push(workflow);
        }
        
        debug!("Found {} workflows matching pattern '{}'", workflows.len(), name_pattern);
        Ok(workflows)
    }
    
    /// Count total workflows
    pub async fn count(&self) -> Result<usize, PersistenceError> {
        debug!("Counting total workflows");
        
        let conn = self.pool.get_connection()?;
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM workflows")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        
        debug!("Total workflows: {}", count);
        Ok(count as usize)
    }
}
