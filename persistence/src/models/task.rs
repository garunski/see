use crate::models::TaskExecutionStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskExecution {
    pub id: String,
    pub workflow_id: String,
    pub name: String,
    pub status: TaskExecutionStatus,
    pub output: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub user_input: Option<String>,
    pub input_request_id: Option<String>,
    pub prompt_id: Option<String>,
}

impl Default for TaskExecution {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            workflow_id: String::new(),
            name: String::new(),
            status: TaskExecutionStatus::Pending,
            output: None,
            error: None,
            created_at: now,
            completed_at: None,
            user_input: None,
            input_request_id: None,
            prompt_id: None,
        }
    }
}

impl TaskExecution {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Task ID cannot be empty".to_string());
        }
        if self.workflow_id.is_empty() {
            return Err("Workflow ID cannot be empty".to_string());
        }
        if self.name.is_empty() {
            return Err("Task name cannot be empty".to_string());
        }

        match self.status {
            TaskExecutionStatus::Complete | TaskExecutionStatus::Failed => {
                if self.completed_at.is_none() {
                    return Err("Completed tasks must have completion timestamp".to_string());
                }
            }
            TaskExecutionStatus::WaitingForInput => {
                if self.completed_at.is_some() {
                    return Err("Waiting tasks should not have completion timestamp".to_string());
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn is_finished(&self) -> bool {
        matches!(
            self.status,
            TaskExecutionStatus::Complete | TaskExecutionStatus::Failed
        )
    }

    pub fn is_waiting_for_input(&self) -> bool {
        matches!(self.status, TaskExecutionStatus::WaitingForInput)
    }

    pub fn has_user_input(&self) -> bool {
        self.user_input.is_some()
    }

    pub fn get_input_request_id(&self) -> Option<&str> {
        self.input_request_id.as_deref()
    }
}
