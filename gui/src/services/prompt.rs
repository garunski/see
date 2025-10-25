use s_e_e_core::UserPrompt;

#[derive(Debug, thiserror::Error)]
pub enum UserPromptError {
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

pub struct UserPromptService;

impl UserPromptService {
    pub async fn fetch_prompts() -> Result<Vec<UserPrompt>, UserPromptError> {
        let store = s_e_e_core::get_global_store()
            .map_err(|e| UserPromptError::DatabaseUnavailable(e.to_string()))?;

        store
            .list_prompts()
            .await
            .map_err(|e| UserPromptError::FetchPromptsFailed(e.to_string()))
    }

    pub async fn create_prompt(prompt: UserPrompt) -> Result<(), UserPromptError> {
        let store = s_e_e_core::get_global_store()
            .map_err(|e| UserPromptError::DatabaseUnavailable(e.to_string()))?;

        store
            .save_prompt(&prompt)
            .await
            .map_err(|e| UserPromptError::CreatePromptFailed(e.to_string()))
    }

    pub async fn update_prompt(prompt: UserPrompt) -> Result<(), UserPromptError> {
        let store = s_e_e_core::get_global_store()
            .map_err(|e| UserPromptError::DatabaseUnavailable(e.to_string()))?;

        store
            .save_prompt(&prompt)
            .await
            .map_err(|e| UserPromptError::UpdatePromptFailed(e.to_string()))
    }

    pub async fn delete_prompt(id: &str) -> Result<(), UserPromptError> {
        let store = s_e_e_core::get_global_store()
            .map_err(|e| UserPromptError::DatabaseUnavailable(e.to_string()))?;

        store
            .delete_prompt(id)
            .await
            .map_err(|e| UserPromptError::DeletePromptFailed(e.to_string()))
    }
}
