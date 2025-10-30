use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct WorkflowSchema {
    pub id: String,
    pub name: String,
    pub tasks: Vec<TaskSchema>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct TaskSchema {
    pub id: String,
    pub name: String,
    #[serde(rename = "function")]
    pub _function: Value,
    #[serde(default)]
    pub next_tasks: Vec<TaskSchema>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationError {
    pub path: String,

    pub message: String,

    pub expected: Option<String>,

    pub suggestions: Vec<String>,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Validation error at '{}': {}", self.path, self.message)?;
        if let Some(ref expected) = self.expected {
            write!(f, " (expected: {})", expected)?;
        }
        if !self.suggestions.is_empty() {
            write!(f, "\nSuggestions:")?;
            for suggestion in &self.suggestions {
                write!(f, "\n  - {}", suggestion)?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for ValidationError {}

/// Collection of validation errors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationErrors {
    /// List of validation errors
    pub errors: Vec<ValidationError>,
}

impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Found {} validation errors:", self.errors.len())?;
        for error in &self.errors {
            writeln!(f, "  {}", error)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}

pub fn check_duplicate_task_ids(
    task: &TaskSchema,
    seen_ids: &mut HashSet<String>,
) -> Result<(), String> {
    if !seen_ids.insert(task.id.clone()) {
        return Err(format!("Duplicate task ID: '{}'", task.id));
    }

    for next_task in &task.next_tasks {
        check_duplicate_task_ids(next_task, seen_ids)?;
    }

    Ok(())
}

pub fn validate_no_duplicate_task_ids(workflow: &WorkflowSchema) -> Result<(), String> {
    let mut seen_ids = HashSet::new();

    for task in &workflow.tasks {
        check_duplicate_task_ids(task, &mut seen_ids)?;
    }

    Ok(())
}
