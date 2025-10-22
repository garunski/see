#![allow(clippy::result_large_err)]
use crate::errors::CoreError;
use crate::persistence::models::{WorkflowExecution, WorkflowExecutionSummary};
use crate::types::TaskInfo;
use async_trait::async_trait;
use bincode;
use redb::{Database, ReadOnlyTable, ReadableTable, Table};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task;
use tokio::time::{sleep, Duration};
use tracing::{debug, instrument, trace};

const EXECUTIONS_TABLE: &str = "executions";
const EXECUTION_IDS_TABLE: &str = "execution_ids";
const SETTINGS_TABLE: &str = "settings";
const TASKS_TABLE: &str = "tasks";

// Table definitions for consistent access
const EXECUTIONS_DEF: redb::TableDefinition<&str, &[u8]> =
    redb::TableDefinition::new(EXECUTIONS_TABLE);
const EXECUTION_IDS_DEF: redb::TableDefinition<&str, &str> =
    redb::TableDefinition::new(EXECUTION_IDS_TABLE);
const SETTINGS_DEF: redb::TableDefinition<&str, &[u8]> = redb::TableDefinition::new(SETTINGS_TABLE);
const TASKS_DEF: redb::TableDefinition<&str, &[u8]> = redb::TableDefinition::new(TASKS_TABLE);

#[derive(Debug)]
pub struct RedbStore {
    db: Arc<Database>,
}

impl RedbStore {
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
        Ok(Self { db: Arc::new(db) })
    }

    pub fn default_path() -> Result<PathBuf, CoreError> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "home dir"))?;
        Ok(home_dir.join(".see").join("audit.redb"))
    }

    pub fn new_default() -> Result<Self, CoreError> {
        Self::new(Self::default_path()?)
    }

    // Helper methods for common database operations
    async fn execute_read<F, T>(&self, operation: F) -> Result<T, CoreError>
    where
        F: FnOnce(&Database) -> Result<T, CoreError> + Send + 'static,
        T: Send + 'static,
    {
        let db = Arc::clone(&self.db);
        task::spawn_blocking(move || operation(&db))
            .await
            .map_err(|e| CoreError::Dataflow(format!("task join error: {}", e)))?
    }

    async fn execute_write<F, T>(&self, operation: F) -> Result<T, CoreError>
    where
        F: FnOnce(&Database) -> Result<T, CoreError> + Send + 'static,
        T: Send + 'static,
    {
        let db = Arc::clone(&self.db);
        task::spawn_blocking(move || operation(&db))
            .await
            .map_err(|e| CoreError::Dataflow(format!("task join error: {}", e)))?
    }

    async fn retry_operation<F, T>(&self, operation: F) -> Result<T, CoreError>
    where
        F: Fn() -> Result<T, CoreError> + Send + Sync + Clone + 'static,
        T: Send + 'static,
    {
        let mut last_error = None;

        for attempt in 0..3 {
            let operation = operation.clone();
            let result = task::spawn_blocking(move || operation())
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

    fn serialize<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, CoreError> {
        bincode::serialize(value).map_err(|e| CoreError::Dataflow(e.to_string()))
    }

    fn deserialize<T: serde::de::DeserializeOwned>(data: &[u8]) -> Result<T, CoreError> {
        bincode::deserialize(data).map_err(|e| CoreError::Dataflow(e.to_string()))
    }

    // Key generation helpers
    fn execution_timestamp_key(timestamp: &str, id: &str) -> String {
        format!("{}:{}", timestamp, id)
    }

    fn workflow_metadata_key(id: &str) -> String {
        format!("workflow:{}", id)
    }

    fn task_key(execution_id: &str, task_id: &str) -> String {
        format!("task:{}:{}", execution_id, task_id)
    }

    fn task_prefix(execution_id: &str) -> String {
        format!("task:{}:", execution_id)
    }

    pub async fn load_settings(
        &self,
    ) -> Result<Option<crate::persistence::models::AppSettings>, CoreError> {
        self.execute_read(|db| {
            let read_txn = db.begin_read()?;
            let settings_table: ReadOnlyTable<&str, &[u8]> = read_txn.open_table(SETTINGS_DEF)?;
            if let Some(serialized) = settings_table.get("app_settings")? {
                let settings: crate::persistence::models::AppSettings =
                    Self::deserialize(serialized.value())?;
                Ok(Some(settings))
            } else {
                Ok(None)
            }
        })
        .await
    }

    pub async fn save_settings(
        &self,
        settings: &crate::persistence::models::AppSettings,
    ) -> Result<(), CoreError> {
        let settings = settings.clone();
        let db = Arc::clone(&self.db);
        self.retry_operation(move || {
            task::block_in_place(|| {
                let write_txn = db.begin_write()?;
                {
                    let mut settings_table: Table<&str, &[u8]> =
                        write_txn.open_table(SETTINGS_DEF)?;
                    let serialized = Self::serialize(&settings)?;
                    settings_table.insert("app_settings", serialized.as_slice())?;
                }
                write_txn.commit()?;
                Ok(())
            })
        })
        .await
    }
}

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
    async fn save_workflow_metadata(
        &self,
        metadata: &crate::persistence::models::WorkflowMetadata,
    ) -> Result<(), CoreError>;
    async fn save_task_execution(
        &self,
        task: &crate::persistence::models::TaskExecution,
    ) -> Result<(), CoreError>;
    async fn get_workflow_with_tasks(
        &self,
        execution_id: &str,
    ) -> Result<WorkflowExecution, CoreError>;
    async fn list_workflow_metadata(
        &self,
        limit: usize,
    ) -> Result<Vec<crate::persistence::models::WorkflowMetadata>, CoreError>;
    async fn delete_workflow_metadata_and_tasks(&self, execution_id: &str)
        -> Result<(), CoreError>;
    async fn load_settings(
        &self,
    ) -> Result<Option<crate::persistence::models::AppSettings>, CoreError>;
    async fn save_settings(
        &self,
        settings: &crate::persistence::models::AppSettings,
    ) -> Result<(), CoreError>;
}

#[async_trait]
impl AuditStore for RedbStore {
    #[instrument(skip(self, execution), fields(execution_id = %execution.id))]
    async fn save_workflow_execution(
        &self,
        execution: &WorkflowExecution,
    ) -> Result<String, CoreError> {
        let execution = execution.clone();
        let id = execution.id.clone();
        let db = Arc::clone(&self.db);

        trace!("Entering save_workflow_execution");
        self.retry_operation(move || {
            let execution = execution.clone();
            let id = id.clone();
            task::block_in_place(|| {
                trace!("Beginning write transaction");
                let write_txn = db.begin_write()?;
                {
                    trace!("Opening tables");
                    let mut executions_table: Table<&str, &[u8]> =
                        write_txn.open_table(EXECUTIONS_DEF)?;
                    let mut execution_ids_table: Table<&str, &str> =
                        write_txn.open_table(EXECUTION_IDS_DEF)?;
                    let serialized = Self::serialize(&execution)?;
                    debug!(serialized_size = serialized.len(), "Serialized execution");
                    executions_table.insert(id.as_str(), serialized.as_slice())?;
                    trace!("Inserted into executions table");
                    let timestamp_key = Self::execution_timestamp_key(&execution.timestamp, &id);
                    execution_ids_table.insert(timestamp_key.as_str(), id.as_str())?;
                    trace!("Inserted into execution_ids table");
                }
                trace!("Committing transaction");
                write_txn.commit()?;
                trace!("Write transaction committed successfully");
                Ok(id)
            })
        })
        .await
    }

    async fn get_workflow_execution(&self, id: &str) -> Result<WorkflowExecution, CoreError> {
        let id = id.to_string();
        self.execute_read(move |db| {
            let read_txn = db.begin_read()?;
            let executions_table: ReadOnlyTable<&str, &[u8]> =
                read_txn.open_table(EXECUTIONS_DEF)?;
            if let Some(serialized) = executions_table.get(id.as_str())? {
                let execution: WorkflowExecution = Self::deserialize(serialized.value())?;
                Ok(execution)
            } else {
                Err(CoreError::Dataflow(format!(
                    "Workflow execution with id '{}' not found",
                    id
                )))
            }
        })
        .await
    }

    async fn list_workflow_executions(
        &self,
        limit: usize,
    ) -> Result<Vec<WorkflowExecutionSummary>, CoreError> {
        self.execute_read(move |db| {
            let read_txn = db.begin_read()?;
            let execution_ids_table: ReadOnlyTable<&str, &str> =
                read_txn.open_table(EXECUTION_IDS_DEF)?;
            let executions_table: ReadOnlyTable<&str, &[u8]> =
                read_txn.open_table(EXECUTIONS_DEF)?;
            let mut summaries = Vec::new();
            let mut count = 0;
            for item in execution_ids_table.iter()?.rev() {
                if count >= limit {
                    break;
                }
                let (_, id_value) = item?;
                let id: &str = id_value.value();
                if let Some(serialized) = executions_table.get(id)? {
                    let execution: WorkflowExecution = Self::deserialize(serialized.value())?;
                    summaries.push(WorkflowExecutionSummary {
                        id: execution.id,
                        workflow_name: execution.workflow_name,
                        timestamp: execution.timestamp,
                        success: execution.success,
                        task_count: execution.tasks.len(),
                    });
                    count += 1;
                }
            }
            Ok(summaries)
        })
        .await
    }

    async fn delete_workflow_execution(&self, id: &str) -> Result<(), CoreError> {
        let id = id.to_string();
        self.execute_write(move |db| {
            let write_txn = db.begin_write()?;
            {
                let mut executions_table: Table<&str, &[u8]> =
                    write_txn.open_table(EXECUTIONS_DEF)?;
                let mut execution_ids_table: Table<&str, &str> =
                    write_txn.open_table(EXECUTION_IDS_DEF)?;
                let mut tasks_table: Table<&str, &[u8]> = write_txn.open_table(TASKS_DEF)?;

                let timestamp_key = {
                    let execution_data = executions_table.get(id.as_str())?;
                    if let Some(serialized) = execution_data {
                        let execution: WorkflowExecution = Self::deserialize(serialized.value())?;
                        Self::execution_timestamp_key(&execution.timestamp, &id)
                    } else {
                        return Err(CoreError::Dataflow(format!(
                            "Workflow execution with id '{}' not found",
                            id
                        )));
                    }
                };

                executions_table.remove(id.as_str())?;
                execution_ids_table.remove(timestamp_key.as_str())?;

                let metadata_key = Self::workflow_metadata_key(&id);
                executions_table.remove(metadata_key.as_str())?;

                let task_prefix = Self::task_prefix(&id);
                let mut keys_to_delete = Vec::new();

                for item in tasks_table.iter()? {
                    let (key, _) = item?;
                    if key.value().starts_with(&task_prefix) {
                        keys_to_delete.push(key.value().to_string());
                    }
                }

                for key in keys_to_delete {
                    tasks_table.remove(key.as_str())?;
                }
            }
            write_txn.commit()?;
            Ok(())
        })
        .await
    }

    #[instrument(skip(self, metadata), fields(metadata_id = %metadata.id, status = ?metadata.status))]
    async fn save_workflow_metadata(
        &self,
        metadata: &crate::persistence::models::WorkflowMetadata,
    ) -> Result<(), CoreError> {
        let metadata = metadata.clone();
        let key = Self::workflow_metadata_key(&metadata.id);

        trace!("Saving workflow metadata");
        self.execute_write(move |db| {
            let write_txn = db.begin_write()?;
            {
                let mut workflows_table: Table<&str, &[u8]> =
                    write_txn.open_table(EXECUTIONS_DEF)?;
                let serialized = Self::serialize(&metadata)?;
                workflows_table.insert(key.as_str(), serialized.as_slice())?;
            }
            write_txn.commit()?;
            Ok(())
        })
        .await
    }

    #[instrument(skip(self, task), fields(execution_id = %task.execution_id, task_id = %task.task_id))]
    async fn save_task_execution(
        &self,
        task: &crate::persistence::models::TaskExecution,
    ) -> Result<(), CoreError> {
        let task = task.clone();
        let key = Self::task_key(&task.execution_id, &task.task_id);

        trace!("Saving task execution");
        self.execute_write(move |db| {
            let write_txn = db.begin_write()?;
            {
                let mut tasks_table: Table<&str, &[u8]> = write_txn.open_table(TASKS_DEF)?;
                let serialized = Self::serialize(&task)?;
                tasks_table.insert(key.as_str(), serialized.as_slice())?;
            }
            write_txn.commit()?;
            Ok(())
        })
        .await
    }

    async fn get_workflow_with_tasks(
        &self,
        execution_id: &str,
    ) -> Result<WorkflowExecution, CoreError> {
        let execution_id = execution_id.to_string();
        self.execute_read(move |db| {
            let read_txn = db.begin_read()?;

            let workflows_table: ReadOnlyTable<&str, &[u8]> =
                read_txn.open_table(EXECUTIONS_DEF)?;
            let workflow_key = Self::workflow_metadata_key(&execution_id);
            let metadata: crate::persistence::models::WorkflowMetadata =
                if let Some(serialized) = workflows_table.get(workflow_key.as_str())? {
                    Self::deserialize(serialized.value())?
                } else {
                    return Err(CoreError::Dataflow(format!(
                        "Workflow {} not found",
                        execution_id
                    )));
                };

            let tasks_table: ReadOnlyTable<&str, &[u8]> = read_txn.open_table(TASKS_DEF)?;
            let task_prefix = Self::task_prefix(&execution_id);

            let mut tasks = Vec::new();
            let mut per_task_logs = std::collections::HashMap::new();

            for item in tasks_table.iter()? {
                let (key, value) = item?;
                let key_str = key.value();
                if key_str.starts_with(&task_prefix) {
                    let task_exec: crate::persistence::models::TaskExecution =
                        Self::deserialize(value.value())?;

                    tasks.push(TaskInfo {
                        id: task_exec.task_id.clone(),
                        name: task_exec.task_name.clone(),
                        status: task_exec.status,
                    });

                    per_task_logs.insert(task_exec.task_id.clone(), task_exec.logs.clone());
                }
            }

            let ordered_tasks = if !metadata.task_ids.is_empty() {
                let mut ordered_tasks = Vec::new();
                for task_id in &metadata.task_ids {
                    if let Some(task) = tasks.iter().find(|t| &t.id == task_id) {
                        ordered_tasks.push(task.clone());
                    }
                }
                ordered_tasks
            } else {
                tasks
            };

            Ok(WorkflowExecution {
                id: metadata.id,
                workflow_name: metadata.workflow_name,
                timestamp: metadata.start_timestamp,
                success: metadata.status == crate::persistence::models::WorkflowStatus::Complete,
                tasks: ordered_tasks,
                audit_trail: vec![],
                per_task_logs,
                errors: if metadata.status == crate::persistence::models::WorkflowStatus::Failed {
                    vec!["Workflow failed".to_string()]
                } else {
                    vec![]
                },
            })
        })
        .await
    }

    async fn list_workflow_metadata(
        &self,
        limit: usize,
    ) -> Result<Vec<crate::persistence::models::WorkflowMetadata>, CoreError> {
        self.execute_read(move |db| {
            let read_txn = db.begin_read()?;
            let workflows_table: ReadOnlyTable<&str, &[u8]> =
                read_txn.open_table(EXECUTIONS_DEF)?;

            let mut metadata_list = Vec::new();
            let mut count = 0;

            for item in workflows_table.iter()? {
                if count >= limit {
                    break;
                }

                let (key, value) = item?;
                if key.value().starts_with("workflow:") {
                    let metadata: crate::persistence::models::WorkflowMetadata =
                        Self::deserialize(value.value())?;

                    metadata_list.push(metadata);
                    count += 1;
                }
            }

            metadata_list.sort_by(|a, b| b.start_timestamp.cmp(&a.start_timestamp));

            Ok(metadata_list)
        })
        .await
    }

    async fn delete_workflow_metadata_and_tasks(
        &self,
        execution_id: &str,
    ) -> Result<(), CoreError> {
        let execution_id = execution_id.to_string();
        self.execute_write(move |db| {
            let write_txn = db.begin_write()?;
            {
                let mut executions_table: Table<&str, &[u8]> =
                    write_txn.open_table(EXECUTIONS_DEF)?;
                let mut tasks_table: Table<&str, &[u8]> = write_txn.open_table(TASKS_DEF)?;

                let metadata_key = Self::workflow_metadata_key(&execution_id);
                executions_table.remove(metadata_key.as_str())?;

                let task_prefix = Self::task_prefix(&execution_id);
                let mut keys_to_delete = Vec::new();

                for item in tasks_table.iter()? {
                    let (key, _) = item?;
                    if key.value().starts_with(&task_prefix) {
                        keys_to_delete.push(key.value().to_string());
                    }
                }

                for key in keys_to_delete {
                    tasks_table.remove(key.as_str())?;
                }
            }
            write_txn.commit()?;
            Ok(())
        })
        .await
    }

    async fn load_settings(
        &self,
    ) -> Result<Option<crate::persistence::models::AppSettings>, CoreError> {
        self.load_settings().await
    }

    async fn save_settings(
        &self,
        settings: &crate::persistence::models::AppSettings,
    ) -> Result<(), CoreError> {
        self.save_settings(settings).await
    }
}
