use crate::components::{SectionCard, TextInput, TextareaInput, ValidationMessage};
use dioxus::prelude::*;

#[component]
pub fn PromptFormFields(
    prompt_id: Signal<String>,
    name: Signal<String>,
    content: Signal<String>,
    validation_error: Signal<String>,
    is_new: bool,
) -> Element {
    rsx! {
        SectionCard {
            title: None,
            children: rsx! {
                div { class: "space-y-6",
                    TextInput {
                        label: "Prompt ID".to_string(),
                        value: prompt_id,
                        oninput: move |value| prompt_id.set(value),
                        placeholder: Some("e.g., generate-rust-code".to_string()),
                        help_text: Some("Human-readable identifier used to reference this prompt in workflows".to_string()),
                        required: Some(true),
                        disabled: Some(!is_new),
                    }

                    TextInput {
                        label: "Name".to_string(),
                        value: name,
                        oninput: move |value| name.set(value),
                        placeholder: Some("Display name for this prompt".to_string()),
                        help_text: Some("The name shown in lists and references".to_string()),
                        required: Some(true),
                        disabled: None,
                    }

                    TextareaInput {
                        label: "Prompt Content".to_string(),
                        value: content,
                        oninput: move |value| content.set(value),
                        placeholder: Some("Enter the prompt template content...".to_string()),
                        help_text: Some("The actual prompt text that will be sent to the AI model".to_string()),
                        rows: Some(15),
                        disabled: None,
                    }

                    ValidationMessage {
                        message: validation_error,
                    }
                }
            },
            padding: None,
        }
    }
}
