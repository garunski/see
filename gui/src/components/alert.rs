use crate::icons::Icon;
use dioxus::prelude::*;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum AlertType {
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Props, PartialEq, Clone)]
pub struct AlertProps {
    pub alert_type: AlertType,
    pub title: Option<String>,
    pub message: String,
    #[props(default = None)]
    pub dismissible: Option<bool>,
    #[props(default = None)]
    pub on_dismiss: Option<EventHandler<()>>,
    #[props(default = None)]
    pub actions: Option<Element>,
}

#[component]
pub fn Alert(props: AlertProps) -> Element {
    let AlertProps {
        alert_type,
        title,
        message,
        dismissible,
        on_dismiss,
        actions,
    } = props;

    let (bg_color, border_color, text_color, icon_color, icon_name) = match alert_type {
        AlertType::Info => (
            "bg-purple-50 dark:bg-purple-900/20",
            "border-purple-200 dark:border-purple-800",
            "text-purple-700 dark:text-purple-300",
            "text-purple-600 dark:text-purple-400",
            "exclamation_circle",
        ),
        AlertType::Warning => (
            "bg-amber-50 dark:bg-amber-900/20",
            "border-amber-200 dark:border-amber-800",
            "text-amber-800 dark:text-amber-200",
            "text-amber-600 dark:text-amber-400",
            "exclamation_circle",
        ),
        AlertType::Error => (
            "bg-red-50 dark:bg-red-900/20",
            "border-red-200 dark:border-red-800",
            "text-red-800 dark:text-red-200",
            "text-red-600 dark:text-red-400",
            "exclamation_circle",
        ),
        AlertType::Success => (
            "bg-green-50 dark:bg-green-900/20",
            "border-green-200 dark:border-green-800",
            "text-green-800 dark:text-green-200",
            "text-green-600 dark:text-green-400",
            "check_circle",
        ),
    };

    rsx! {
        div {
            class: format!("rounded-md {} border {} p-4 mb-6", bg_color, border_color),
            div { class: "flex items-center justify-between",
                div { class: "flex items-center gap-3",
                    Icon {
                        name: icon_name.to_string(),
                        class: Some(format!("w-5 h-5 {} flex-shrink-0", icon_color)),
                        size: None,
                        variant: Some("outline".to_string()),
                    }
                    div {
                        if let Some(ref alert_title) = title {
                            h3 {
                                class: format!("text-sm font-medium {}", text_color),
                                "{alert_title}"
                            }
                        }
                        p {
                            class: if title.is_some() {
                                format!("text-sm {} mt-1", text_color)
                            } else {
                                format!("text-sm {}", text_color)
                            },
                            "{message}"
                        }
                    }
                }
                div { class: "flex items-center gap-3",
                    if let Some(alert_actions) = actions {
                        {alert_actions}
                    }
                    if dismissible.unwrap_or(false) {
                        button {
                            r#type: "button",
                            class: format!("inline-flex rounded-md {} hover:opacity-75 focus:outline focus:outline-2 focus:outline-offset-2", text_color),
                            onclick: move |_| {
                                if let Some(handler) = on_dismiss {
                                    handler.call(());
                                }
                            },
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
