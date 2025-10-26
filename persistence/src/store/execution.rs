//! Execution store operations
//!
//! This file contains ONLY execution CRUD operations following Single Responsibility Principle.

use super::Store;
use crate::logging::{
    log_db_operation_error, log_db_operation_start, log_db_operation_success, log_deserialization,
    log_serialization,
};
use crate::models::{TaskExecution, WorkflowExecution, WorkflowMetadata};
use sqlx::Row;

impl Store {
    /// Save a workflow execution
    pub async fn save_workflow_execution(
        &self,
        execution: WorkflowExecution,
    ) -> Result<(), String> {
        log_db_operation_start("save_workflow_execution", "workflow_executions");

        let json_data = serde_json::to_string(&execution).map_err(|e| {
            log_db_operation_error(
                "save_workflow_execution",
                "workflow_executions",
                &e.to_string(),
            );
            format!("Serialization error: {}", e)
        })?;

        log_serialization("WorkflowExecution", json_data.len());

        sqlx::query("INSERT OR REPLACE INTO workflow_executions (id, data) VALUES (?, ?)")
            .bind(&execution.id)
            .bind(&json_data)
            .execute(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error(
                    "save_workflow_execution",
                    "workflow_executions",
                    &e.to_string(),
                );
                format!("Database error: {}", e)
            })?;

        log_db_operation_success("save_workflow_execution", "workflow_executions", 0);
        Ok(())
    }

    /// Get a workflow execution by ID
    pub async fn get_workflow_execution(
        &self,
        id: &str,
    ) -> Result<Option<WorkflowExecution>, String> {
        log_db_operation_start("get_workflow_execution", "workflow_executions");

        let row = sqlx::query("SELECT data FROM workflow_executions WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error(
                    "get_workflow_execution",
                    "workflow_executions",
                    &e.to_string(),
                );
                format!("Database error: {}", e)
            })?;

        match row {
            Some(row) => {
                let json_data: String = row.get("data");
                log_deserialization("WorkflowExecution", json_data.len());

                let execution = serde_json::from_str(&json_data).map_err(|e| {
                    log_db_operation_error(
                        "get_workflow_execution",
                        "workflow_executions",
                        &e.to_string(),
                    );
                    format!("Deserialization error: {}", e)
                })?;

                log_db_operation_success("get_workflow_execution", "workflow_executions", 0);
                Ok(Some(execution))
            }
            None => {
                log_db_operation_success("get_workflow_execution", "workflow_executions", 0);
                Ok(None)
            }
        }
    }

    /// List all workflow executions
    pub async fn list_workflow_executions(&self) -> Result<Vec<WorkflowExecution>, String> {
        log_db_operation_start("list_workflow_executions", "workflow_executions");

        let rows = sqlx::query("SELECT data FROM workflow_executions ORDER BY id")
            .fetch_all(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error(
                    "list_workflow_executions",
                    "workflow_executions",
                    &e.to_string(),
                );
                format!("Database error: {}", e)
            })?;

        let mut executions = Vec::new();
        for row in rows {
            let json_data: String = row.get("data");
            log_deserialization("WorkflowExecution", json_data.len());

            let execution = serde_json::from_str(&json_data).map_err(|e| {
                log_db_operation_error(
                    "list_workflow_executions",
                    "workflow_executions",
                    &e.to_string(),
                );
                format!("Deserialization error: {}", e)
            })?;
            executions.push(execution);
        }

        log_db_operation_success("list_workflow_executions", "workflow_executions", 0);
        Ok(executions)
    }

    /// Delete a workflow execution
    pub async fn delete_workflow_execution(&self, id: &str) -> Result<(), String> {
        log_db_operation_start("delete_workflow_execution", "workflow_executions");

        sqlx::query("DELETE FROM workflow_executions WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error(
                    "delete_workflow_execution",
                    "workflow_executions",
                    &e.to_string(),
                );
                format!("Database error: {}", e)
            })?;

        log_db_operation_success("delete_workflow_execution", "workflow_executions", 0);
        Ok(())
    }

    /// List workflow metadata
    pub async fn list_workflow_metadata(&self) -> Result<Vec<WorkflowMetadata>, String> {
        log_db_operation_start("list_workflow_metadata", "workflow_executions");

        let executions = self.list_workflow_executions().await?;
        let metadata = executions
            .into_iter()
            .map(|exec| WorkflowMetadata {
                id: exec.id,
                name: exec.workflow_name.clone(),
                status: exec.status.to_string(),
                workflow_name: exec.workflow_name,
                start_timestamp: exec.created_at,
                task_ids: exec.tasks.iter().map(|t| t.id.clone()).collect(),
            })
            .collect();

        log_db_operation_success("list_workflow_metadata", "workflow_executions", 0);
        Ok(metadata)
    }

    /// Delete workflow metadata and associated tasks
    pub async fn delete_workflow_metadata_and_tasks(&self, id: &str) -> Result<(), String> {
        log_db_operation_start("delete_workflow_metadata_and_tasks", "workflow_executions");

        // Delete execution
        self.delete_workflow_execution(id).await?;

        // Get all tasks and filter by workflow_id
        let rows = sqlx::query("SELECT id, data FROM task_executions")
            .fetch_all(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error(
                    "delete_workflow_metadata_and_tasks",
                    "task_executions",
                    &e.to_string(),
                );
                format!("Database error: {}", e)
            })?;

        // Delete tasks that belong to this workflow
        for row in rows {
            let task_id: String = row.get("id");
            let json_data: String = row.get("data");

            let task: TaskExecution = serde_json::from_str(&json_data).map_err(|e| {
                log_db_operation_error(
                    "delete_workflow_metadata_and_tasks",
                    "task_executions",
                    &e.to_string(),
                );
                format!("Deserialization error: {}", e)
            })?;

            if task.workflow_id == id {
                sqlx::query("DELETE FROM task_executions WHERE id = ?")
                    .bind(&task_id)
                    .execute(self.pool())
                    .await
                    .map_err(|e| {
                        log_db_operation_error(
                            "delete_workflow_metadata_and_tasks",
                            "task_executions",
                            &e.to_string(),
                        );
                        format!("Database error: {}", e)
                    })?;
            }
        }

        log_db_operation_success(
            "delete_workflow_metadata_and_tasks",
            "workflow_executions",
            0,
        );
        Ok(())
    }

    /// Get workflow execution with tasks
    pub async fn get_workflow_with_tasks(&self, id: &str) -> Result<WorkflowExecution, String> {
        log_db_operation_start("get_workflow_with_tasks", "workflow_executions");

        let mut execution = self
            .get_workflow_execution(id)
            .await?
            .ok_or_else(|| format!("Workflow execution not found: {}", id))?;

        // Only load additional tasks from task_executions table if the execution has no tasks
        // This handles cases where tasks might be stored separately, but preserves tasks
        // that are already embedded in the WorkflowExecution JSON
        if execution.tasks.is_empty() {
            let additional_tasks = self.get_tasks_for_workflow(id).await?;
            execution.tasks = additional_tasks;
        }

        log_db_operation_success("get_workflow_with_tasks", "workflow_executions", 0);
        Ok(execution)
    }
}
