use crate::components::{Button, ButtonSize, ButtonVariant};
use crate::router::Route;
use crate::services::prompt::PromptService;
use crate::state::AppStateProvider;
use dioxus::prelude::*;
use dioxus_router::prelude::Link;
use see_core::persistence::models::Prompt;

#[component]
pub fn PromptEditPage(id: String) -> Element {
    let state_provider = use_context::<AppStateProvider>();

    let is_new = id.is_empty();

    let mut prompt_id = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut content = use_signal(String::new);
    let mut validation_error = use_signal(String::new);
    let mut is_saving = use_signal(|| false);

    // Load existing prompt data if editing
    let prompt_id_for_effect = id.clone();
    use_effect(move || {
        if !is_new && !prompt_id_for_effect.is_empty() {
            if let Some(prompt) = state_provider
                .prompts
                .read()
                .get_prompt(prompt_id_for_effect.clone())
            {
                prompt_id.set(prompt.id.clone());
                description.set(prompt.description.clone());
                content.set(prompt.content.clone());
            }
        }
    });

    rsx! {
        div { class: "space-y-8",
            div { class: "flex items-center justify-between",
                div { class: "flex items-center gap-4",
                    Link {
                        to: Route::PromptsListPage {},
                        class: "inline-flex items-center gap-x-1.5 rounded-md bg-zinc-100 dark:bg-zinc-800 px-3 py-2 text-sm font-semibold text-zinc-900 dark:text-zinc-100 shadow-sm hover:bg-zinc-200 dark:hover:bg-zinc-700",
                        svg { class: "-ml-0.5 h-4 w-4", view_box: "0 0 20 20", fill: "currentColor",
                            path { fill_rule: "evenodd", d: "M17 10a.75.75 0 01-.75.75H5.612l2.158 1.96a.75.75 0 11-1.04 1.08l-3.5-3.25a.75.75 0 010-1.08l3.5-3.25a.75.75 0 111.04 1.08L5.612 9.25H16.25A.75.75 0 0117 10z", clip_rule: "evenodd" }
                        }
                        "Back"
                    }
                    div {
                        h1 { class: "text-xl font-bold text-zinc-900 dark:text-white",
                            if is_new { "Create Prompt" } else { "Edit Prompt" }
                        }
                        p { class: "mt-2 text-zinc-600 dark:text-zinc-400",
                            if is_new { "Create a new prompt template" } else { "Edit prompt template" }
                        }
                    }
                }
                Button {
                    variant: ButtonVariant::Primary,
                    size: ButtonSize::Medium,
                    disabled: Some(is_saving()),
                    loading: Some(is_saving()),
                    onclick: move |_| {
                        // Validation
                        if prompt_id().trim().is_empty() {
                            validation_error.set("ID is required".to_string());
                            return;
                        }
                        if content().trim().is_empty() {
                            validation_error.set("Content is required".to_string());
                            return;
                        }

                        // Check for duplicate ID if creating new
                        if is_new {
                            let prompts_guard = state_provider.prompts.read();
                            let existing_prompt = prompts_guard.get_prompt(prompt_id().trim().to_string());
                            if existing_prompt.is_some() {
                                validation_error.set("A prompt with this ID already exists".to_string());
                                return;
                            }
                        }

                        validation_error.set(String::new());
                        is_saving.set(true);

                        let prompt = Prompt {
                            id: prompt_id().trim().to_string(),
                            content: content().trim().to_string(),
                            description: description().trim().to_string(),
                            created_at: if is_new {
                                chrono::Utc::now().to_rfc3339()
                            } else {
                                // Keep existing created_at for updates
                                state_provider
                                    .prompts
                                    .read()
                                    .get_prompt(id.clone())
                                    .map(|p| p.created_at.clone())
                                    .unwrap_or_else(|| chrono::Utc::now().to_rfc3339())
                            },
                        };

                        let mut state_provider = state_provider.clone();
                        let id_for_save = id.clone();
                        spawn(async move {
                            let result = if is_new {
                                PromptService::create_prompt(prompt.clone()).await
                            } else {
                                PromptService::update_prompt(prompt.clone()).await
                            };

                            match result {
                                Ok(_) => {
                                    if is_new {
                                        state_provider.prompts.write().add_prompt(prompt);
                                    } else {
                                        state_provider.prompts.write().update_prompt(
                                            id_for_save.clone(),
                                            prompt,
                                        );
                                    }
                                }
                                Err(e) => {
                                    validation_error.set(format!("Failed to save prompt: {}", e));
                                }
                            }
                            is_saving.set(false);
                        });
                    },
                    if is_saving() { "Saving..." } else { "Save" }
                }
            }

            div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 p-8 shadow-sm",
                div { class: "space-y-6",
                    div {
                        label { class: "block text-sm font-medium text-zinc-900 dark:text-white mb-2",
                            "Prompt ID"
                        }
                        input {
                            r#type: "text",
                            value: "{prompt_id()}",
                            oninput: move |evt| prompt_id.set(evt.value()),
                            placeholder: "e.g., generate-rust-code",
                            class: "block w-full rounded-md border-0 py-1.5 text-zinc-900 dark:text-white shadow-sm ring-1 ring-inset ring-zinc-300 dark:ring-zinc-600 placeholder:text-zinc-400 dark:placeholder:text-zinc-500 focus:ring-2 focus:ring-inset focus:ring-blue-600 dark:bg-zinc-700 sm:text-sm sm:leading-6"
                        }
                        p { class: "mt-1 text-xs text-zinc-500 dark:text-zinc-400",
                            "Human-readable identifier used to reference this prompt in workflows"
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-zinc-900 dark:text-white mb-2",
                            "Description"
                        }
                        input {
                            r#type: "text",
                            value: "{description()}",
                            oninput: move |evt| description.set(evt.value()),
                            placeholder: "Brief description of what this prompt does",
                            class: "block w-full rounded-md border-0 py-1.5 text-zinc-900 dark:text-white shadow-sm ring-1 ring-inset ring-zinc-300 dark:ring-zinc-600 placeholder:text-zinc-400 dark:placeholder:text-zinc-500 focus:ring-2 focus:ring-inset focus:ring-blue-600 dark:bg-zinc-700 sm:text-sm sm:leading-6"
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-zinc-900 dark:text-white mb-2",
                            "Prompt Content"
                        }
                        textarea {
                            value: "{content()}",
                            oninput: move |evt| content.set(evt.value()),
                            placeholder: "Enter the prompt template content...",
                            rows: 15,
                            class: "block w-full rounded-md border-0 py-1.5 text-zinc-900 dark:text-white shadow-sm ring-1 ring-inset ring-zinc-300 dark:ring-zinc-600 placeholder:text-zinc-400 dark:placeholder:text-zinc-500 focus:ring-2 focus:ring-inset focus:ring-blue-600 dark:bg-zinc-700 sm:text-sm sm:leading-6 font-mono"
                        }
                        p { class: "mt-1 text-xs text-zinc-500 dark:text-zinc-400",
                            "The actual prompt text that will be sent to the AI model"
                        }
                    }

                    if !validation_error().is_empty() {
                        div { class: "mt-2 text-sm text-red-600 dark:text-red-400",
                            {validation_error()}
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn PromptEditPageNew() -> Element {
    rsx! {
        PromptEditPage { id: "".to_string() }
    }
}
