use crate::queries::UpdateSettingsMutation;
use dioxus_query::prelude::{use_mutation, Mutation, UseMutation};

pub struct SettingsMutation {
    pub update_mutation: UseMutation<UpdateSettingsMutation>,
}

pub fn use_settings_mutation() -> SettingsMutation {
    let update_mutation = use_mutation(Mutation::new(UpdateSettingsMutation));

    SettingsMutation { update_mutation }
}
