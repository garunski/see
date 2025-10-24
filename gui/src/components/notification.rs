use crate::icons::Icon;
use dioxus::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationType {
    Success,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NotificationData {
    pub r#type: NotificationType,
    pub title: String,
    pub message: String,
    pub show: bool,
}

#[component]
pub fn Notification(notification: Signal<NotificationData>, on_close: EventHandler<()>) -> Element {
    let notification_data = notification();

    if !notification_data.show {
        return rsx! { div {} };
    }

    let icon_name = match notification_data.r#type {
        NotificationType::Success => "check_circle",
        NotificationType::Error => "exclamation_circle",
    };

    let icon_color = match notification_data.r#type {
        NotificationType::Success => "text-green-400",
        NotificationType::Error => "text-red-400",
    };

    rsx! {
        div {
            "aria-live": "assertive",
            class: "pointer-events-none fixed inset-0 flex items-end px-4 py-6 sm:items-start sm:p-6 z-50",
            div {
                class: "flex w-full flex-col items-center space-y-4 sm:items-end",
                div {
                    class: "pointer-events-auto w-full max-w-sm translate-y-0 transform rounded-lg bg-white opacity-100 shadow-lg outline outline-1 outline-black/5 transition duration-300 ease-out sm:translate-x-0 dark:bg-gray-800 dark:-outline-offset-1 dark:outline-white/10",
                    div { class: "p-4",
                        div { class: "flex items-start",
                            div { class: "shrink-0",
                                Icon {
                                    name: icon_name.to_string(),
                                    class: Some(format!("size-6 {}", icon_color)),
                                    size: None,
                                    variant: Some("outline".to_string()),
                                }
                            }
                            div { class: "ml-3 w-0 flex-1 pt-0.5",
                                p { class: "text-sm font-medium text-gray-900 dark:text-white",
                                    {notification_data.title}
                                }
                                p { class: "mt-1 text-sm text-gray-500 dark:text-gray-400",
                                    {notification_data.message}
                                }
                            }
                            div { class: "ml-4 flex shrink-0",
                                button {
                                    r#type: "button",
                                    class: "inline-flex rounded-md text-gray-400 hover:text-gray-500 focus:outline focus:outline-2 focus:outline-offset-2 focus:outline-indigo-600 dark:hover:text-white dark:focus:outline-indigo-500",
                                    onclick: move |_| on_close.call(()),
                                    span { class: "sr-only", "Close" }
                                    Icon {
                                        name: "x".to_string(),
                                        class: Some("size-5".to_string()),
                                        size: None,
                                        variant: Some("outline".to_string()),
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
