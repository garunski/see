// System prompt model tests ONLY
// Tests for SystemPrompt struct and methods

use s_e_e_persistence::SystemPrompt;

#[test]
fn test_system_prompt_creation() {
    let now = chrono::Utc::now();
    let prompt = SystemPrompt {
        id: "test-prompt".to_string(),
        name: "Test Prompt".to_string(),
        content: "Test content".to_string(),
        description: Some("A test prompt".to_string()),
        template: "Test content".to_string(),
        variables: vec!["var1".to_string(), "var2".to_string()],
        tags: vec!["test".to_string()],
        metadata: serde_json::Value::Object(serde_json::Map::new()),
        version: "1.0.0".to_string(),
        created_at: now,
        updated_at: now,
    };

    assert_eq!(prompt.id, "test-prompt");
    assert_eq!(prompt.name, "Test Prompt");
    assert_eq!(prompt.version, "1.0.0");
}

#[test]
fn test_system_prompt_validation() {
    let now = chrono::Utc::now();
    
    // Valid prompt
    let prompt = SystemPrompt {
        id: "test".to_string(),
        name: "Test".to_string(),
        content: "Test content".to_string(),
        description: None,
        template: "Test content".to_string(),
        variables: Vec::new(),
        tags: Vec::new(),
        metadata: serde_json::Value::Object(serde_json::Map::new()),
        version: "1.0.0".to_string(),
        created_at: now,
        updated_at: now,
    };
    
    assert!(prompt.validate().is_ok());
    
    // Invalid: empty content
    let invalid = SystemPrompt {
        id: "test".to_string(),
        name: "Test".to_string(),
        content: "".to_string(),
        description: None,
        template: "".to_string(),
        variables: Vec::new(),
        tags: Vec::new(),
        metadata: serde_json::Value::Object(serde_json::Map::new()),
        version: "1.0.0".to_string(),
        created_at: now,
        updated_at: now,
    };
    
    assert!(invalid.validate().is_err());
}

#[test]
fn test_system_prompt_needs_update() {
    let now = chrono::Utc::now();
    let prompt = SystemPrompt {
        id: "test".to_string(),
        name: "Test".to_string(),
        content: "Content".to_string(),
        description: None,
        template: "Content".to_string(),
        variables: Vec::new(),
        tags: Vec::new(),
        metadata: serde_json::Value::Object(serde_json::Map::new()),
        version: "1.0.0".to_string(),
        created_at: now,
        updated_at: now,
    };
    
    assert!(prompt.needs_update("1.0.1"));
    assert!(!prompt.needs_update("1.0.0"));
}

#[test]
fn test_system_prompt_default() {
    let prompt = SystemPrompt::default();
    
    assert!(!prompt.id.is_empty());
    assert!(!prompt.version.is_empty());
}

#[test]
fn test_system_prompt_serialization() {
    let now = chrono::Utc::now();
    let prompt = SystemPrompt {
        id: "test".to_string(),
        name: "Test".to_string(),
        content: "Content".to_string(),
        description: None,
        template: "Content".to_string(),
        variables: vec!["var1".to_string()],
        tags: vec!["tag1".to_string()],
        metadata: serde_json::json!({"key": "value"}),
        version: "1.0.0".to_string(),
        created_at: now,
        updated_at: now,
    };
    
    let json = serde_json::to_string(&prompt).unwrap();
    let deserialized: SystemPrompt = serde_json::from_str(&json).unwrap();
    
    assert_eq!(deserialized.id, prompt.id);
    assert_eq!(deserialized.name, prompt.name);
    assert_eq!(deserialized.version, prompt.version);
}

