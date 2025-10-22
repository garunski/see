use crate::components::ui::Card;
use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct TaskLogsProps {
    pub logs: Vec<String>,
}

#[component]
pub fn TaskLogs(props: TaskLogsProps) -> Element {
    if props.logs.is_empty() {
        return rsx! { div {} };
    }

    rsx! {
        Card {
            h3 {
                class: "text-base font-semibold text-zinc-950 dark:text-white mb-4",
                "Current Task Logs"
            }
            div {
                class: "space-y-2 max-h-64 overflow-y-auto",
                for log in props.logs.iter() {
                    div {
                        class: "text-sm text-zinc-700 dark:text-zinc-300 font-mono bg-zinc-100 dark:bg-zinc-800 p-2 rounded",
                        {log.clone()}
                    }
                }
            }
        }
    }
}
