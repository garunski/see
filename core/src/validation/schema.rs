//! Manual JSON Schema loader for workflow validation

use serde_json::Value;

/// Load the manual JSON Schema from the embedded file
pub fn load_workflow_schema() -> Value {
    let schema_json = include_str!("../../schema/workflow.schema.json");
    serde_json::from_str(schema_json).expect("Failed to parse workflow schema")
}

/// Get schema version
pub fn get_schema_version() -> String {
    let schema = load_workflow_schema();
    schema["version"].as_str().unwrap_or("unknown").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_schema() {
        let schema = load_workflow_schema();
        assert!(schema.is_object());
    }

    #[test]
    fn test_schema_version() {
        let version = get_schema_version();
        assert_eq!(version, "1.0.0");
    }

    #[test]
    fn test_schema_structure() {
        let schema = load_workflow_schema();
        assert!(schema.is_object());

        // Check top-level required fields
        assert!(schema.get("$schema").is_some());
        assert!(schema.get("version").is_some());
        assert!(schema.get("definitions").is_some());
        assert!(schema.get("properties").is_some());
    }
}
