use crate::components::layout::{List, ListItem};
use crate::components::{EmptyState, PageHeader, SectionCard};
use crate::icons::Icon;
use crate::layout::router::Route;
use crate::queries::GetPrompts;
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use dioxus_router::prelude::{use_navigator, Link};

#[component]
pub fn UserPromptsListPage() -> Element {
    let query_result = use_query(Query::new((), GetPrompts)).suspend()?;
    let navigator = use_navigator();

    // Extract the Vec<Prompt> from the query result
    let prompts = match query_result {
        Ok(value) => value,
        Err(e) => {
            tracing::error!("Failed to load prompts: {}", e);
            return rsx! {
                div { class: "space-y-8",
                    PageHeader {
                        title: "Prompts".to_string(),
                        description: "Manage your prompt templates".to_string(),
                    }
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
            PageHeader {
                title: "Prompts".to_string(),
                description: "Manage your prompt templates".to_string(),
                actions: Some(rsx! {
                    Link {
                        to: Route::UserPromptEditPageNew {},
                        class: "inline-flex items-center gap-x-1.5 rounded-md bg-blue-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-blue-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600",
                        Icon {
                            name: "plus".to_string(),
                            class: Some("-ml-0.5 h-5 w-5".to_string()),
                            size: None,
                            variant: Some("outline".to_string()),
                        }
                        "Create prompt"
                    }
                }),
            }

            // Prompts Section
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
                                        subtitle: Some(rsx! {
                                            if let Some(desc) = &prompt.description {
                                                if desc.is_empty() {
                                                    span { class: "text-zinc-500 dark:text-zinc-400 italic",
                                                        "No description"
                                                    }
                                                } else {
                                                    span {
                                                        {desc.clone()}
                                                    }
                                                }
                                            } else {
                                                span { class: "text-zinc-500 dark:text-zinc-400 italic",
                                                    "No description"
                                                }
                                            }
                                        }),
                                        right_content: Some(rsx! {
                                            span { class: "text-xs text-zinc-500 dark:text-zinc-400",
                                                {format!("{} variables", prompt.variables.len())}
                                            }
                                        }),
                                        onclick: move |_| {
                                            navigator.push(Route::UserPromptEditPage { id: prompt_id.clone() });
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
}
