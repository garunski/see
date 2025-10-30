use crate::models::enums::{InputRequestStatus, InputType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserInputRequest {
    pub id: String,
    pub task_execution_id: String,
    pub workflow_execution_id: String,
    pub prompt_text: String,
    pub input_type: InputType,
    pub required: bool,
    pub default_value: Option<Value>,
    pub validation_rules: Value,
    pub status: InputRequestStatus,
    pub created_at: DateTime<Utc>,
    pub fulfilled_at: Option<DateTime<Utc>>,
    pub fulfilled_value: Option<String>,
}

impl Default for UserInputRequest {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            task_execution_id: String::new(),
            workflow_execution_id: String::new(),
            prompt_text: String::new(),
            input_type: InputType::String,
            required: true,
            default_value: None,
            validation_rules: Value::Object(serde_json::Map::new()),
            status: InputRequestStatus::Pending,
            created_at: now,
            fulfilled_at: None,
            fulfilled_value: None,
        }
    }
}

impl UserInputRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Request ID cannot be empty".to_string());
        }
        if self.task_execution_id.is_empty() {
            return Err("Task execution ID cannot be empty".to_string());
        }
        if self.workflow_execution_id.is_empty() {
            return Err("Workflow execution ID cannot be empty".to_string());
        }
        if self.prompt_text.is_empty() {
            return Err("Prompt text cannot be empty".to_string());
        }

        match self.status {
            InputRequestStatus::Fulfilled => {
                if self.fulfilled_at.is_none() {
                    return Err("Fulfilled requests must have fulfillment timestamp".to_string());
                }
                if self.fulfilled_value.is_none() {
                    return Err("Fulfilled requests must have a value".to_string());
                }
            }
            InputRequestStatus::Pending => {
                if self.fulfilled_at.is_some() {
                    return Err(
                        "Pending requests should not have fulfillment timestamp".to_string()
                    );
                }
                if self.fulfilled_value.is_some() {
                    return Err("Pending requests should not have a value".to_string());
                }
            }
        }

        Ok(())
    }

    pub fn is_fulfilled(&self) -> bool {
        matches!(self.status, InputRequestStatus::Fulfilled)
    }

    pub fn is_pending(&self) -> bool {
        matches!(self.status, InputRequestStatus::Pending)
    }
}
