use crate::components::{IconButton, IconButtonSize, IconButtonVariant};
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;

#[component]
pub fn TaskDetailsHeader(task_name: String, task_id: String) -> Element {
    let navigator = use_navigator();

    rsx! {
        div { class: "space-y-4",
            div { class: "flex items-center gap-4",
                IconButton {
                    variant: IconButtonVariant::Ghost,
                    size: IconButtonSize::Medium,
                    icon: Some("arrow_left".to_string()),
                    onclick: EventHandler::new(move |_| {
                        navigator.go_back();
                    }),
                    class: None,
                    "Back"
                }
            }

            div {
                h1 { class: "text-2xl font-bold text-zinc-950 dark:text-white",
                    {task_name}
                }
                p { class: "text-sm text-zinc-500 dark:text-zinc-400 mt-1", "ID: {task_id}" }
            }
        }
    }
}
