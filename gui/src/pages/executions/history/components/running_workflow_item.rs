use crate::components::layout::ListItem;
use crate::components::{Badge, BadgeColor};
use crate::icons::Icon;
use crate::layout::router::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use s_e_e_core::WorkflowMetadata;

#[component]
pub fn RunningWorkflowItem(workflow: WorkflowMetadata) -> Element {
    let navigator = use_navigator();

    rsx! {
        ListItem {
            icon_name: "play".to_string(),
            icon_variant: Some("outline".to_string()),
            title: rsx! {
                {workflow.workflow_name.clone()}
            },
            subtitle: Some(rsx! {
                div { class: "flex flex-col gap-1",
                    div { class: "text-sm text-gray-500 dark:text-gray-400",
                        "Started: {workflow.start_timestamp}"
                    }
                    div { class: "text-xs text-gray-500 dark:text-gray-400",
                        "{workflow.task_ids.len()} tasks"
                    }
                }
            }),
            right_content: Some(rsx! {
                div { class: "flex items-center gap-2",
                    Icon {
                        name: "play".to_string(),
                        class: Some("w-5 h-5 text-blue-600 dark:text-blue-400 animate-spin".to_string()),
                        size: None,
                        variant: Some("outline".to_string()),
                    }
                    Badge {
                        color: BadgeColor::Blue,
                        "In Progress"
                    }
                }
            }),
            onclick: move |_| {
                navigator.push(Route::WorkflowDetailsPage { id: workflow.id.clone() });
            },
        }
    }
}
