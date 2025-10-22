// Main store module - contains AuditStore trait and RedbStore

#![allow(clippy::result_large_err)]

use crate::errors::CoreError;
use crate::persistence::models::{
    AppSettings, TaskExecution, WorkflowExecution, WorkflowExecutionSummary, WorkflowMetadata,
};
use async_trait::async_trait;
use redb::{Database, Table, TableDefinition};
use std::path::PathBuf;
use std::sync::Arc;

mod db_ops;
mod keys;
mod repositories;
mod serialization;

use db_ops::DatabaseOperations;
use repositories::{SettingsRepository, TaskRepository, WorkflowRepository};

// Table definitions for consistent access
const EXECUTIONS_TABLE: &str = "executions";
const EXECUTION_IDS_TABLE: &str = "execution_ids";
const SETTINGS_TABLE: &str = "settings";
const TASKS_TABLE: &str = "tasks";

const EXECUTIONS_DEF: TableDefinition<&str, &[u8]> = TableDefinition::new(EXECUTIONS_TABLE);
const EXECUTION_IDS_DEF: TableDefinition<&str, &str> = TableDefinition::new(EXECUTION_IDS_TABLE);
const SETTINGS_DEF: TableDefinition<&str, &[u8]> = TableDefinition::new(SETTINGS_TABLE);
const TASKS_DEF: TableDefinition<&str, &[u8]> = TableDefinition::new(TASKS_TABLE);

/// Main store trait for audit operations
#[async_trait]
pub trait AuditStore: Send + Sync {
    async fn save_workflow_execution(
        &self,
        execution: &WorkflowExecution,
    ) -> Result<String, CoreError>;
    async fn get_workflow_execution(&self, id: &str) -> Result<WorkflowExecution, CoreError>;
    async fn list_workflow_executions(
        &self,
        limit: usize,
    ) -> Result<Vec<WorkflowExecutionSummary>, CoreError>;
    async fn delete_workflow_execution(&self, id: &str) -> Result<(), CoreError>;
    async fn save_workflow_metadata(&self, metadata: &WorkflowMetadata) -> Result<(), CoreError>;
    async fn save_task_execution(&self, task: &TaskExecution) -> Result<(), CoreError>;
    async fn get_workflow_with_tasks(
        &self,
        execution_id: &str,
    ) -> Result<WorkflowExecution, CoreError>;
    async fn list_workflow_metadata(
        &self,
        limit: usize,
    ) -> Result<Vec<WorkflowMetadata>, CoreError>;
    async fn delete_workflow_metadata_and_tasks(&self, execution_id: &str)
        -> Result<(), CoreError>;
    async fn load_settings(&self) -> Result<Option<AppSettings>, CoreError>;
    async fn save_settings(&self, settings: &AppSettings) -> Result<(), CoreError>;
}

/// RedbStore implementation using repository pattern
#[derive(Debug)]
pub struct RedbStore {
    workflow_repo: WorkflowRepository,
    task_repo: TaskRepository,
    settings_repo: SettingsRepository,
}

impl RedbStore {
    /// Create a new RedbStore with the given database path
    pub fn new(db_path: PathBuf) -> Result<Self, CoreError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Database::create(db_path)?;
        let write_txn = db.begin_write()?;
        {
            let _executions_table: Table<&str, &[u8]> = write_txn.open_table(EXECUTIONS_DEF)?;
            let _execution_ids_table: Table<&str, &str> =
                write_txn.open_table(EXECUTION_IDS_DEF)?;
            let _settings_table: Table<&str, &[u8]> = write_txn.open_table(SETTINGS_DEF)?;
            let _tasks_table: Table<&str, &[u8]> = write_txn.open_table(TASKS_DEF)?;
        }
        write_txn.commit()?;

        let db_ops = DatabaseOperations::new(Arc::new(db));
        let workflow_repo = WorkflowRepository::new(db_ops.clone());
        let task_repo = TaskRepository::new(db_ops.clone());
        let settings_repo = SettingsRepository::new(db_ops);

        Ok(Self {
            workflow_repo,
            task_repo,
            settings_repo,
        })
    }

    /// Get the default database path
    pub fn default_path() -> Result<PathBuf, CoreError> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "home dir"))?;
        Ok(home_dir.join(".see").join("audit.redb"))
    }

    /// Create a new RedbStore with the default path
    pub fn new_default() -> Result<Self, CoreError> {
        Self::new(Self::default_path()?)
    }
}

#[async_trait]
impl AuditStore for RedbStore {
    async fn save_workflow_execution(
        &self,
        execution: &WorkflowExecution,
    ) -> Result<String, CoreError> {
        self.workflow_repo.save_execution(execution).await
    }

    async fn get_workflow_execution(&self, id: &str) -> Result<WorkflowExecution, CoreError> {
        self.workflow_repo.get_execution(id).await
    }

    async fn list_workflow_executions(
        &self,
        limit: usize,
    ) -> Result<Vec<WorkflowExecutionSummary>, CoreError> {
        self.workflow_repo.list_executions(limit).await
    }

    async fn delete_workflow_execution(&self, id: &str) -> Result<(), CoreError> {
        self.workflow_repo.delete_execution(id).await
    }

    async fn save_workflow_metadata(&self, metadata: &WorkflowMetadata) -> Result<(), CoreError> {
        self.workflow_repo.save_metadata(metadata).await
    }

    async fn save_task_execution(&self, task: &TaskExecution) -> Result<(), CoreError> {
        self.task_repo.save_task(task).await
    }

    async fn get_workflow_with_tasks(
        &self,
        execution_id: &str,
    ) -> Result<WorkflowExecution, CoreError> {
        self.workflow_repo.get_with_tasks(execution_id).await
    }

    async fn list_workflow_metadata(
        &self,
        limit: usize,
    ) -> Result<Vec<WorkflowMetadata>, CoreError> {
        self.workflow_repo.list_metadata(limit).await
    }

    async fn delete_workflow_metadata_and_tasks(
        &self,
        execution_id: &str,
    ) -> Result<(), CoreError> {
        self.workflow_repo
            .delete_metadata_and_tasks(execution_id)
            .await
    }

    async fn load_settings(&self) -> Result<Option<AppSettings>, CoreError> {
        self.settings_repo.load().await
    }

    async fn save_settings(&self, settings: &AppSettings) -> Result<(), CoreError> {
        self.settings_repo.save(settings).await
    }
}
