use dioxus::prelude::*;
use see_core::TaskStatus;

#[component]
pub fn StepNavigator(
    current_step: usize,
    total_steps: usize,
    task_name: String,
    task_status: TaskStatus,
    on_prev: EventHandler<()>,
    on_next: EventHandler<()>,
) -> Element {
    let is_first = current_step == 0;
    let is_last = current_step >= total_steps - 1;
    let step_display = current_step + 1;

    let status_badge_class = match task_status {
        TaskStatus::Complete => {
            "bg-emerald-100 text-emerald-800 dark:bg-emerald-900/20 dark:text-emerald-400"
        }
        TaskStatus::Failed => "bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400",
        TaskStatus::InProgress => {
            "bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-400"
        }
        TaskStatus::Pending => "bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-400",
    };

    rsx! {
        div { class: "flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-lg",
            button { class: "flex items-center space-x-2 px-3 py-2 text-sm font-medium rounded-lg transition-colors duration-200", class: if is_first { "text-gray-400 cursor-not-allowed" } else { "text-gray-700 hover:bg-gray-200 dark:text-gray-300 dark:hover:bg-gray-700" }, disabled: is_first, onclick: move |_| if !is_first { on_prev.call(()) },
                div { class: "w-4 h-4", "◀" }
                span { "Previous" }
            }

            div { class: "flex flex-col items-center space-y-2",
                div { class: "text-sm font-medium text-gray-700 dark:text-gray-300", "Step {step_display} of {total_steps}" }
                div { class: "flex items-center space-x-3",
                    div { class: "text-lg font-semibold text-gray-900 dark:text-white", {task_name} }
                    span { class: "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {status_badge_class}", {task_status.to_string()} }
                }
            }

            button { class: "flex items-center space-x-2 px-3 py-2 text-sm font-medium rounded-lg transition-colors duration-200", class: if is_last { "text-gray-400 cursor-not-allowed" } else { "text-gray-700 hover:bg-gray-200 dark:text-gray-300 dark:hover:bg-gray-700" }, disabled: is_last, onclick: move |_| if !is_last { on_next.call(()) },
                span { "Next" }
                div { class: "w-4 h-4", "▶" }
            }
        }
    }
}
