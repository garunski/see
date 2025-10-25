//! Core store trait - minimal interface for workflow execution

use crate::errors::CoreError;
use persistence::{Workflow, WorkflowExecution, TaskExecution};
use std::sync::Arc;
use tracing::debug;

/// Store trait for workflow execution and GUI compatibility
#[async_trait::async_trait]
pub trait Store: Send + Sync {
    /// Save a workflow definition
    async fn save_workflow(&self, workflow: Workflow) -> Result<(), CoreError>;
    
    /// Get a workflow definition by ID
    async fn get_workflow(&self, id: &str) -> Result<Option<Workflow>, CoreError>;
    
    /// Save a workflow execution
    async fn save_workflow_execution(&self, execution: WorkflowExecution) -> Result<(), CoreError>;
    
    /// Get a workflow execution by ID
    async fn get_workflow_execution(&self, id: &str) -> Result<Option<WorkflowExecution>, CoreError>;
    
    /// Save a task execution
    async fn save_task_execution(&self, task: TaskExecution) -> Result<(), CoreError>;
    
    /// Get tasks for a workflow execution
    async fn get_tasks_for_workflow_execution(&self, workflow_execution_id: &str) -> Result<Vec<TaskExecution>, CoreError>;
    
    // GUI compatibility methods
    /// List all workflow executions
    async fn list_workflow_executions(&self) -> Result<Vec<WorkflowExecution>, CoreError>;
    
    /// Get tasks for a workflow (alias for compatibility)
    async fn get_tasks_for_workflow(&self, workflow_id: &str) -> Result<Vec<TaskExecution>, CoreError>;
    
    /// Save settings
    async fn save_settings(&self, settings: &persistence::AppSettings) -> Result<(), CoreError>;
    
    /// Load settings
    async fn load_settings(&self) -> Result<Option<persistence::AppSettings>, CoreError>;
    
    /// Clear all data
    async fn clear_all_data(&self) -> Result<(), CoreError>;
    
    /// Log audit event
    async fn log_audit_event(&self, event: persistence::AuditEvent) -> Result<(), CoreError>;
    
    /// List prompts
    async fn list_prompts(&self) -> Result<Vec<persistence::UserPrompt>, CoreError>;
    
    /// Save prompt
    async fn save_prompt(&self, prompt: &persistence::UserPrompt) -> Result<(), CoreError>;
    
    /// Delete prompt
    async fn delete_prompt(&self, id: &str) -> Result<(), CoreError>;
    
    /// List workflow metadata
    async fn list_workflow_metadata(&self) -> Result<Vec<persistence::WorkflowMetadata>, CoreError>;
    
    /// Delete workflow execution
    async fn delete_workflow_execution(&self, id: &str) -> Result<(), CoreError>;
    
    /// Delete workflow metadata and tasks
    async fn delete_workflow_metadata_and_tasks(&self, id: &str) -> Result<(), CoreError>;
    
    /// Get workflow with tasks
    async fn get_workflow_with_tasks(&self, id: &str) -> Result<Option<persistence::WorkflowExecution>, CoreError>;
}

/// Persistence-based store implementation
pub struct PersistenceStore {
    instance_manager: Arc<persistence::InstanceManager>,
}

impl PersistenceStore {
    pub fn new(instance_manager: Arc<persistence::InstanceManager>) -> Self {
        Self { instance_manager }
    }
}

#[async_trait::async_trait]
impl Store for PersistenceStore {
    async fn save_workflow(&self, workflow: Workflow) -> Result<(), CoreError> {
        debug!("Saving workflow: {}", workflow.name);
        self.instance_manager.create_workflow(workflow).await
            .map_err(|e| CoreError::Persistence(e))
    }
    
    async fn get_workflow(&self, id: &str) -> Result<Option<Workflow>, CoreError> {
        debug!("Getting workflow: {}", id);
        // For now, get all workflows and find the one with matching ID
        let workflows = self.instance_manager.get_all_workflows().await
            .map_err(|e| CoreError::Persistence(e))?;
        Ok(workflows.into_iter().find(|w| w.id == id))
    }
    
    async fn save_workflow_execution(&self, execution: WorkflowExecution) -> Result<(), CoreError> {
        debug!("Saving workflow execution: {}", execution.id);
        self.instance_manager.create_workflow_execution(execution).await
            .map_err(|e| CoreError::Persistence(e))
    }
    
    async fn get_workflow_execution(&self, id: &str) -> Result<Option<WorkflowExecution>, CoreError> {
        debug!("Getting workflow execution: {}", id);
        // For now, get all workflow executions and find the one with matching ID
        let executions = self.instance_manager.get_all_workflow_executions().await
            .map_err(|e| CoreError::Persistence(e))?;
        Ok(executions.into_iter().find(|e| e.id == id))
    }
    
    async fn save_task_execution(&self, task: TaskExecution) -> Result<(), CoreError> {
        debug!("Saving task execution: {}", task.id);
        self.instance_manager.create_task_execution(task).await
            .map_err(|e| CoreError::Persistence(e))
    }
    
    async fn get_tasks_for_workflow_execution(&self, workflow_execution_id: &str) -> Result<Vec<TaskExecution>, CoreError> {
        debug!("Getting tasks for workflow execution: {}", workflow_execution_id);
        self.instance_manager.get_tasks_for_workflow_execution(workflow_execution_id).await
            .map_err(|e| CoreError::Persistence(e))
    }
    
    // GUI compatibility methods
    async fn list_workflow_executions(&self) -> Result<Vec<WorkflowExecution>, CoreError> {
        debug!("Listing workflow executions");
        self.instance_manager.get_all_workflow_executions().await
            .map_err(|e| CoreError::Persistence(e))
    }
    
    async fn get_tasks_for_workflow(&self, workflow_id: &str) -> Result<Vec<TaskExecution>, CoreError> {
        debug!("Getting tasks for workflow: {}", workflow_id);
        // For now, get all task executions and filter by workflow_id
        // TODO: Implement proper filtering in InstanceManager
        let all_tasks = self.instance_manager.get_all_task_executions().await
            .map_err(|e| CoreError::Persistence(e))?;
        Ok(all_tasks.into_iter().filter(|t| t.workflow_execution_id == workflow_id).collect())
    }
    
    async fn save_settings(&self, settings: &persistence::AppSettings) -> Result<(), CoreError> {
        debug!("Saving app settings");
        // For now, just log the settings
        // TODO: Implement proper settings saving
        debug!("Settings saved: {:?}", settings);
        Ok(())
    }
    
    async fn load_settings(&self) -> Result<Option<persistence::AppSettings>, CoreError> {
        debug!("Loading app settings");
        // For now, return default settings
        // TODO: Implement proper settings loading
        Ok(Some(persistence::AppSettings::default()))
    }
    
    async fn clear_all_data(&self) -> Result<(), CoreError> {
        debug!("Clearing all data");
        // For now, just log
        // TODO: Implement data clearing
        Ok(())
    }
    
    async fn log_audit_event(&self, event: persistence::AuditEvent) -> Result<(), CoreError> {
        debug!("Logging audit event: {}", event.id);
        // For now, just log
        // TODO: Implement audit event logging
        debug!("Audit event logged: {:?}", event);
        Ok(())
    }
    
    async fn list_prompts(&self) -> Result<Vec<persistence::UserPrompt>, CoreError> {
        debug!("Listing user prompts");
        self.instance_manager.get_all_user_prompts().await
            .map_err(|e| CoreError::Persistence(e))
    }
    
    async fn save_prompt(&self, prompt: &persistence::UserPrompt) -> Result<(), CoreError> {
        debug!("Saving user prompt: {}", prompt.id);
        self.instance_manager.create_user_prompt(prompt.clone()).await
            .map_err(|e| CoreError::Persistence(e))
    }
    
    async fn delete_prompt(&self, id: &str) -> Result<(), CoreError> {
        debug!("Deleting user prompt: {}", id);
        self.instance_manager.delete_user_prompt(id).await
            .map_err(|e| CoreError::Persistence(e))
    }
    
    async fn list_workflow_metadata(&self) -> Result<Vec<persistence::WorkflowMetadata>, CoreError> {
        debug!("Listing workflow metadata");
        // For now, return empty list
        // TODO: Implement workflow metadata listing
        Ok(Vec::new())
    }
    
    async fn delete_workflow_execution(&self, id: &str) -> Result<(), CoreError> {
        debug!("Deleting workflow execution: {}", id);
        self.instance_manager.delete_workflow_execution(id).await
            .map_err(|e| CoreError::Persistence(e))
    }
    
    async fn delete_workflow_metadata_and_tasks(&self, id: &str) -> Result<(), CoreError> {
        debug!("Deleting workflow metadata and tasks: {}", id);
        // For now, just delete workflow execution
        self.instance_manager.delete_workflow_execution(id).await
            .map_err(|e| CoreError::Persistence(e))
    }
    
    async fn get_workflow_with_tasks(&self, id: &str) -> Result<Option<persistence::WorkflowExecution>, CoreError> {
        debug!("Getting workflow with tasks: {}", id);
        self.get_workflow_execution(id).await
    }
}
