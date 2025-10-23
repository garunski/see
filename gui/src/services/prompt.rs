use see_core::persistence::models::Prompt;

#[derive(Debug, thiserror::Error)]
pub enum PromptError {
    #[error("Database not available: {0}")]
    DatabaseUnavailable(String),
    #[error("Failed to fetch prompts: {0}")]
    FetchPromptsFailed(String),
    #[error("Failed to create prompt: {0}")]
    CreatePromptFailed(String),
    #[error("Failed to update prompt: {0}")]
    UpdatePromptFailed(String),
    #[error("Failed to delete prompt: {0}")]
    DeletePromptFailed(String),
}

pub struct PromptService;

impl PromptService {
    pub async fn fetch_prompts() -> Result<Vec<Prompt>, PromptError> {
        let store = see_core::get_global_store()
            .map_err(|e| PromptError::DatabaseUnavailable(e.to_string()))?;

        store
            .list_prompts()
            .await
            .map_err(|e| PromptError::FetchPromptsFailed(e.to_string()))
    }

    pub async fn create_prompt(prompt: Prompt) -> Result<(), PromptError> {
        let store = see_core::get_global_store()
            .map_err(|e| PromptError::DatabaseUnavailable(e.to_string()))?;

        store
            .save_prompt(&prompt)
            .await
            .map_err(|e| PromptError::CreatePromptFailed(e.to_string()))
    }

    pub async fn update_prompt(prompt: Prompt) -> Result<(), PromptError> {
        let store = see_core::get_global_store()
            .map_err(|e| PromptError::DatabaseUnavailable(e.to_string()))?;

        store
            .save_prompt(&prompt)
            .await
            .map_err(|e| PromptError::UpdatePromptFailed(e.to_string()))
    }

    pub async fn delete_prompt(id: &str) -> Result<(), PromptError> {
        let store = see_core::get_global_store()
            .map_err(|e| PromptError::DatabaseUnavailable(e.to_string()))?;

        store
            .delete_prompt(id)
            .await
            .map_err(|e| PromptError::DeletePromptFailed(e.to_string()))
    }
}
