// Simplified store implementation - direct redb operations without over-engineering

#![allow(clippy::result_large_err)]

use crate::errors::CoreError;
use crate::persistence::models::{
    AppSettings, Prompt, TaskExecution, WorkflowDefinition, WorkflowExecution,
    WorkflowExecutionSummary, WorkflowMetadata,
};
use async_trait::async_trait;
use redb::{Database, ReadOnlyTable, ReadableTable, Table, TableDefinition};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task;

// Table definitions
const EXECUTIONS_TABLE: &str = "executions";
const EXECUTION_IDS_TABLE: &str = "execution_ids";
const SETTINGS_TABLE: &str = "settings";
const TASKS_TABLE: &str = "tasks";
const PROMPTS_TABLE: &str = "prompts";

const EXECUTIONS_DEF: TableDefinition<&str, &[u8]> = TableDefinition::new(EXECUTIONS_TABLE);
const EXECUTION_IDS_DEF: TableDefinition<&str, &str> = TableDefinition::new(EXECUTION_IDS_TABLE);
const SETTINGS_DEF: TableDefinition<&str, &[u8]> = TableDefinition::new(SETTINGS_TABLE);
const TASKS_DEF: TableDefinition<&str, &[u8]> = TableDefinition::new(TASKS_TABLE);
const PROMPTS_DEF: TableDefinition<&str, &[u8]> = TableDefinition::new(PROMPTS_TABLE);

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
    async fn save_prompt(&self, prompt: &Prompt) -> Result<(), CoreError>;
    async fn get_prompt(&self, id: &str) -> Result<Option<Prompt>, CoreError>;
    async fn list_prompts(&self) -> Result<Vec<Prompt>, CoreError>;
    async fn delete_prompt(&self, id: &str) -> Result<(), CoreError>;
    async fn clear_all_data(&self) -> Result<(), CoreError>;
    async fn get_workflow_definition(&self, id: &str) -> Result<WorkflowDefinition, CoreError>;
}

/// RedbStore implementation with direct operations
#[derive(Debug)]
pub struct RedbStore {
    db: Arc<Database>,
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
            let _prompts_table: Table<&str, &[u8]> = write_txn.open_table(PROMPTS_DEF)?;
        }
        write_txn.commit()?;

        Ok(Self { db: Arc::new(db) })
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

    /// Execute a read operation in a separate thread
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

    /// Execute a write operation in a separate thread
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

    /// Serialize a value to bytes using bincode
    fn serialize<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, CoreError> {
        bincode::serialize(value).map_err(|e| CoreError::Dataflow(e.to_string()))
    }

    /// Deserialize bytes to a value using bincode
    fn deserialize<T: serde::de::DeserializeOwned>(data: &[u8]) -> Result<T, CoreError> {
        bincode::deserialize(data).map_err(|e| CoreError::Dataflow(e.to_string()))
    }

    /// Generate a key for execution timestamp lookup
    fn execution_timestamp_key(timestamp: &str, id: &str) -> String {
        format!("{}:{}", timestamp, id)
    }

    /// Generate a key for workflow metadata storage
    fn workflow_metadata_key(id: &str) -> String {
        format!("workflow:{}", id)
    }

    /// Generate a key for task execution storage
    fn task_key(execution_id: &str, task_id: &str) -> String {
        format!("task:{}:{}", execution_id, task_id)
    }

    /// Generate a prefix for finding all tasks for an execution
    fn task_prefix(execution_id: &str) -> String {
        format!("task:{}:", execution_id)
    }
}

#[async_trait]
impl AuditStore for RedbStore {
    async fn save_workflow_execution(
        &self,
        execution: &WorkflowExecution,
    ) -> Result<String, CoreError> {
        let execution = execution.clone();
        let id = execution.id.clone();

        self.execute_write(move |db| {
            let write_txn = db.begin_write()?;
            {
                let mut executions_table: Table<&str, &[u8]> =
                    write_txn.open_table(EXECUTIONS_DEF)?;
                let mut execution_ids_table: Table<&str, &str> =
                    write_txn.open_table(EXECUTION_IDS_DEF)?;

                let serialized = Self::serialize(&execution)?;
                executions_table.insert(&*execution.id, serialized.as_slice())?;

                // Insert into execution_ids table with timestamp key
                let timestamp_key =
                    Self::execution_timestamp_key(&execution.timestamp, &execution.id);
                execution_ids_table.insert(timestamp_key.as_str(), execution.id.as_str())?;
            }
            write_txn.commit()?;
            Ok(id)
        })
        .await
    }

    async fn get_workflow_execution(&self, id: &str) -> Result<WorkflowExecution, CoreError> {
        let id = id.to_string();
        self.execute_read(move |db| {
            let read_txn = db.begin_read()?;
            let executions_table: ReadOnlyTable<&str, &[u8]> =
                read_txn.open_table(EXECUTIONS_DEF)?;

            if let Some(serialized) = executions_table.get(&*id)? {
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

                // Get execution to find timestamp for index deletion
                if let Some(serialized) = executions_table.get(&*id)? {
                    let execution: WorkflowExecution = Self::deserialize(serialized.value())?;
                    let timestamp_key = Self::execution_timestamp_key(&execution.timestamp, &id);
                    execution_ids_table.remove(timestamp_key.as_str())?;
                }

                // Delete from executions table
                executions_table.remove(&*id)?;

                // Delete metadata
                let metadata_key = Self::workflow_metadata_key(&id);
                executions_table.remove(metadata_key.as_str())?;

                // Delete all associated tasks
                let task_prefix = Self::task_prefix(&id);
                let mut keys_to_delete = Vec::new();
                for item in tasks_table.iter()? {
                    let (key, _) = item?;
                    if key.value().starts_with(&task_prefix) {
                        keys_to_delete.push(key.value().to_string());
                    }
                }
                for key in &keys_to_delete {
                    tasks_table.remove(key.as_str())?;
                }
            }
            write_txn.commit()?;
            Ok(())
        })
        .await
    }

    async fn save_workflow_metadata(&self, metadata: &WorkflowMetadata) -> Result<(), CoreError> {
        let metadata = metadata.clone();
        let key = Self::workflow_metadata_key(&metadata.id);

        self.execute_write(move |db| {
            let write_txn = db.begin_write()?;
            {
                let mut executions_table: Table<&str, &[u8]> =
                    write_txn.open_table(EXECUTIONS_DEF)?;
                let serialized = Self::serialize(&metadata)?;
                executions_table.insert(key.as_str(), serialized.as_slice())?;
            }
            write_txn.commit()?;
            Ok(())
        })
        .await
    }

    async fn save_task_execution(&self, task: &TaskExecution) -> Result<(), CoreError> {
        let task = task.clone();
        let key = Self::task_key(&task.execution_id, &task.task_id);

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
            let tasks_table: ReadOnlyTable<&str, &[u8]> = read_txn.open_table(TASKS_DEF)?;

            let workflow_key = Self::workflow_metadata_key(&execution_id);

            // Get metadata
            let metadata: WorkflowMetadata =
                if let Some(serialized) = workflows_table.get(&*workflow_key)? {
                    Self::deserialize(serialized.value())?
                } else {
                    return Err(CoreError::Dataflow(format!(
                        "Workflow {} not found",
                        execution_id
                    )));
                };

            // Get tasks
            let task_prefix = Self::task_prefix(&execution_id);
            let mut tasks = Vec::new();
            let mut per_task_logs = std::collections::HashMap::new();

            for item in tasks_table.iter()? {
                let (key, value) = item?;
                let key_str = key.value();
                if key_str.starts_with(&task_prefix) {
                    let task_exec: TaskExecution = Self::deserialize(value.value())?;

                    tasks.push(crate::types::TaskInfo {
                        id: task_exec.task_id.clone(),
                        name: task_exec.task_name.clone(),
                        status: task_exec.status,
                    });

                    per_task_logs.insert(task_exec.task_id.clone(), task_exec.logs.clone());
                }
            }

            // Order tasks according to metadata task_ids
            let mut ordered_tasks = Vec::new();
            for task_id in &metadata.task_ids {
                if let Some(task) = tasks.iter().find(|t| &t.id == task_id) {
                    ordered_tasks.push(task.clone());
                }
            }

            // Build execution
            let execution = WorkflowExecution {
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
            };

            Ok(execution)
        })
        .await
    }

    async fn list_workflow_metadata(
        &self,
        limit: usize,
    ) -> Result<Vec<WorkflowMetadata>, CoreError> {
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
                    let metadata: WorkflowMetadata = Self::deserialize(value.value())?;
                    metadata_list.push(metadata);
                    count += 1;
                }
            }

            // Sort by start time descending
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

                // Delete metadata
                let metadata_key = Self::workflow_metadata_key(&execution_id);
                executions_table.remove(metadata_key.as_str())?;

                // Delete all associated tasks
                let task_prefix = Self::task_prefix(&execution_id);
                let mut keys_to_delete = Vec::new();
                for item in tasks_table.iter()? {
                    let (key, _) = item?;
                    if key.value().starts_with(&task_prefix) {
                        keys_to_delete.push(key.value().to_string());
                    }
                }
                for key in &keys_to_delete {
                    tasks_table.remove(key.as_str())?;
                }
            }
            write_txn.commit()?;
            Ok(())
        })
        .await
    }

    async fn load_settings(&self) -> Result<Option<AppSettings>, CoreError> {
        self.execute_read(move |db| {
            let read_txn = db.begin_read()?;
            let settings_table: ReadOnlyTable<&str, &[u8]> = read_txn.open_table(SETTINGS_DEF)?;
            if let Some(serialized) = settings_table.get("app_settings")? {
                let settings: AppSettings = Self::deserialize(serialized.value())?;
                Ok(Some(settings))
            } else {
                Ok(None)
            }
        })
        .await
    }

    async fn save_settings(&self, settings: &AppSettings) -> Result<(), CoreError> {
        let settings = settings.clone();
        self.execute_write(move |db| {
            let write_txn = db.begin_write()?;
            {
                let mut settings_table: Table<&str, &[u8]> = write_txn.open_table(SETTINGS_DEF)?;
                let serialized = Self::serialize(&settings)?;
                settings_table.insert("app_settings", serialized.as_slice())?;
            }
            write_txn.commit()?;
            Ok(())
        })
        .await
    }

    async fn save_prompt(&self, prompt: &Prompt) -> Result<(), CoreError> {
        let prompt = prompt.clone();
        self.execute_write(move |db| {
            let write_txn = db.begin_write()?;
            {
                let mut prompts_table: Table<&str, &[u8]> = write_txn.open_table(PROMPTS_DEF)?;
                let serialized = Self::serialize(&prompt)?;
                prompts_table.insert(prompt.id.as_str(), serialized.as_slice())?;
            }
            write_txn.commit()?;
            Ok(())
        })
        .await
    }

    async fn get_prompt(&self, id: &str) -> Result<Option<Prompt>, CoreError> {
        let id = id.to_string();
        self.execute_read(move |db| {
            let read_txn = db.begin_read()?;
            let prompts_table: ReadOnlyTable<&str, &[u8]> = read_txn.open_table(PROMPTS_DEF)?;
            if let Some(serialized) = prompts_table.get(id.as_str())? {
                let prompt: Prompt = Self::deserialize(serialized.value())?;
                Ok(Some(prompt))
            } else {
                Ok(None)
            }
        })
        .await
    }

    async fn list_prompts(&self) -> Result<Vec<Prompt>, CoreError> {
        self.execute_read(move |db| {
            let read_txn = db.begin_read()?;
            let prompts_table: ReadOnlyTable<&str, &[u8]> = read_txn.open_table(PROMPTS_DEF)?;
            let mut prompts = Vec::new();
            for item in prompts_table.iter()? {
                let (_, value) = item?;
                let prompt: Prompt = Self::deserialize(value.value())?;
                prompts.push(prompt);
            }
            Ok(prompts)
        })
        .await
    }

    async fn delete_prompt(&self, id: &str) -> Result<(), CoreError> {
        let id = id.to_string();
        self.execute_write(move |db| {
            let write_txn = db.begin_write()?;
            {
                let mut prompts_table: Table<&str, &[u8]> = write_txn.open_table(PROMPTS_DEF)?;
                prompts_table.remove(id.as_str())?;
            }
            write_txn.commit()?;
            Ok(())
        })
        .await
    }

    async fn clear_all_data(&self) -> Result<(), CoreError> {
        self.execute_write(move |db| {
            let write_txn = db.begin_write()?;
            {
                let mut executions_table: Table<&str, &[u8]> =
                    write_txn.open_table(EXECUTIONS_DEF)?;
                let mut execution_ids_table: Table<&str, &str> =
                    write_txn.open_table(EXECUTION_IDS_DEF)?;
                let mut settings_table: Table<&str, &[u8]> = write_txn.open_table(SETTINGS_DEF)?;
                let mut tasks_table: Table<&str, &[u8]> = write_txn.open_table(TASKS_DEF)?;
                let mut prompts_table: Table<&str, &[u8]> = write_txn.open_table(PROMPTS_DEF)?;

                // Clear executions table
                let mut keys_to_remove = Vec::new();
                for item in executions_table.iter()? {
                    let (key, _) = item?;
                    keys_to_remove.push(key.value().to_string());
                }
                for key in &keys_to_remove {
                    executions_table.remove(key.as_str())?;
                }

                // Clear execution_ids table
                keys_to_remove.clear();
                for item in execution_ids_table.iter()? {
                    let (key, _) = item?;
                    keys_to_remove.push(key.value().to_string());
                }
                for key in &keys_to_remove {
                    execution_ids_table.remove(key.as_str())?;
                }

                // Clear settings table
                keys_to_remove.clear();
                for item in settings_table.iter()? {
                    let (key, _) = item?;
                    keys_to_remove.push(key.value().to_string());
                }
                for key in &keys_to_remove {
                    settings_table.remove(key.as_str())?;
                }

                // Clear tasks table
                keys_to_remove.clear();
                for item in tasks_table.iter()? {
                    let (key, _) = item?;
                    keys_to_remove.push(key.value().to_string());
                }
                for key in &keys_to_remove {
                    tasks_table.remove(key.as_str())?;
                }

                // Clear prompts table
                keys_to_remove.clear();
                for item in prompts_table.iter()? {
                    let (key, _) = item?;
                    keys_to_remove.push(key.value().to_string());
                }
                for key in &keys_to_remove {
                    prompts_table.remove(key.as_str())?;
                }
            }
            write_txn.commit()?;
            Ok(())
        })
        .await
    }

    async fn get_workflow_definition(&self, id: &str) -> Result<WorkflowDefinition, CoreError> {
        let id = id.to_string();
        self.execute_read(move |db| {
            let read_txn = db.begin_read()?;
            let settings_table: ReadOnlyTable<&str, &[u8]> = read_txn.open_table(SETTINGS_DEF)?;

            if let Some(serialized) = settings_table.get("app_settings")? {
                let settings: AppSettings = Self::deserialize(serialized.value())?;
                settings
                    .workflows
                    .into_iter()
                    .find(|w| w.id == id)
                    .ok_or_else(|| {
                        CoreError::Dataflow(format!("Workflow with id '{}' not found", id))
                    })
            } else {
                Err(CoreError::Dataflow("App settings not found".to_string()))
            }
        })
        .await
    }
}
