//! Store trait and persistence implementation

use crate::errors::CoreError;
use persistence::{Workflow, WorkflowExecution, TaskExecution, UserPrompt, AiPrompt, Setting};
use std::sync::Arc;
use tracing::debug;

/// Store trait for data persistence
#[async_trait::async_trait]
pub trait Store: Send + Sync {
    /// Save a workflow definition
    async fn save_workflow(&self, workflow: Workflow) -> Result<(), CoreError>;
    
    /// Get a workflow definition by ID
    async fn get_workflow(&self, id: &str) -> Result<Option<Workflow>, CoreError>;
    
    /// List all workflow definitions
    async fn list_workflows(&self) -> Result<Vec<Workflow>, CoreError>;
    
    /// Save a workflow execution
    async fn save_workflow_execution(&self, execution: WorkflowExecution) -> Result<(), CoreError>;
    
    /// Get a workflow execution by ID
    async fn get_workflow_execution(&self, id: &str) -> Result<Option<WorkflowExecution>, CoreError>;
    
    /// List all workflow executions
    async fn list_workflow_executions(&self) -> Result<Vec<WorkflowExecution>, CoreError>;
    
    /// Save a task execution
    async fn save_task_execution(&self, task: TaskExecution) -> Result<(), CoreError>;
    
    /// Get tasks for a workflow execution
    async fn get_tasks_for_workflow_execution(&self, workflow_execution_id: &str) -> Result<Vec<TaskExecution>, CoreError>;
    
    /// Save an AI prompt
    async fn save_prompt(&self, prompt: AiPrompt) -> Result<(), CoreError>;
    
    /// Get an AI prompt by ID
    async fn get_prompt(&self, id: &str) -> Result<Option<AiPrompt>, CoreError>;
    
    /// List all AI prompts
    async fn list_prompts(&self) -> Result<Vec<AiPrompt>, CoreError>;
    
    /// Delete an AI prompt
    async fn delete_prompt(&self, id: &str) -> Result<(), CoreError>;
    
    /// Save a setting
    async fn save_setting(&self, setting: Setting) -> Result<(), CoreError>;
    
    /// Get a setting by key
    async fn get_setting(&self, key: &str) -> Result<Option<Setting>, CoreError>;
    
    /// List all settings
    async fn list_settings(&self) -> Result<Vec<Setting>, CoreError>;
    
    /// Load settings (for GUI compatibility)
    async fn load_settings(&self) -> Result<Option<persistence::AppSettings>, CoreError>;
    
    /// Save settings (for GUI compatibility)
    async fn save_settings(&self, settings: &persistence::AppSettings) -> Result<(), CoreError>;
    
    /// Clear all data
    async fn clear_all_data(&self) -> Result<(), CoreError>;
    
    /// Delete workflow execution
    async fn delete_workflow_execution(&self, id: &str) -> Result<(), CoreError>;
    
    /// Delete workflow metadata and tasks
    async fn delete_workflow_metadata_and_tasks(&self, id: &str) -> Result<(), CoreError>;
    
    /// Get workflow with tasks (for GUI compatibility)
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
        // For now, we'll need to implement this in InstanceManager
        // TODO: Add get_workflow method to InstanceManager
        Ok(None)
    }
    
    async fn list_workflows(&self) -> Result<Vec<Workflow>, CoreError> {
        debug!("Listing workflows");
        self.instance_manager.get_all_workflows().await
            .map_err(|e| CoreError::Persistence(e))
    }
    
    async fn save_workflow_execution(&self, execution: WorkflowExecution) -> Result<(), CoreError> {
        debug!("Saving workflow execution: {}", execution.id);
        self.instance_manager.create_workflow_execution(execution).await
            .map_err(|e| CoreError::Persistence(e))
    }
    
    async fn get_workflow_execution(&self, id: &str) -> Result<Option<WorkflowExecution>, CoreError> {
        debug!("Getting workflow execution: {}", id);
        // For now, we'll need to implement this in InstanceManager
        // TODO: Add get_workflow_execution method to InstanceManager
        Ok(None)
    }
    
    async fn list_workflow_executions(&self) -> Result<Vec<WorkflowExecution>, CoreError> {
        debug!("Listing workflow executions");
        self.instance_manager.get_all_workflow_executions().await
            .map_err(|e| CoreError::Persistence(e))
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
    
    async fn save_prompt(&self, prompt: AiPrompt) -> Result<(), CoreError> {
        debug!("Saving AI prompt: {}", prompt.name);
        self.instance_manager.create_ai_prompt(prompt).await
            .map_err(|e| CoreError::Persistence(e))
    }
    
    async fn get_prompt(&self, id: &str) -> Result<Option<AiPrompt>, CoreError> {
        debug!("Getting AI prompt: {}", id);
        // For now, we'll need to implement this in InstanceManager
        // TODO: Add get_ai_prompt method to InstanceManager
        Ok(None)
    }
    
    async fn list_prompts(&self) -> Result<Vec<AiPrompt>, CoreError> {
        debug!("Listing AI prompts");
        self.instance_manager.get_all_ai_prompts().await
            .map_err(|e| CoreError::Persistence(e))
    }
    
    async fn delete_prompt(&self, id: &str) -> Result<(), CoreError> {
        debug!("Deleting AI prompt: {}", id);
        // For now, we'll need to implement this in InstanceManager
        // TODO: Add delete_ai_prompt method to InstanceManager
        Ok(())
    }
    
    async fn save_setting(&self, setting: Setting) -> Result<(), CoreError> {
        debug!("Saving setting: {}", setting.key);
        self.instance_manager.create_setting(setting).await
            .map_err(|e| CoreError::Persistence(e))
    }
    
    async fn get_setting(&self, key: &str) -> Result<Option<Setting>, CoreError> {
        debug!("Getting setting: {}", key);
        self.instance_manager.get_setting(key).await
            .map_err(|e| CoreError::Persistence(e))
    }
    
    async fn list_settings(&self) -> Result<Vec<Setting>, CoreError> {
        debug!("Listing settings");
        self.instance_manager.get_all_settings().await
            .map_err(|e| CoreError::Persistence(e))
    }
    
    async fn load_settings(&self) -> Result<Option<persistence::AppSettings>, CoreError> {
        debug!("Loading app settings");
        // For now, return default settings
        // TODO: Implement proper settings loading
        Ok(Some(persistence::AppSettings::default()))
    }
    
    async fn save_settings(&self, settings: &persistence::AppSettings) -> Result<(), CoreError> {
        debug!("Saving app settings");
        // For now, just log the settings
        // TODO: Implement proper settings saving
        debug!("Settings saved: {:?}", settings);
        Ok(())
    }
    
    async fn clear_all_data(&self) -> Result<(), CoreError> {
        debug!("Clearing all data");
        // For now, just log
        // TODO: Implement data clearing
        Ok(())
    }
    
    async fn delete_workflow_execution(&self, id: &str) -> Result<(), CoreError> {
        debug!("Deleting workflow execution: {}", id);
        // For now, just log
        // TODO: Implement workflow execution deletion
        Ok(())
    }
    
    async fn delete_workflow_metadata_and_tasks(&self, id: &str) -> Result<(), CoreError> {
        debug!("Deleting workflow metadata and tasks: {}", id);
        // For now, just log
        // TODO: Implement workflow metadata and tasks deletion
        Ok(())
    }
    
    async fn get_workflow_with_tasks(&self, id: &str) -> Result<Option<persistence::WorkflowExecution>, CoreError> {
        debug!("Getting workflow with tasks: {}", id);
        // For now, just log
        // TODO: Implement workflow with tasks retrieval
        Ok(None)
    }
}

/// Simple in-memory store for fallback
pub struct SimpleStore {
    data: std::sync::Mutex<std::collections::HashMap<String, String>>,
}

impl SimpleStore {
    pub fn new() -> Self {
        Self {
            data: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    pub fn save(&self, key: String, value: String) -> Result<(), CoreError> {
        let mut data = self.data.lock()
            .map_err(|e| CoreError::MutexLock(format!("Failed to lock store: {}", e)))?;
        data.insert(key, value);
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, CoreError> {
        let data = self.data.lock()
            .map_err(|e| CoreError::MutexLock(format!("Failed to lock store: {}", e)))?;
        Ok(data.get(key).cloned())
    }
}

#[async_trait::async_trait]
impl Store for SimpleStore {
    async fn save_workflow(&self, _workflow: Workflow) -> Result<(), CoreError> {
        // SimpleStore doesn't support workflow persistence
        Ok(())
    }
    
    async fn get_workflow(&self, _id: &str) -> Result<Option<Workflow>, CoreError> {
        Ok(None)
    }
    
    async fn list_workflows(&self) -> Result<Vec<Workflow>, CoreError> {
        Ok(Vec::new())
    }
    
    async fn save_workflow_execution(&self, _execution: WorkflowExecution) -> Result<(), CoreError> {
        // SimpleStore doesn't support workflow execution persistence
        Ok(())
    }
    
    async fn get_workflow_execution(&self, _id: &str) -> Result<Option<WorkflowExecution>, CoreError> {
        Ok(None)
    }
    
    async fn list_workflow_executions(&self) -> Result<Vec<WorkflowExecution>, CoreError> {
        Ok(Vec::new())
    }
    
    async fn save_task_execution(&self, _task: TaskExecution) -> Result<(), CoreError> {
        // SimpleStore doesn't support task execution persistence
        Ok(())
    }
    
    async fn get_tasks_for_workflow_execution(&self, _workflow_execution_id: &str) -> Result<Vec<TaskExecution>, CoreError> {
        Ok(Vec::new())
    }
    
    async fn save_prompt(&self, _prompt: AiPrompt) -> Result<(), CoreError> {
        // SimpleStore doesn't support prompt persistence
        Ok(())
    }
    
    async fn get_prompt(&self, _id: &str) -> Result<Option<AiPrompt>, CoreError> {
        Ok(None)
    }
    
    async fn list_prompts(&self) -> Result<Vec<AiPrompt>, CoreError> {
        Ok(Vec::new())
    }
    
    async fn delete_prompt(&self, _id: &str) -> Result<(), CoreError> {
        // SimpleStore doesn't support prompt deletion
        Ok(())
    }
    
    async fn save_setting(&self, setting: Setting) -> Result<(), CoreError> {
        self.save(setting.key, serde_json::to_string(&setting.value).unwrap_or_default())
    }
    
    async fn get_setting(&self, key: &str) -> Result<Option<Setting>, CoreError> {
        if let Some(value) = self.get(key)? {
            if let Ok(json_value) = serde_json::from_str(&value) {
                return Ok(Some(Setting::new(key.to_string(), json_value)));
            }
        }
        Ok(None)
    }
    
    async fn list_settings(&self) -> Result<Vec<Setting>, CoreError> {
        let data = self.data.lock()
            .map_err(|e| CoreError::MutexLock(format!("Failed to lock store: {}", e)))?;
        
        let mut settings = Vec::new();
        for (key, value) in data.iter() {
            if let Ok(json_value) = serde_json::from_str(value) {
                settings.push(Setting::new(key.clone(), json_value));
            }
        }
        Ok(settings)
    }
    
    async fn load_settings(&self) -> Result<Option<persistence::AppSettings>, CoreError> {
        Ok(Some(persistence::AppSettings::default()))
    }
    
    async fn save_settings(&self, _settings: &persistence::AppSettings) -> Result<(), CoreError> {
        Ok(())
    }
    
    async fn clear_all_data(&self) -> Result<(), CoreError> {
        let mut data = self.data.lock()
            .map_err(|e| CoreError::MutexLock(format!("Failed to lock store: {}", e)))?;
        data.clear();
        Ok(())
    }
    
    async fn delete_workflow_execution(&self, _id: &str) -> Result<(), CoreError> {
        Ok(())
    }
    
    async fn delete_workflow_metadata_and_tasks(&self, _id: &str) -> Result<(), CoreError> {
        Ok(())
    }
    
    async fn get_workflow_with_tasks(&self, _id: &str) -> Result<Option<persistence::WorkflowExecution>, CoreError> {
        Ok(None)
    }
}