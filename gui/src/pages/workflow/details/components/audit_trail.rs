use crate::components::ui::Card;
use dioxus::prelude::*;
use see_core::AuditEntry;

#[derive(Props, PartialEq, Clone)]
pub struct AuditTrailProps {
    pub entries: Vec<AuditEntry>,
}

#[component]
pub fn AuditTrail(props: AuditTrailProps) -> Element {
    if props.entries.is_empty() {
        return rsx! { div {} };
    }

    rsx! {
        Card {
            h3 {
                class: "text-base font-semibold text-zinc-950 dark:text-white mb-4",
                "Current Task Audit Trail"
            }
            div {
                class: "space-y-3",
                for entry in props.entries.iter() {
                    div {
                        class: "flex items-start gap-3 p-3 bg-zinc-50 dark:bg-zinc-800 rounded-lg",
                        div {
                            class: "w-2 h-2 bg-blue-500 rounded-full mt-2 flex-shrink-0"
                        }
                        div {
                            class: "flex-1 min-w-0",
                            div {
                                class: "text-sm text-zinc-950 dark:text-white font-medium",
                                "Task: {entry.task_id}"
                            }
                            div {
                                class: "text-xs text-zinc-500 dark:text-zinc-400 mt-1",
                                "{entry.timestamp} - Status: {entry.status} - Changes: {entry.changes_count}"
                            }
                        }
                    }
                }
            }
        }
    }
}
