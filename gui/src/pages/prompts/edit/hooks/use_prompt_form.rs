use dioxus::prelude::*;
use s_e_e_core::Prompt;

pub struct PromptFormState {
    pub prompt_id: Signal<String>,
    pub name: Signal<String>,
    pub content: Signal<String>,
    pub validation_error: Signal<String>,
}

pub fn use_prompt_form(id: String, loaded_prompt: Option<Prompt>) -> PromptFormState {
    let mut prompt_id = use_signal(String::new);
    let mut name = use_signal(String::new);
    let mut content = use_signal(String::new);
    let validation_error = use_signal(String::new);

    // Load prompt data into form fields only once
    let mut is_loaded = use_signal(|| false);
    use_effect(move || {
        if !is_loaded() && !id.is_empty() {
            if let Some(prompt) = &loaded_prompt {
                prompt_id.set(prompt.id.clone());
                name.set(prompt.name.clone());
                content.set(prompt.content.clone());
                is_loaded.set(true);
            }
        }
    });

    PromptFormState {
        prompt_id,
        name,
        content,
        validation_error,
    }
}
