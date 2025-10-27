use crate::components::SectionCard;
use crate::layout::router::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;

use crate::pages::prompts::list::components::{PromptList, PromptsListHeader};
use crate::pages::prompts::list::hooks::use_prompts_list;

#[component]
pub fn UserPromptsListPage() -> Element {
    let navigator = use_navigator();

    // Load prompts via hook
    let prompts = match use_prompts_list() {
        Ok(p) => p,
        Err(e) => {
            return rsx! {
                div { class: "space-y-8",
                    PromptsListHeader {}
                    SectionCard {
                        title: Some("Error".to_string()),
                        children: rsx! {
                            div { class: "text-red-600 dark:text-red-400",
                                "Failed to load prompts: {e}"
                            }
                        },
                        padding: None,
                    }
                }
            };
        }
    };

    rsx! {
        div { class: "space-y-8",
            PromptsListHeader {}

            PromptList {
                prompts,
                on_prompt_click: move |prompt_id: String| {
                    navigator.push(Route::UserPromptEditPage { id: prompt_id });
                },
            }
        }
    }
}
