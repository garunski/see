use crate::components::{EmptyState, List, ListItemWithLink, PageHeader, SectionCard};
use crate::hooks::use_prompts;
use crate::icons::Icon;
use crate::layout::router::Route;
use crate::services::prompt::UserPromptService;
use crate::state::AppStateProvider;
use dioxus::prelude::*;
use dioxus_router::prelude::Link;

#[component]
pub fn UserPromptsListPage() -> Element {
    let state_provider = use_context::<AppStateProvider>();
    let prompts = use_prompts();

    // Load prompts on mount
    let state_provider_clone = state_provider.clone();
    use_effect(move || {
        if state_provider_clone.prompts.read().needs_reload {
            let mut state_provider = state_provider_clone.clone();
            spawn(async move {
                match UserPromptService::fetch_prompts().await {
                    Ok(loaded_prompts) => {
                        state_provider.prompts.write().load_prompts(loaded_prompts);
                    }
                    Err(e) => {
                        tracing::error!("Failed to load prompts: {}", e);
                    }
                }
            });
        }
    });

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
                            class: Some("-ml-0.5".to_string()),
                            size: Some("h-5 w-5".to_string()),
                            variant: Some("outline".to_string()),
                        }
                        "Create UserPrompt"
                    }
                }),
            }

            if prompts.read().is_empty() {
                SectionCard {
                    title: Some("All UserPrompts".to_string()),
                    children: rsx! {
                        EmptyState {
                            message: "No prompts yet. Create your first prompt to get started.".to_string(),
                        }
                    },
                    padding: None,
                }
            } else {
                List {
                    for prompt in prompts.read().iter() {
                        ListItemWithLink {
                            icon_name: "prompts".to_string(),
                            icon_variant: Some("outline".to_string()),
                            title: prompt.id.clone(),
                            subtitle: Some(rsx! {
                                div { class: "flex flex-col gap-1",
                                    div { class: "text-sm text-gray-900 dark:text-white max-w-xs truncate",
                                        {prompt.description.clone()}
                                    }
                                    div { class: "text-xs text-gray-500 dark:text-gray-400",
                                        "Template: {prompt.template.len()} characters"
                                    }
                                }
                            }),
                            right_content: None,
                            link_to: rsx! {
                                Link {
                                    to: Route::UserPromptEditPage { id: prompt.id.clone() },
                                    span { class: "absolute inset-x-0 -top-px bottom-0" }
                                    {prompt.id.clone()}
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}
