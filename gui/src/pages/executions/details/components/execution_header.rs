use crate::components::{IconButton, IconButtonSize, IconButtonVariant};
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;

#[component]
pub fn ExecutionHeader() -> Element {
    let navigator = use_navigator();

    rsx! {
        div { class: "flex items-center gap-4",
            IconButton {
                variant: IconButtonVariant::Ghost,
                size: IconButtonSize::Medium,
                onclick: move |_| navigator.go_back(),
                class: Some("flex items-center gap-2".to_string()),
                icon: Some("arrow_left".to_string()),
                icon_variant: "outline".to_string(),
                "Back"
            }
            h1 { class: "text-lg font-semibold text-zinc-950 dark:text-white", "Workflow Execution Details" }
        }
    }
}
