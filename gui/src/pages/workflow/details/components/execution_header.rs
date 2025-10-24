use crate::components::{Button, ButtonSize, ButtonVariant};
use crate::icons::Icon;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;

#[component]
pub fn ExecutionHeader() -> Element {
    let navigator = use_navigator();

    rsx! {
        div { class: "flex items-center gap-4",
            Button {
                variant: ButtonVariant::Ghost,
                size: ButtonSize::Medium,
                onclick: move |_| navigator.go_back(),
                class: "flex items-center gap-2".to_string(),
                Icon {
                    name: "arrow_left".to_string(),
                    class: Some("w-5 h-5".to_string()),
                    size: None,
                    variant: Some("outline".to_string()),
                }
                span { "Back to History" }
            }
            h1 { class: "text-lg font-semibold text-zinc-950 dark:text-white", "Workflow Execution Details" }
        }
    }
}
