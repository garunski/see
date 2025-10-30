use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub is_default: bool,
    pub is_edited: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for WorkflowDefinition {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            description: None,
            content: String::new(),
            is_default: false,
            is_edited: false,
            created_at: now,
            updated_at: now,
        }
    }
}

impl WorkflowDefinition {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_default_workflows() -> Vec<WorkflowDefinition> {
        vec![
            WorkflowDefinition {
                id: "default-simple".to_string(),
                name: "Simple Workflow".to_string(),
                description: Some("A simple workflow with one task".to_string()),
                content: r#"{"id":"simple","name":"Simple Workflow","tasks":[{"id":"task1","name":"Echo Hello","function":{"cli_command":{"command":"echo","args":["Hello World"]}},"next_tasks":[]}]}"#.to_string(),
                is_default: true,
                is_edited: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            WorkflowDefinition {
                id: "default-parallel".to_string(),
                name: "Parallel Workflow".to_string(),
                description: Some("A workflow with parallel tasks".to_string()),
                content: r#"{"id":"parallel","name":"Parallel Workflow","tasks":[{"id":"task1","name":"Task 1","function":{"cli_command":{"command":"echo","args":["Task 1"]}},"next_tasks":[]},{"id":"task2","name":"Task 2","function":{"cli_command":{"command":"echo","args":["Task 2"]}},"next_tasks":[]}]}"#.to_string(),
                is_default: true,
                is_edited: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            WorkflowDefinition {
                id: "default-nested".to_string(),
                name: "Nested Workflow".to_string(),
                description: Some("A workflow with nested task dependencies".to_string()),
                content: r#"{"id":"nested","name":"Nested Workflow","tasks":[{"id":"task1","name":"First Task","function":{"cli_command":{"command":"echo","args":["First"]}},"next_tasks":[{"id":"task2","name":"Second Task","function":{"cli_command":{"command":"echo","args":["Second"]}},"next_tasks":[]}]}]}"#.to_string(),
                is_default: true,
                is_edited: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ]
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Workflow ID cannot be empty".to_string());
        }
        if self.name.is_empty() {
            return Err("Workflow name cannot be empty".to_string());
        }
        if self.content.is_empty() {
            return Err("Workflow content cannot be empty".to_string());
        }

        serde_json::from_str::<serde_json::Value>(&self.content)
            .map_err(|e| format!("Invalid JSON content: {}", e))?;

        Ok(())
    }
}
