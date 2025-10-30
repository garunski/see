


use s_e_e_persistence::SystemWorkflow;

#[test]
fn test_system_workflow_creation() {
    let now = chrono::Utc::now();
    let workflow = SystemWorkflow {
        id: "test-workflow".to_string(),
        name: "Test Workflow".to_string(),
        description: Some("A test workflow".to_string()),
        content: r#"{"id":"test","name":"Test","tasks":[]}"#.to_string(),
        version: "1.0.0".to_string(),
        created_at: now,
        updated_at: now,
    };

    assert_eq!(workflow.id, "test-workflow");
    assert_eq!(workflow.name, "Test Workflow");
    assert_eq!(workflow.version, "1.0.0");
}

#[test]
fn test_system_workflow_validation() {
    let now = chrono::Utc::now();


    let workflow = SystemWorkflow {
        id: "test-workflow".to_string(),
        name: "Test Workflow".to_string(),
        description: Some("A test workflow".to_string()),
        content: r#"{"id":"test","name":"Test","tasks":[]}"#.to_string(),
        version: "1.0.0".to_string(),
        created_at: now,
        updated_at: now,
    };

    assert!(workflow.validate().is_ok());


    let invalid = SystemWorkflow {
        id: "test".to_string(),
        name: "Test".to_string(),
        description: None,
        content: "".to_string(),
        version: "1.0.0".to_string(),
        created_at: now,
        updated_at: now,
    };

    assert!(invalid.validate().is_err());
}

#[test]
fn test_system_workflow_needs_update() {
    let now = chrono::Utc::now();
    let workflow = SystemWorkflow {
        id: "test".to_string(),
        name: "Test".to_string(),
        description: None,
        content: r#"{"id":"test","name":"Test","tasks":[]}"#.to_string(),
        version: "1.0.0".to_string(),
        created_at: now,
        updated_at: now,
    };

    assert!(workflow.needs_update("1.0.1"));
    assert!(!workflow.needs_update("1.0.0"));
}

#[test]
fn test_system_workflow_default() {
    let workflow = SystemWorkflow::default();

    assert!(!workflow.id.is_empty());
    assert!(!workflow.version.is_empty());
}

#[test]
fn test_system_workflow_serialization() {
    let now = chrono::Utc::now();
    let workflow = SystemWorkflow {
        id: "test".to_string(),
        name: "Test".to_string(),
        description: None,
        content: r#"{"id":"test","name":"Test","tasks":[]}"#.to_string(),
        version: "1.0.0".to_string(),
        created_at: now,
        updated_at: now,
    };

    let json = serde_json::to_string(&workflow).unwrap();
    let deserialized: SystemWorkflow = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, workflow.id);
    assert_eq!(deserialized.name, workflow.name);
    assert_eq!(deserialized.version, workflow.version);
}

