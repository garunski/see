use crate::queries::use_update_settings_mutation;
use dioxus::prelude::*;
use dioxus_query_custom::prelude::MutationState;
use s_e_e_core::AppSettings;

pub struct SettingsMutation {
    #[allow(dead_code)]
    pub state: Signal<MutationState<()>>,
    pub mutate_fn: std::rc::Rc<dyn Fn(AppSettings)>,
}

pub fn use_settings_mutation() -> SettingsMutation {
    let (state, mutate_fn) = use_update_settings_mutation();

    SettingsMutation {
        state,
        mutate_fn: std::rc::Rc::new(mutate_fn),
    }
}
