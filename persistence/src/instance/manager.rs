//! Instance manager for multi-instance coordination

use std::sync::Arc;
use uuid::Uuid;
use tracing::{debug, info};

use crate::db::DatabasePool;
use crate::error::PersistenceError;
use crate::models::*;
use crate::store::*;

/// Manages instance-specific database operations
pub struct InstanceManager {
    instance_id: String,
    workflow_store: WorkflowStore,
    workflow_execution_store: WorkflowExecutionStore,
    task_execution_store: TaskExecutionStore,
    user_prompt_store: UserPromptStore,
    ai_prompt_store: AiPromptStore,
    settings_store: SettingsStore,
}

impl InstanceManager {
    /// Create a new instance manager
    pub fn new(pool: Arc<DatabasePool>) -> Self {
        let instance_id = Uuid::new_v4().to_string();
        info!("Creating instance manager with ID: {}", instance_id);
        
        Self {
            workflow_store: WorkflowStore::new(pool.clone()),
            workflow_execution_store: WorkflowExecutionStore::new(pool.clone()),
            task_execution_store: TaskExecutionStore::new(pool.clone()),
            user_prompt_store: UserPromptStore::new(pool.clone()),
            ai_prompt_store: AiPromptStore::new(pool.clone()),
            settings_store: SettingsStore::new(pool.clone()),
            instance_id,
        }
    }
    
    /// Get the instance ID
    pub fn get_instance_id(&self) -> &str {
        &self.instance_id
    }
    
    /// Create a workflow with instance ID
    pub async fn create_workflow(&self, mut workflow: Workflow) -> Result<(), PersistenceError> {
        debug!("Creating workflow for instance {}: {}", self.instance_id, workflow.name);
        
        // Add instance ID to metadata
        if let Some(metadata) = workflow.metadata.as_object_mut() {
            metadata.insert("instance_id".to_string(), serde_json::Value::String(self.instance_id.clone()));
        }
        
        self.workflow_store.save(&workflow).await
    }
    
    /// Get workflows for this instance
    pub async fn get_workflows_for_instance(&self) -> Result<Vec<Workflow>, PersistenceError> {
        debug!("Getting workflows for instance: {}", self.instance_id);
        self.workflow_store.list_by_instance(&self.instance_id, 1000, 0).await
    }
    
    /// Get all workflows (across all instances)
    pub async fn get_all_workflows(&self) -> Result<Vec<Workflow>, PersistenceError> {
        debug!("Getting all workflows");
        self.workflow_store.list(1000, 0).await
    }
    
    /// Create a workflow execution with instance ID
    pub async fn create_workflow_execution(&self, mut execution: WorkflowExecution) -> Result<(), PersistenceError> {
        debug!("Creating workflow execution for instance {}: {}", self.instance_id, execution.id);
        
        // Add instance ID to metadata
        if let Some(metadata) = execution.metadata.as_object_mut() {
            metadata.insert("instance_id".to_string(), serde_json::Value::String(self.instance_id.clone()));
        }
        
        self.workflow_execution_store.save(&execution).await
    }
    
    /// Get workflow executions for this instance
    pub async fn get_workflow_executions_for_instance(&self) -> Result<Vec<WorkflowExecution>, PersistenceError> {
        debug!("Getting workflow executions for instance: {}", self.instance_id);
        self.workflow_execution_store.list_by_instance(&self.instance_id, 1000, 0).await
    }
    
    /// Get all workflow executions
    pub async fn get_all_workflow_executions(&self) -> Result<Vec<WorkflowExecution>, PersistenceError> {
        debug!("Getting all workflow executions");
        self.workflow_execution_store.list(1000, 0).await
    }
    
    /// Get all task executions
    pub async fn get_all_task_executions(&self) -> Result<Vec<TaskExecution>, PersistenceError> {
        debug!("Getting all task executions");
        self.task_execution_store.list(1000, 0).await
    }
    
    /// Create a task execution with instance ID
    pub async fn create_task_execution(&self, mut task: TaskExecution) -> Result<(), PersistenceError> {
        debug!("Creating task execution for instance {}: {}", self.instance_id, task.id);
        
        // Add instance ID to metadata
        if let Some(metadata) = task.metadata.as_object_mut() {
            metadata.insert("instance_id".to_string(), serde_json::Value::String(self.instance_id.clone()));
        }
        
        self.task_execution_store.save(&task).await
    }
    
    /// Get task executions for a workflow execution
    pub async fn get_tasks_for_workflow_execution(&self, workflow_execution_id: &str) -> Result<Vec<TaskExecution>, PersistenceError> {
        debug!("Getting tasks for workflow execution: {}", workflow_execution_id);
        self.task_execution_store.list_by_workflow_execution(workflow_execution_id).await
    }
    
    /// Create a user prompt with instance ID
    pub async fn create_user_prompt(&self, mut prompt: UserPrompt) -> Result<(), PersistenceError> {
        debug!("Creating user prompt for instance {}: {}", self.instance_id, prompt.name);
        
        // Add instance ID to metadata
        if let Some(metadata) = prompt.metadata.as_object_mut() {
            metadata.insert("instance_id".to_string(), serde_json::Value::String(self.instance_id.clone()));
        }
        
        self.user_prompt_store.save(&prompt).await
    }
    
    /// Get user prompts for this instance
    pub async fn get_user_prompts_for_instance(&self) -> Result<Vec<UserPrompt>, PersistenceError> {
        debug!("Getting user prompts for instance: {}", self.instance_id);
        self.user_prompt_store.list_by_instance(&self.instance_id).await
    }
    
    /// Get all user prompts
    pub async fn get_all_user_prompts(&self) -> Result<Vec<UserPrompt>, PersistenceError> {
        debug!("Getting all user prompts");
        self.user_prompt_store.list().await
    }
    
    /// Create an AI prompt with instance ID
    pub async fn create_ai_prompt(&self, mut prompt: AiPrompt) -> Result<(), PersistenceError> {
        debug!("Creating AI prompt for instance {}: {}", self.instance_id, prompt.name);
        
        // Add instance ID to metadata
        if let Some(metadata) = prompt.metadata.as_object_mut() {
            metadata.insert("instance_id".to_string(), serde_json::Value::String(self.instance_id.clone()));
        }
        
        self.ai_prompt_store.save(&prompt).await
    }
    
    /// Get AI prompts for this instance
    pub async fn get_ai_prompts_for_instance(&self) -> Result<Vec<AiPrompt>, PersistenceError> {
        debug!("Getting AI prompts for instance: {}", self.instance_id);
        self.ai_prompt_store.list_by_instance(&self.instance_id).await
    }
    
    /// Get all AI prompts
    pub async fn get_all_ai_prompts(&self) -> Result<Vec<AiPrompt>, PersistenceError> {
        debug!("Getting all AI prompts");
        self.ai_prompt_store.list().await
    }
    
    /// Create a setting with instance ID
    pub async fn create_setting(&self, mut setting: Setting) -> Result<(), PersistenceError> {
        debug!("Creating setting for instance {}: {}", self.instance_id, setting.key);
        
        // Add instance ID to metadata
        if let Some(metadata) = setting.metadata.as_object_mut() {
            metadata.insert("instance_id".to_string(), serde_json::Value::String(self.instance_id.clone()));
        }
        
        self.settings_store.save(&setting).await
    }
    
    /// Get settings for this instance
    pub async fn get_settings_for_instance(&self) -> Result<Vec<Setting>, PersistenceError> {
        debug!("Getting settings for instance: {}", self.instance_id);
        self.settings_store.list_by_instance(&self.instance_id).await
    }
    
    /// Get all settings
    pub async fn get_all_settings(&self) -> Result<Vec<Setting>, PersistenceError> {
        debug!("Getting all settings");
        self.settings_store.list().await
    }
    
    /// Get a setting by key
    pub async fn get_setting(&self, key: &str) -> Result<Option<Setting>, PersistenceError> {
        debug!("Getting setting: {}", key);
        self.settings_store.get(key).await
    }
    
    /// Set a string setting
    pub async fn set_string_setting(&self, key: &str, value: &str) -> Result<(), PersistenceError> {
        debug!("Setting string setting: {} = {}", key, value);
        self.settings_store.set_string(key, value).await
    }
    
    /// Set a boolean setting
    pub async fn set_bool_setting(&self, key: &str, value: bool) -> Result<(), PersistenceError> {
        debug!("Setting boolean setting: {} = {}", key, value);
        self.settings_store.set_bool(key, value).await
    }
    
    /// Set a JSON setting
    pub async fn set_json_setting(&self, key: &str, value: serde_json::Value) -> Result<(), PersistenceError> {
        debug!("Setting JSON setting: {} = {}", key, value);
        self.settings_store.set_json(key, value).await
    }
    
    /// Get workflow store reference
    pub fn workflow_store(&self) -> &WorkflowStore {
        &self.workflow_store
    }
    
    /// Get workflow execution store reference
    pub fn workflow_execution_store(&self) -> &WorkflowExecutionStore {
        &self.workflow_execution_store
    }
    
    /// Get task execution store reference
    pub fn task_execution_store(&self) -> &TaskExecutionStore {
        &self.task_execution_store
    }
    
    /// Get user prompt store reference
    pub fn user_prompt_store(&self) -> &UserPromptStore {
        &self.user_prompt_store
    }
    
    /// Get AI prompt store reference
    pub fn ai_prompt_store(&self) -> &AiPromptStore {
        &self.ai_prompt_store
    }
    
    /// Get settings store reference
    pub fn settings_store(&self) -> &SettingsStore {
        &self.settings_store
    }
    
    /// Delete a user prompt
    pub async fn delete_user_prompt(&self, id: &str) -> Result<(), PersistenceError> {
        debug!("Deleting user prompt: {}", id);
        self.user_prompt_store.delete(id).await
    }
    
    /// Delete a workflow execution
    pub async fn delete_workflow_execution(&self, id: &str) -> Result<(), PersistenceError> {
        debug!("Deleting workflow execution: {}", id);
        self.workflow_execution_store.delete(id).await
    }
}