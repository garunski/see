use crate::services::prompt::UserPromptService;
use dioxus_query::prelude::*;
use s_e_e_core::Prompt;

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct GetPrompts;

impl QueryCapability for GetPrompts {
    type Ok = Vec<Prompt>;
    type Err = String;
    type Keys = ();

    async fn run(&self, _: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        UserPromptService::fetch_prompts()
            .await
            .map_err(|e| e.to_string())
    }
}

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct GetPrompt;

impl QueryCapability for GetPrompt {
    type Ok = Option<Prompt>;
    type Err = String;
    type Keys = String;

    async fn run(&self, id: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        let prompts = UserPromptService::fetch_prompts()
            .await
            .map_err(|e| e.to_string())?;

        Ok(prompts.into_iter().find(|p| p.id == *id))
    }
}

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct CreatePromptMutation;

impl MutationCapability for CreatePromptMutation {
    type Ok = ();
    type Err = String;
    type Keys = Prompt;

    async fn run(&self, prompt: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        UserPromptService::create_prompt(prompt.clone())
            .await
            .map_err(|e| e.to_string())
    }

    async fn on_settled(&self, prompt: &Self::Keys, _: &Result<Self::Ok, Self::Err>) {
        QueriesStorage::<GetPrompts>::invalidate_matching(()).await;
        QueriesStorage::<GetPrompt>::invalidate_matching(prompt.id.clone()).await;
    }
}

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct UpdatePromptMutation;

impl MutationCapability for UpdatePromptMutation {
    type Ok = ();
    type Err = String;
    type Keys = Prompt;

    async fn run(&self, prompt: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        UserPromptService::update_prompt(prompt.clone())
            .await
            .map_err(|e| e.to_string())
    }

    async fn on_settled(&self, prompt: &Self::Keys, _: &Result<Self::Ok, Self::Err>) {
        QueriesStorage::<GetPrompts>::invalidate_matching(()).await;
        QueriesStorage::<GetPrompt>::invalidate_matching(prompt.id.clone()).await;
    }
}

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct DeletePromptMutation;

impl MutationCapability for DeletePromptMutation {
    type Ok = ();
    type Err = String;
    type Keys = String;

    async fn run(&self, id: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        UserPromptService::delete_prompt(id)
            .await
            .map_err(|e| e.to_string())
    }

    async fn on_settled(&self, id: &Self::Keys, _: &Result<Self::Ok, Self::Err>) {
        QueriesStorage::<GetPrompts>::invalidate_matching(()).await;
        QueriesStorage::<GetPrompt>::invalidate_matching(id.clone()).await;
    }
}
