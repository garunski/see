use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct CardProps {
    pub children: Element,
}

#[component]
pub fn Card(props: CardProps) -> Element {
    rsx! {
        div {
            class: "bg-white dark:bg-zinc-900 rounded-lg shadow-sm ring-1 ring-zinc-950/5 dark:ring-white/10 p-6",
            {props.children}
        }
    }
}
