use crate::queries::{CreatePromptMutation, DeletePromptMutation, UpdatePromptMutation};
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, Mutation, UseMutation};

pub struct PromptMutations {
    pub create_mutation: UseMutation<CreatePromptMutation>,
    pub update_mutation: UseMutation<UpdatePromptMutation>,
    pub delete_mutation: UseMutation<DeletePromptMutation>,
    pub is_saving: Memo<bool>,
    pub is_deleting: Memo<bool>,
}

pub fn use_prompt_mutations() -> PromptMutations {
    let create_mutation = use_mutation(Mutation::new(CreatePromptMutation));
    let update_mutation = use_mutation(Mutation::new(UpdatePromptMutation));
    let delete_mutation = use_mutation(Mutation::new(DeletePromptMutation));

    let is_saving = use_memo(move || {
        create_mutation.read().state().is_loading() || update_mutation.read().state().is_loading()
    });
    let is_deleting = use_memo(move || delete_mutation.read().state().is_loading());

    PromptMutations {
        create_mutation,
        update_mutation,
        delete_mutation,
        is_saving,
        is_deleting,
    }
}
