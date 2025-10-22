// Reusable table operations trait and implementations

use super::types::TableContext;
use crate::errors::CoreError;
use crate::persistence::store::keys::{
    execution_timestamp_key, task_prefix, workflow_metadata_key,
};
use crate::persistence::store::serialization::{deserialize, serialize};
use redb::{ReadOnlyTable, ReadableTable, Table};

/// Trait for common table operations to reduce code duplication
pub trait TableOperations {
    /// Get a value by key and deserialize it
    fn get_by_key<T>(&self, key: &str) -> Result<Option<T>, CoreError>
    where
        T: serde::de::DeserializeOwned;

    /// Insert a serialized value
    fn insert_serialized<T>(&mut self, key: &str, value: &T) -> Result<(), CoreError>
    where
        T: serde::Serialize;

    /// Delete entries by prefix
    fn delete_by_prefix(&mut self, prefix: &str) -> Result<Vec<String>, CoreError>;
}

impl TableOperations for Table<'_, &str, &[u8]> {
    fn get_by_key<T>(&self, key: &str) -> Result<Option<T>, CoreError>
    where
        T: serde::de::DeserializeOwned,
    {
        if let Some(serialized) = self.get(key)? {
            let value: T = deserialize(serialized.value())?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn insert_serialized<T>(&mut self, key: &str, value: &T) -> Result<(), CoreError>
    where
        T: serde::Serialize,
    {
        let serialized = serialize(value)?;
        self.insert(key, serialized.as_slice())?;
        Ok(())
    }

    fn delete_by_prefix(&mut self, prefix: &str) -> Result<Vec<String>, CoreError> {
        let mut keys_to_delete = Vec::new();

        for item in self.iter()? {
            let (key, _) = item?;
            if key.value().starts_with(prefix) {
                keys_to_delete.push(key.value().to_string());
            }
        }

        for key in &keys_to_delete {
            self.remove(key.as_str())?;
        }

        Ok(keys_to_delete)
    }
}

impl TableOperations for ReadOnlyTable<&'static str, &'static [u8]> {
    fn get_by_key<T>(&self, key: &str) -> Result<Option<T>, CoreError>
    where
        T: serde::de::DeserializeOwned,
    {
        if let Some(serialized) = self.get(key)? {
            let value: T = deserialize(serialized.value())?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn insert_serialized<T>(&mut self, _key: &str, _value: &T) -> Result<(), CoreError>
    where
        T: serde::Serialize,
    {
        Err(CoreError::Dataflow(
            "Cannot insert into read-only table".to_string(),
        ))
    }

    fn delete_by_prefix(&mut self, _prefix: &str) -> Result<Vec<String>, CoreError> {
        Err(CoreError::Dataflow(
            "Cannot delete from read-only table".to_string(),
        ))
    }
}

/// Helper functions for common workflow operations
pub struct WorkflowTableOps;

impl WorkflowTableOps {
    /// Save a workflow execution with its timestamp index
    pub fn save_execution_with_index(
        tables: &mut TableContext,
        execution: &crate::persistence::models::WorkflowExecution,
    ) -> Result<(), CoreError> {
        // Insert into executions table
        tables
            .executions_table
            .insert_serialized(&execution.id, execution)?;

        // Insert into execution_ids table with timestamp key
        let timestamp_key = execution_timestamp_key(&execution.timestamp, &execution.id);
        tables
            .execution_ids_table
            .insert(timestamp_key.as_str(), execution.id.as_str())?;

        Ok(())
    }

    /// Delete execution and all associated data
    pub fn delete_execution_complete(
        tables: &mut TableContext,
        execution_id: &str,
    ) -> Result<(), CoreError> {
        // Get execution to find timestamp for index deletion
        let execution: Option<crate::persistence::models::WorkflowExecution> =
            tables.executions_table.get_by_key(execution_id)?;

        if let Some(exec) = execution {
            // Delete from execution_ids table
            let timestamp_key = execution_timestamp_key(&exec.timestamp, execution_id);
            tables.execution_ids_table.remove(timestamp_key.as_str())?;
        }

        // Delete from executions table
        tables.executions_table.remove(execution_id)?;

        // Delete metadata
        let metadata_key = workflow_metadata_key(execution_id);
        tables.executions_table.remove(metadata_key.as_str())?;

        // Delete all associated tasks
        let task_prefix = task_prefix(execution_id);
        tables.tasks_table.delete_by_prefix(&task_prefix)?;

        Ok(())
    }

    /// Delete metadata and all associated tasks
    pub fn delete_metadata_and_tasks(
        tables: &mut TableContext,
        execution_id: &str,
    ) -> Result<(), CoreError> {
        // Delete metadata
        let metadata_key = workflow_metadata_key(execution_id);
        tables.executions_table.remove(metadata_key.as_str())?;

        // Delete all associated tasks
        let task_prefix = task_prefix(execution_id);
        tables.tasks_table.delete_by_prefix(&task_prefix)?;

        Ok(())
    }
}
