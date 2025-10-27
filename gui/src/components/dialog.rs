use super::{IconButton, IconButtonSize, IconButtonVariant};
use dioxus::prelude::*;

#[component]
pub fn ConfirmDialog(
    show: bool,
    title: String,
    message: String,
    confirm_text: String,
    cancel_text: String,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    if !show {
        return rsx! { div {} };
    }

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center",
            div {
                class: "absolute inset-0 bg-black/50",
                onclick: move |_| on_cancel.call(())
            }
            div {
                class: "relative bg-white dark:bg-zinc-800 rounded-xl shadow-xl p-6 max-w-md w-full mx-4 z-10",
                h3 { class: "text-lg font-semibold text-zinc-900 dark:text-white mb-2", "{title}" }
                p { class: "text-zinc-600 dark:text-zinc-400 mb-6", "{message}" }
                div { class: "flex gap-3 justify-end",
                    IconButton {
                        variant: IconButtonVariant::Secondary,
                        size: IconButtonSize::Medium,
                        onclick: move |_| on_cancel.call(()),
                        icon: Some("x".to_string()),
                        icon_variant: "outline".to_string(),
                        "{cancel_text}"
                    }
                    IconButton {
                        variant: IconButtonVariant::Danger,
                        size: IconButtonSize::Medium,
                        onclick: move |_| on_confirm.call(()),
                        icon: Some("check_circle".to_string()),
                        icon_variant: "outline".to_string(),
                        "{confirm_text}"
                    }
                }
            }
        }
    }
}
