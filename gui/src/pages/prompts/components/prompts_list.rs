use crate::components::{Button, ButtonSize, ButtonVariant, ConfirmDialog};
use crate::router::Route;
use crate::services::prompt::PromptService;
use crate::state::AppStateProvider;
use dioxus::prelude::*;
use dioxus_router::prelude::Link;

#[component]
pub fn PromptsListPage() -> Element {
    let state_provider = use_context::<AppStateProvider>();
    let prompts = use_memo(move || {
        let prompts = state_provider.prompts.read().get_prompts().clone();
        prompts
    });
    let mut show_delete_dialog = use_signal(|| false);
    let mut prompt_to_delete = use_signal(String::new);

    // Load prompts on mount
    let state_provider_clone = state_provider.clone();
    use_effect(move || {
        if state_provider_clone.prompts.read().needs_reload {
            let mut state_provider = state_provider_clone.clone();
            spawn(async move {
                match PromptService::fetch_prompts().await {
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

    let mut delete_prompt = {
        let _state_provider = state_provider.clone();
        let mut show_dialog = show_delete_dialog;
        move |id: String| {
            prompt_to_delete.set(id);
            show_dialog.set(true);
        }
    };

    let confirm_delete = {
        let state_provider = state_provider.clone();
        let mut show_dialog = show_delete_dialog;
        move |_| {
            let id = prompt_to_delete();
            show_dialog.set(false);

            let mut state_provider = state_provider.clone();
            spawn(async move {
                match PromptService::delete_prompt(&id).await {
                    Ok(_) => {
                        state_provider.prompts.write().remove_prompt(id);
                    }
                    Err(e) => {
                        tracing::error!("Failed to delete prompt: {}", e);
                    }
                }
            });
        }
    };

    rsx! {
        div { class: "space-y-8",
            div { class: "flex items-center justify-between",
                div {
                    h1 { class: "text-xl font-bold text-zinc-900 dark:text-white", "Prompts" }
                    p { class: "mt-2 text-zinc-600 dark:text-zinc-400", "Manage your prompt templates" }
                }
                Link {
                    to: Route::PromptEditPageNew {},
                    class: "inline-flex items-center gap-x-1.5 rounded-md bg-blue-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-blue-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600",
                    svg { class: "-ml-0.5 h-5 w-5", view_box: "0 0 20 20", fill: "currentColor",
                        path { d: "M10.75 4.75a.75.75 0 00-1.5 0v4.5h-4.5a.75.75 0 000 1.5h4.5v4.5a.75.75 0 001.5 0v-4.5h4.5a.75.75 0 000-1.5h-4.5v-4.5z" }
                    }
                    "Create Prompt"
                }
            }

            div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 shadow-sm",
                div { class: "px-6 py-4 border-b border-zinc-200 dark:border-zinc-700",
                    h3 { class: "text-base font-semibold text-zinc-900 dark:text-white", "All Prompts" }
                }

                if prompts().is_empty() {
                    div { class: "px-6 py-12 text-center",
                        div { class: "text-zinc-500 dark:text-zinc-400",
                            "No prompts yet. Create your first prompt to get started."
                        }
                    }
                } else {
                    div { class: "overflow-hidden",
                        table { class: "min-w-full divide-y divide-zinc-200 dark:divide-zinc-700",
                            thead { class: "bg-zinc-50 dark:bg-zinc-700",
                                tr {
                                    th { class: "px-6 py-3 text-left text-xs font-medium text-zinc-500 dark:text-zinc-300 uppercase tracking-wider", "ID" }
                                    th { class: "px-6 py-3 text-left text-xs font-medium text-zinc-500 dark:text-zinc-300 uppercase tracking-wider", "Description" }
                                    th { class: "px-6 py-3 text-left text-xs font-medium text-zinc-500 dark:text-zinc-300 uppercase tracking-wider", "Created" }
                                    th { class: "px-6 py-3 text-right text-xs font-medium text-zinc-500 dark:text-zinc-300 uppercase tracking-wider", "Actions" }
                                }
                            }
                            tbody { class: "bg-white dark:bg-zinc-800 divide-y divide-zinc-200 dark:divide-zinc-700",
                                for prompt in prompts().into_iter() {
                                    tr { class: "hover:bg-zinc-50 dark:hover:bg-zinc-700",
                                        td { class: "px-6 py-4 whitespace-nowrap",
                                            div { class: "text-sm font-medium text-zinc-900 dark:text-white",
                                                {prompt.id.clone()}
                                            }
                                        }
                                        td { class: "px-6 py-4",
                                            div { class: "text-sm text-zinc-900 dark:text-white max-w-xs truncate",
                                                {prompt.description.clone()}
                                            }
                                        }
                                        td { class: "px-6 py-4 whitespace-nowrap",
                                            div { class: "text-sm text-zinc-500 dark:text-zinc-400",
                                                {prompt.created_at.clone()}
                                            }
                                        }
                                        td { class: "px-6 py-4 whitespace-nowrap text-right text-sm font-medium",
                                            div { class: "flex items-center justify-end gap-2",
                                                Link {
                                                    to: Route::PromptEditPage { id: prompt.id.clone() },
                                                    class: "text-blue-600 hover:text-blue-900 dark:text-blue-400 dark:hover:text-blue-300",
                                                    "Edit"
                                                }
                                                Button {
                                                    variant: ButtonVariant::Danger,
                                                    size: ButtonSize::Small,
                                                    onclick: move |_| delete_prompt(prompt.id.clone()),
                                                    "Delete"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        ConfirmDialog {
            show: show_delete_dialog(),
            title: "Delete Prompt?".to_string(),
            message: format!("Are you sure you want to delete the prompt '{}'? This action cannot be undone.", prompt_to_delete()),
            confirm_text: "Delete".to_string(),
            cancel_text: "Cancel".to_string(),
            on_confirm: confirm_delete,
            on_cancel: move |_| show_delete_dialog.set(false)
        }
    }
}
