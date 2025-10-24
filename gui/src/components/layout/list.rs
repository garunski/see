use crate::icons::Icon;
use dioxus::prelude::*;

#[component]
pub fn List(children: Element) -> Element {
    rsx! {
        ul {
            role: "list",
            class: "divide-y divide-gray-100 overflow-hidden bg-white shadow-sm outline outline-1 outline-gray-900/5 sm:rounded-xl dark:divide-white/5 dark:bg-gray-800/50 dark:shadow-none dark:outline-white/10 dark:sm:-outline-offset-1",
            {children}
        }
    }
}

#[component]
pub fn ListItem(onclick: Option<EventHandler<()>>, children: Element) -> Element {
    rsx! {
        li {
            class: "relative flex justify-between gap-x-6 px-4 py-5 hover:bg-gray-50 sm:px-6 dark:hover:bg-white/[0.025]",
            onclick: move |_| {
                if let Some(handler) = onclick {
                    handler.call(());
                }
            },
            {children}
        }
    }
}

#[component]
pub fn ListItemWithIcon(
    icon_name: String,
    icon_variant: Option<String>,
    title: String,
    subtitle: Option<Element>,
    right_content: Option<Element>,
    onclick: Option<EventHandler<()>>,
) -> Element {
    rsx! {
        li {
            class: "relative flex justify-between gap-x-6 px-4 py-5 hover:bg-gray-50 sm:px-6 dark:hover:bg-white/[0.025]",
            onclick: move |_| {
                if let Some(handler) = onclick {
                    handler.call(());
                }
            },
            div { class: "flex min-w-0 gap-x-4",
                div { class: "size-12 flex-none rounded-full bg-gray-50 dark:bg-gray-800 dark:outline dark:outline-1 dark:-outline-offset-1 dark:outline-white/10 flex items-center justify-center",
                    Icon {
                        name: icon_name,
                        class: Some("size-6 text-gray-400 dark:text-gray-500".to_string()),
                        size: None,
                        variant: icon_variant,
                    }
                }
                div { class: "min-w-0 flex-auto",
                    p { class: "text-sm/6 font-semibold text-gray-900 dark:text-white",
                        if let Some(handler) = onclick {
                            span {
                                class: "absolute inset-x-0 -top-px bottom-0",
                                onclick: move |_| handler.call(())
                            }
                        }
                        {title}
                    }
                    if let Some(subtitle) = subtitle {
                        p { class: "mt-1 flex text-xs/5 text-gray-500 dark:text-gray-400",
                            {subtitle}
                        }
                    }
                }
            }
            div { class: "flex shrink-0 items-center gap-x-4",
                if let Some(content) = right_content {
                    div { class: "hidden sm:flex sm:flex-col sm:items-end",
                        {content}
                    }
                }
                Icon {
                    name: "chevron_right".to_string(),
                    class: Some("size-5 flex-none text-gray-400 dark:text-gray-500".to_string()),
                    size: None,
                    variant: Some("outline".to_string()),
                }
            }
        }
    }
}

#[component]
pub fn ListItemWithLink(
    icon_name: String,
    icon_variant: Option<String>,
    title: String,
    subtitle: Option<Element>,
    right_content: Option<Element>,
    link_to: Element,
) -> Element {
    rsx! {
        li {
            class: "relative flex justify-between gap-x-6 px-4 py-5 hover:bg-gray-50 sm:px-6 dark:hover:bg-white/[0.025]",
            div { class: "flex min-w-0 gap-x-4",
                div { class: "size-12 flex-none rounded-full bg-gray-50 dark:bg-gray-800 dark:outline dark:outline-1 dark:-outline-offset-1 dark:outline-white/10 flex items-center justify-center",
                    Icon {
                        name: icon_name,
                        class: Some("size-6 text-gray-400 dark:text-gray-500".to_string()),
                        size: None,
                        variant: icon_variant,
                    }
                }
                div { class: "min-w-0 flex-auto",
                    p { class: "text-sm/6 font-semibold text-gray-900 dark:text-white",
                        {link_to}
                    }
                    if let Some(subtitle) = subtitle {
                        p { class: "mt-1 flex text-xs/5 text-gray-500 dark:text-gray-400",
                            {subtitle}
                        }
                    }
                }
            }
            div { class: "flex shrink-0 items-center gap-x-4",
                if let Some(content) = right_content {
                    div { class: "hidden sm:flex sm:flex-col sm:items-end",
                        {content}
                    }
                }
                Icon {
                    name: "chevron_right".to_string(),
                    class: Some("size-5 flex-none text-gray-400 dark:text-gray-500".to_string()),
                    size: None,
                    variant: Some("outline".to_string()),
                }
            }
        }
    }
}
