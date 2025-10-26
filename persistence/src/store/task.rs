//! Task store operations
//!
//! This file contains ONLY task operations following Single Responsibility Principle.

use super::Store;
use crate::logging::{
    log_db_operation_error, log_db_operation_start, log_db_operation_success, log_deserialization,
    log_serialization,
};
use crate::models::TaskExecution;
use sqlx::Row;

impl Store {
    /// Save a task execution
    pub async fn save_task_execution(&self, task: TaskExecution) -> Result<(), String> {
        log_db_operation_start("save_task_execution", "task_executions");

        let json_data = serde_json::to_string(&task).map_err(|e| {
            log_db_operation_error("save_task_execution", "task_executions", &e.to_string());
            format!("Serialization error: {}", e)
        })?;

        log_serialization("TaskExecution", json_data.len());

        sqlx::query("INSERT OR REPLACE INTO task_executions (id, data) VALUES (?, ?)")
            .bind(&task.id)
            .bind(&json_data)
            .execute(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error("save_task_execution", "task_executions", &e.to_string());
                format!("Database error: {}", e)
            })?;

        log_db_operation_success("save_task_execution", "task_executions", 0);
        Ok(())
    }

    /// Get tasks for a workflow
    pub async fn get_tasks_for_workflow(
        &self,
        workflow_id: &str,
    ) -> Result<Vec<TaskExecution>, String> {
        log_db_operation_start("get_tasks_for_workflow", "task_executions");

        let rows = sqlx::query("SELECT data FROM task_executions")
            .fetch_all(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error("get_tasks_for_workflow", "task_executions", &e.to_string());
                format!("Database error: {}", e)
            })?;

        let mut tasks = Vec::new();
        for row in rows {
            let json_data: String = row.get("data");
            log_deserialization("TaskExecution", json_data.len());

            let task: TaskExecution = serde_json::from_str(&json_data).map_err(|e| {
                log_db_operation_error("get_tasks_for_workflow", "task_executions", &e.to_string());
                format!("Deserialization error: {}", e)
            })?;

            if task.workflow_id == workflow_id {
                tasks.push(task);
            }
        }

        log_db_operation_success("get_tasks_for_workflow", "task_executions", 0);
        Ok(tasks)
    }
}
