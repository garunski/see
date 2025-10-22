use crate::components::ui::{Card, Icon, IconName};
use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct ErrorListProps {
    pub errors: Vec<String>,
}

#[component]
pub fn ErrorList(props: ErrorListProps) -> Element {
    if props.errors.is_empty() {
        return rsx! { div {} };
    }

    rsx! {
        Card {
            h3 {
                class: "text-base font-semibold text-red-800 dark:text-red-200 mb-4",
                "Errors"
            }
            div {
                class: "space-y-3",
                for error in props.errors.iter() {
                    div {
                        class: "p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg",
                        div {
                            class: "flex items-start gap-3",
                            Icon {
                                name: IconName::Error,
                                class: Some("w-5 h-5 text-red-600 dark:text-red-400 mt-0.5 flex-shrink-0".to_string())
                            }
                            div {
                                class: "text-sm text-red-800 dark:text-red-200",
                                {error.clone()}
                            }
                        }
                    }
                }
            }
        }
    }
}
