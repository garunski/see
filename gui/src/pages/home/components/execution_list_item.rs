use crate::components::Badge;
use crate::icons::Icon;
use crate::layout::router::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::Link;
use s_e_e_core::{WorkflowExecutionStatus, WorkflowExecutionSummary};

#[component]
pub fn ExecutionListItem(execution: WorkflowExecutionSummary) -> Element {
    let badge_color = match execution.status {
        WorkflowExecutionStatus::WaitingForInput => crate::components::BadgeColor::Amber,
        WorkflowExecutionStatus::Complete => crate::components::BadgeColor::Emerald,
        WorkflowExecutionStatus::Failed => crate::components::BadgeColor::Red,
        WorkflowExecutionStatus::Running => crate::components::BadgeColor::Blue,
        WorkflowExecutionStatus::Pending => crate::components::BadgeColor::Zinc,
    };

    let status_text = match execution.status {
        WorkflowExecutionStatus::WaitingForInput => "Waiting for Input",
        WorkflowExecutionStatus::Complete => "Complete",
        WorkflowExecutionStatus::Failed => "Failed",
        WorkflowExecutionStatus::Running => "Running",
        WorkflowExecutionStatus::Pending => "Pending",
    };

    rsx! {
        li {
            class: "relative flex justify-between gap-x-6 px-4 py-5 hover:bg-gray-50 sm:px-6 dark:hover:bg-white/[0.025] cursor-pointer",
            onclick: move |_| {},
            div { class: "flex min-w-0 gap-x-4",
                div { class: "size-12 flex-none rounded-full bg-gray-50 dark:bg-gray-800 dark:outline dark:outline-1 dark:-outline-offset-1 dark:outline-white/10 flex items-center justify-center",
                    Icon {
                        name: "clock".to_string(),
                        class: Some("size-6 text-gray-400 dark:text-gray-500".to_string()),
                        size: None,
                        variant: Some("outline".to_string()),
                    }
                }
                div { class: "min-w-0 flex-auto",
                    Link {
                        to: Route::WorkflowDetailsPage { id: execution.id.clone() },
                        class: "block",
                        p { class: "text-sm/6 font-semibold text-gray-900 dark:text-white",
                            {execution.workflow_name.clone()}
                        }
                    }
                    p { class: "mt-1 text-xs text-gray-500 dark:text-gray-400",
                        {execution.timestamp.to_string()}
                    }
                }
            }
            div { class: "flex shrink-0 items-center gap-x-4",
                div { class: "hidden sm:flex sm:flex-col sm:items-end",
                    Badge {
                        color: badge_color,
                        {status_text}
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
