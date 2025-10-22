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
    if let Some(status_msg) = message {
        let indicator_class = format!(
            "w-2.5 h-2.5 rounded-full {}",
            match status_msg.status {
                ExecutionStatus::Running => "bg-blue-500 animate-pulse",
                ExecutionStatus::Complete => "bg-emerald-500",
                ExecutionStatus::Failed => "bg-red-500",
                ExecutionStatus::Idle => "bg-zinc-400",
            }
        );
        let time_str = status_msg.timestamp.format("%H:%M:%S").to_string();

        rsx! {
            div {
                class: "fixed bottom-0 left-0 right-0 bg-zinc-100 dark:bg-zinc-800 p-3 border-t border-zinc-200 dark:border-zinc-700 flex items-center justify-between text-sm text-zinc-700 dark:text-zinc-300 z-50",
                div { class: "flex items-center gap-2",
                    div { class: "{indicator_class}" }
                    span { "{status_msg.message}" }
                }
                span { class: "text-xs text-zinc-500 dark:text-zinc-400", "{time_str}" }
            }
        }
    } else {
        rsx! {}
    }
}
