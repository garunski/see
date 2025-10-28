use crate::components::{Badge, BadgeColor, EmptyState, SectionCard};
use dioxus::prelude::*;

#[component]
pub fn TaskDetailsUserInputTab(input_request: Option<s_e_e_core::UserInputRequest>) -> Element {
    if let Some(req) = input_request {
        rsx! {
            SectionCard {
                title: Some("User Input Prompt".to_string()),
                children: rsx! {
                    div { class: "space-y-4",
                        if !req.prompt_text.is_empty() {
                            div { class: "space-y-2",
                                span { class: "text-sm font-medium text-zinc-600 dark:text-zinc-400", "Prompt:" }
                                div { class: "text-sm text-zinc-900 dark:text-zinc-100 bg-white dark:bg-zinc-900 rounded-lg p-3 border border-zinc-200 dark:border-zinc-700",
                                    "{req.prompt_text}"
                                }
                            }
                        }
                        div { class: "space-y-2",
                            span { class: "text-sm font-medium text-zinc-600 dark:text-zinc-400", "Input Type:" }
                            div { class: "text-sm text-zinc-900 dark:text-zinc-100",
                                "{req.input_type}"
                                if !req.required {
                                    span { class: "text-zinc-500 dark:text-zinc-400 ml-2", "(optional)" }
                                } else {
                                    span { class: "text-red-600 dark:text-red-400 ml-2", "(required)" }
                                }
                            }
                        }
                        if let Some(default) = &req.default_value {
                            div { class: "space-y-2",
                                span { class: "text-sm font-medium text-zinc-600 dark:text-zinc-400", "Default Value:" }
                                div { class: "text-sm text-zinc-900 dark:text-zinc-100 bg-white dark:bg-zinc-900 rounded-lg p-3 border border-zinc-200 dark:border-zinc-700 font-mono",
                                    "{default}"
                                }
                            }
                        }
                        div { class: "space-y-2",
                            span { class: "text-sm font-medium text-zinc-600 dark:text-zinc-400", "Status:" }
                            Badge {
                                color: if req.status.to_string() == "pending" {
                                    BadgeColor::Amber
                                } else {
                                    BadgeColor::Emerald
                                },
                                class: None,
                                {format!("{}", req.status)}
                            }
                        }
                    }
                },
                padding: None,
            }
        }
    } else {
        rsx! {
            SectionCard {
                title: Some("User Input Prompt".to_string()),
                children: rsx! {
                    EmptyState {
                        message: "No user input requested for this task".to_string(),
                    }
                },
                padding: None,
            }
        }
    }
}
