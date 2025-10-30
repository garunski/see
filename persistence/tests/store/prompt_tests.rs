



use s_e_e_persistence::{Store, Prompt};
use chrono::Utc;

async fn create_test_store() -> Store {
    Store::new(":memory:").await.unwrap()
}

fn create_test_prompt() -> Prompt {
    Prompt {
        id: "prompt-1".to_string(),
        name: "Test Prompt".to_string(),
        content: "This is a test prompt".to_string(),
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


    store.save_prompt(&prompt1).await.unwrap();
    store.save_prompt(&prompt2).await.unwrap();


    let prompts = store.list_prompts().await.unwrap();
    assert_eq!(prompts.len(), 2);


    assert_eq!(prompts[0].id, "prompt-1");
    assert_eq!(prompts[1].id, "prompt-2");
}

#[tokio::test]
async fn test_delete_prompt() {
    let store = create_test_store().await;
    let prompt = create_test_prompt();


    store.save_prompt(&prompt).await.unwrap();


    let prompts = store.list_prompts().await.unwrap();
    assert_eq!(prompts.len(), 1);


    let result = store.delete_prompt("prompt-1").await;
    assert!(result.is_ok());


    let prompts = store.list_prompts().await.unwrap();
    assert!(prompts.is_empty());
}

#[tokio::test]
async fn test_delete_prompt_not_found() {
    let store = create_test_store().await;


    let result = store.delete_prompt("nonexistent").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_save_prompt_update() {
    let store = create_test_store().await;
    let mut prompt = create_test_prompt();


    store.save_prompt(&prompt).await.unwrap();


    prompt.name = "Updated Prompt".to_string();
    prompt.content = "Updated content".to_string();


    store.save_prompt(&prompt).await.unwrap();


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
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };


    store.save_prompt(&prompt).await.unwrap();


    let prompts = store.list_prompts().await.unwrap();
    assert_eq!(prompts.len(), 1);

    let retrieved_prompt = &prompts[0];
    assert_eq!(retrieved_prompt.id, prompt.id);
    assert_eq!(retrieved_prompt.name, prompt.name);
    assert_eq!(retrieved_prompt.content, prompt.content);
}
