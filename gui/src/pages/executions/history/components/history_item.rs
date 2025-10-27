use crate::components::layout::ListItem;
use crate::components::{Badge, BadgeColor};
use crate::layout::router::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use s_e_e_core::{WorkflowExecutionStatus, WorkflowExecutionSummary};

#[component]
pub fn HistoryItem(execution: WorkflowExecutionSummary) -> Element {
    let navigator = use_navigator();

    let badge_color = match execution.status {
        WorkflowExecutionStatus::WaitingForInput => BadgeColor::Amber,
        WorkflowExecutionStatus::Complete => BadgeColor::Emerald,
        WorkflowExecutionStatus::Failed => BadgeColor::Red,
        WorkflowExecutionStatus::Running => BadgeColor::Blue,
        WorkflowExecutionStatus::Pending => BadgeColor::Zinc,
    };

    let status_text = match execution.status {
        WorkflowExecutionStatus::WaitingForInput => "Waiting for Input",
        WorkflowExecutionStatus::Complete => "Success",
        WorkflowExecutionStatus::Failed => "Failed",
        WorkflowExecutionStatus::Running => "Running",
        WorkflowExecutionStatus::Pending => "Pending",
    };

    rsx! {
        ListItem {
            icon_name: "workflows".to_string(),
            icon_variant: Some("outline".to_string()),
            title: rsx! {
                {execution.workflow_name.clone()}
            },
            subtitle: Some(rsx! {
                div { class: "flex flex-col gap-1",
                    div { class: "text-sm text-gray-500 dark:text-gray-400",
                        "Executed: {execution.timestamp}"
                    }
                    div { class: "text-xs text-gray-500 dark:text-gray-400",
                        "{execution.task_count} tasks completed"
                    }
                }
            }),
            right_content: Some(rsx! {
                Badge {
                    color: badge_color,
                    {status_text}
                }
            }),
            onclick: move |_| {
                navigator.push(Route::WorkflowDetailsPage { id: execution.id.clone() });
            },
        }
    }
}
