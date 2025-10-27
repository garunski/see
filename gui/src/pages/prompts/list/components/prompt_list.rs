use crate::components::layout::{List, ListItem};
use crate::components::{EmptyState, SectionCard};
use dioxus::prelude::*;
use s_e_e_core::Prompt;

#[component]
pub fn PromptList(prompts: Vec<Prompt>, on_prompt_click: EventHandler<String>) -> Element {
    rsx! {
        if prompts.is_empty() {
            SectionCard {
                title: Some("Prompts".to_string()),
                children: rsx! {
                    EmptyState {
                        message: "No prompts yet. Create your first prompt to get started.".to_string(),
                    }
                },
                padding: None,
            }
        } else {
            SectionCard {
                title: Some("Prompts".to_string()),
                children: rsx! {
                    List {
                        for prompt in prompts {
                            {let prompt_id = prompt.id.clone();
                            rsx! {
                                ListItem {
                                    icon_name: "document".to_string(),
                                    icon_variant: Some("outline".to_string()),
                                    title: rsx! {
                                        {prompt.name.clone()}
                                    },
                                    subtitle: None,
                                    right_content: None,
                                    onclick: move |_| {
                                        on_prompt_click.call(prompt_id.clone());
                                    },
                                }
                            }}
                        }
                    }
                },
                padding: None,
            }
        }
    }
}
