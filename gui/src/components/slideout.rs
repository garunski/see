use crate::components::{IconButton, IconButtonSize, IconButtonVariant};
use dioxus::prelude::*;

#[component]
pub fn Slideout(
    is_open: bool,
    backdrop_class: String,
    on_close: EventHandler<()>,
    title: String,
    subtitle: Option<String>,
    children: Element,
    footer: Option<Element>,
    show_close_button: Option<bool>,
) -> Element {
    if !is_open {
        return rsx! { div {} };
    }

    rsx! {
        // Backdrop
        div {
            class: format!("fixed inset-0 z-50 cursor-pointer {}", backdrop_class),
            onclick: move |_| on_close.call(()),

            // Panel
            div {
                class: "fixed inset-y-0 right-0 z-50 w-3/4 transform transition-transform duration-500 ease-in-out sm:duration-700",
                onclick: move |e| e.stop_propagation(),

                div {
                    class: "flex h-full flex-col divide-y divide-gray-200 bg-white shadow-xl dark:divide-white/10 dark:bg-gray-800 dark:after:absolute dark:after:inset-y-0 dark:after:left-0 dark:after:w-px dark:after:bg-white/10",

                    // Header
                    div {
                        class: "flex items-center justify-between px-4 py-4 sm:px-6",

                        // Title and subtitle
                        div {
                            h2 {
                                class: "text-lg font-semibold text-gray-900 dark:text-white",
                                "{title}"
                            }
                            if let Some(subtitle) = subtitle {
                                div {
                                    class: "text-sm text-gray-500 dark:text-gray-400 mt-1",
                                    "{subtitle}"
                                }
                            }
                        }

                        // Optional close button
                        if show_close_button.unwrap_or(true) {
                            IconButton {
                                variant: IconButtonVariant::Ghost,
                                size: IconButtonSize::Small,
                                onclick: Some(on_close),
                                class: Some("text-gray-400 hover:text-gray-500 dark:hover:text-white".to_string()),
                                icon: Some("x".to_string()),
                                icon_variant: "outline".to_string(),
                                ""
                            }
                        }
                    }

                    // Content
                    div {
                        class: "flex-1 overflow-y-auto py-6",
                        div {
                            class: "px-4 sm:px-6",
                            {children}
                        }
                    }

                    // Footer (optional)
                    if let Some(footer_content) = footer {
                        div {
                            class: "flex shrink-0 items-center justify-between px-4 py-4",
                            {footer_content}
                        }
                    }
                }
            }
        }
    }
}
