use crate::components::ExecutionStatus;
use chrono::{DateTime, Local};
use dioxus::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct StatusMessage {
    pub message: String,
    pub status: ExecutionStatus,
    pub timestamp: DateTime<Local>,
}

impl StatusMessage {
    pub fn new(message: String, status: ExecutionStatus) -> Self {
        Self {
            message,
            status,
            timestamp: Local::now(),
        }
    }
}

#[component]
pub fn StatusBar(message: Option<StatusMessage>) -> Element {
    rsx! {
        if let Some(status_msg) = message {
            div {
                class: "fixed bottom-0 left-0 right-0 bg-zinc-100 dark:bg-zinc-800 p-3 border-t border-zinc-200 dark:border-zinc-700 flex items-center justify-between text-sm text-zinc-700 dark:text-zinc-300 z-50",
                div { class: "flex items-center gap-2",
                    div {
                        class: "w-2.5 h-2.5 rounded-full bg-blue-500"
                    }
                    span { "{status_msg.message}" }
                }
                span { class: "text-xs text-zinc-500 dark:text-zinc-400", "12:34:56" }
            }
        }
    }
}
