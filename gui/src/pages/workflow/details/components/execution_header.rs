use crate::components::ui::{Button, ButtonVariant, Icon, IconName};
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;

#[component]
pub fn ExecutionHeader() -> Element {
    let navigator = use_navigator();

    rsx! {
        div {
            class: "flex items-center gap-4",
            Button {
                variant: ButtonVariant::Ghost,
                onclick: move |_| navigator.go_back(),
                Icon {
                    name: IconName::BackArrow,
                    class: None
                }
                span { "Back to History" }
            }
            h1 {
                class: "text-lg font-semibold text-zinc-950 dark:text-white",
                "Workflow Execution Details"
            }
        }
    }
}
