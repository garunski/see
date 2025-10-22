// Key generation functions for database operations

/// Generate a key for execution timestamp lookup
pub fn execution_timestamp_key(timestamp: &str, id: &str) -> String {
    format!("{}:{}", timestamp, id)
}

/// Generate a key for workflow metadata storage
pub fn workflow_metadata_key(id: &str) -> String {
    format!("workflow:{}", id)
}

/// Generate a key for task execution storage
pub fn task_key(execution_id: &str, task_id: &str) -> String {
    format!("task:{}:{}", execution_id, task_id)
}

/// Generate a prefix for finding all tasks for an execution
pub fn task_prefix(execution_id: &str) -> String {
    format!("task:{}:", execution_id)
}
