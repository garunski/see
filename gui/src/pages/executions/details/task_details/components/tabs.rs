use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct TaskDetailsTabsProps {
    pub selected_tab: Signal<String>,
    pub on_tab_change: EventHandler<String>,
    pub show_user_input: bool,
}

#[component]
pub fn TaskDetailsTabs(props: TaskDetailsTabsProps) -> Element {
    let TaskDetailsTabsProps {
        selected_tab,
        on_tab_change,
        show_user_input,
    } = props;

    let is_active = |tab: &str| -> String {
        format!(
            "py-2 px-1 border-b-2 font-medium text-sm cursor-pointer {}",
            if selected_tab() == tab {
                "border-blue-500 text-blue-600 dark:text-blue-400"
            } else {
                "border-transparent text-zinc-500 hover:text-zinc-700 hover:border-zinc-300 dark:text-zinc-400 dark:hover:text-zinc-300"
            }
        )
    };

    rsx! {
        div { class: "border-b border-zinc-200 dark:border-zinc-700",
            div { class: "flex space-x-8",
                button {
                    class: is_active("Details"),
                    onclick: move |_| on_tab_change.call("Details".to_string()),
                    "Details"
                }
                button {
                    class: is_active("Output"),
                    onclick: move |_| on_tab_change.call("Output".to_string()),
                    "Output"
                }
                if show_user_input {
                    button {
                        class: is_active("User Input"),
                        onclick: move |_| on_tab_change.call("User Input".to_string()),
                        "User Input"
                    }
                }
            }
        }
    }
}
