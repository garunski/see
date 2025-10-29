//! Tests for Prompt model
//! 
//! Tests serialization, validation following Single Responsibility Principle.

use s_e_e_persistence::Prompt;
use chrono::Utc;

#[test]
fn test_prompt_default() {
    let prompt = Prompt::default();
    
    assert!(!prompt.id.is_empty());
    assert!(prompt.name.is_empty());
    assert!(prompt.content.is_empty());
    assert!(prompt.created_at <= Utc::now());
}

#[test]
fn test_prompt_validation_success() {
    let prompt = Prompt {
        id: "prompt-1".to_string(),
        name: "Test Prompt".to_string(),
        content: "This is a test prompt".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    let result = prompt.validate();
    assert!(result.is_ok());
}

#[test]
fn test_prompt_validation_empty_id() {
    let prompt = Prompt {
        id: "".to_string(),
        name: "Test Prompt".to_string(),
        content: "This is a test prompt".to_string(),
        ..Default::default()
    };
    
    let result = prompt.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("ID cannot be empty"));
}

#[test]
fn test_prompt_validation_empty_name() {
    let prompt = Prompt {
        id: "prompt-1".to_string(),
        name: "".to_string(),
        content: "This is a test prompt".to_string(),
        ..Default::default()
    };
    
    let result = prompt.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("name cannot be empty"));
}

#[test]
fn test_prompt_validation_empty_content() {
    let prompt = Prompt {
        id: "prompt-1".to_string(),
        name: "Test Prompt".to_string(),
        content: "".to_string(),
        ..Default::default()
    };
    
    let result = prompt.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("content cannot be empty"));
}

#[test]
fn test_prompt_update_content() {
    let mut prompt = Prompt {
        id: "prompt-1".to_string(),
        name: "Test Prompt".to_string(),
        content: "Old content".to_string(),
        created_at: Utc::now(),
    };
    
    prompt.update_content("New content".to_string());
    
    assert_eq!(prompt.content, "New content");
}

#[test]
fn test_prompt_update_name() {
    let mut prompt = Prompt {
        id: "prompt-1".to_string(),
        name: "Old Name".to_string(),
        content: "Test content".to_string(),
        created_at: Utc::now(),
    };
    
    prompt.update_name("New Name".to_string());
    
    assert_eq!(prompt.name, "New Name");
}

#[test]
fn test_prompt_serialization() {
    let prompt = Prompt {
        id: "prompt-1".to_string(),
        name: "Test Prompt".to_string(),
        content: "This is a test prompt".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // Test serialization
    let json = serde_json::to_string(&prompt).unwrap();
    assert!(json.contains("prompt-1"));
    assert!(json.contains("Test Prompt"));
    assert!(json.contains("This is a test prompt"));
    
    // Test deserialization
    let deserialized: Prompt = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, prompt.id);
    assert_eq!(deserialized.name, prompt.name);
    assert_eq!(deserialized.content, prompt.content);
}
