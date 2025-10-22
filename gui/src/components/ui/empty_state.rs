use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct EmptyStateProps {
    pub title: String,
    pub description: Option<String>,
    pub children: Option<Element>,
}

#[component]
pub fn EmptyState(props: EmptyStateProps) -> Element {
    rsx! {
        div {
            class: "text-center py-12",
            div {
                class: "text-zinc-500 dark:text-zinc-400 text-sm",
                {props.title}
            }
            if let Some(description) = props.description {
                div {
                    class: "text-zinc-400 dark:text-zinc-500 text-xs mt-1",
                    {description}
                }
            }
            if let Some(children) = props.children {
                div {
                    class: "mt-4",
                    {children}
                }
            }
        }
    }
}
