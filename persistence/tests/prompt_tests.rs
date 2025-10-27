//! Tests for prompt store operations
//!
//! Tests save_prompt, list_prompts, delete_prompt following Single Responsibility Principle.

use chrono::Utc;
use persistence::{Prompt, Store};

async fn create_test_store() -> Store {
    Store::new(":memory:").await.unwrap()
}

fn create_test_prompt() -> Prompt {
    Prompt {
        id: "prompt-1".to_string(),
        name: "Test Prompt".to_string(),
        content: "This is a test prompt".to_string(),
        description: None,
        template: "This is a test prompt".to_string(),
        variables: Vec::new(),
        tags: Vec::new(),
        metadata: serde_json::Value::Object(serde_json::Map::new()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[tokio::test]
async fn test_save_prompt() {
    let store = create_test_store().await;
    let prompt = create_test_prompt();

    let result = store.save_prompt(&prompt).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_prompts_empty() {
    let store = create_test_store().await;

    let prompts = store.list_prompts().await.unwrap();
    assert!(prompts.is_empty());
}

#[tokio::test]
async fn test_list_prompts_multiple() {
    let store = create_test_store().await;

    // Create multiple prompts
    let prompt1 = Prompt {
        id: "prompt-1".to_string(),
        name: "Prompt 1".to_string(),
        content: "Content 1".to_string(),
        ..Default::default()
    };

    let prompt2 = Prompt {
        id: "prompt-2".to_string(),
        name: "Prompt 2".to_string(),
        content: "Content 2".to_string(),
        ..Default::default()
    };

    // Save prompts
    store.save_prompt(&prompt1).await.unwrap();
    store.save_prompt(&prompt2).await.unwrap();

    // List prompts
    let prompts = store.list_prompts().await.unwrap();
    assert_eq!(prompts.len(), 2);

    // Check that prompts are ordered by ID
    assert_eq!(prompts[0].id, "prompt-1");
    assert_eq!(prompts[1].id, "prompt-2");
}

#[tokio::test]
async fn test_delete_prompt() {
    let store = create_test_store().await;
    let prompt = create_test_prompt();

    // Save prompt
    store.save_prompt(&prompt).await.unwrap();

    // Verify it exists
    let prompts = store.list_prompts().await.unwrap();
    assert_eq!(prompts.len(), 1);

    // Delete prompt
    let result = store.delete_prompt("prompt-1").await;
    assert!(result.is_ok());

    // Verify it's gone
    let prompts = store.list_prompts().await.unwrap();
    assert!(prompts.is_empty());
}

#[tokio::test]
async fn test_delete_prompt_not_found() {
    let store = create_test_store().await;

    // Delete non-existent prompt should not error
    let result = store.delete_prompt("nonexistent").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_save_prompt_update() {
    let store = create_test_store().await;
    let mut prompt = create_test_prompt();

    // Save initial prompt
    store.save_prompt(&prompt).await.unwrap();

    // Update prompt
    prompt.name = "Updated Prompt".to_string();
    prompt.content = "Updated content".to_string();

    // Save updated prompt
    store.save_prompt(&prompt).await.unwrap();

    // Verify update
    let prompts = store.list_prompts().await.unwrap();
    assert_eq!(prompts.len(), 1);

    let retrieved_prompt = &prompts[0];
    assert_eq!(retrieved_prompt.name, "Updated Prompt");
    assert_eq!(retrieved_prompt.content, "Updated content");
}

#[tokio::test]
async fn test_prompt_serialization() {
    let store = create_test_store().await;
    let prompt = Prompt {
        id: "prompt-1".to_string(),
        name: "Test Prompt".to_string(),
        content: "This is a test prompt with special characters: !@#$%^&*()".to_string(),
        description: None,
        template: "This is a test prompt with special characters: !@#$%^&*()".to_string(),
        variables: Vec::new(),
        tags: Vec::new(),
        metadata: serde_json::Value::Object(serde_json::Map::new()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Save prompt
    store.save_prompt(&prompt).await.unwrap();

    // Retrieve and verify
    let prompts = store.list_prompts().await.unwrap();
    assert_eq!(prompts.len(), 1);

    let retrieved_prompt = &prompts[0];
    assert_eq!(retrieved_prompt.id, prompt.id);
    assert_eq!(retrieved_prompt.name, prompt.name);
    assert_eq!(retrieved_prompt.content, prompt.content);
}
