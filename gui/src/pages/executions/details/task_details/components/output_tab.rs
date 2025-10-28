use crate::components::{EmptyState, SectionCard};
use dioxus::prelude::*;
use s_e_e_core::TaskExecution;

#[component]
pub fn TaskDetailsOutputTab(task: TaskExecution) -> Element {
    rsx! {
        SectionCard {
            title: Some("Output".to_string()),
            children: rsx! {
                if let Some(output) = task.output.as_ref() {
                    div { class: "bg-white dark:bg-zinc-900 rounded-lg p-4 border border-zinc-200 dark:border-zinc-700",
                        pre { class: "text-sm text-zinc-900 dark:text-zinc-100 whitespace-pre-wrap font-mono overflow-x-auto",
                            "{output}"
                        }
                    }
                } else {
                    EmptyState {
                        message: "No output available".to_string(),
                    }
                }
            },
            padding: None,
        }
    }
}
