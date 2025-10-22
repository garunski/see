use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct BadgeProps {
    pub variant: BadgeVariant,
    pub children: Element,
}

#[derive(PartialEq, Clone)]
pub enum BadgeVariant {
    Success,
    Error,
    #[allow(dead_code)]
    Warning,
    Info,
    Neutral,
}

impl BadgeVariant {
    fn classes(&self) -> &'static str {
        match self {
            BadgeVariant::Success => {
                "bg-emerald-100 text-emerald-800 dark:bg-emerald-900 dark:text-emerald-200"
            }
            BadgeVariant::Error => "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
            BadgeVariant::Warning => {
                "bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200"
            }
            BadgeVariant::Info => "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
            BadgeVariant::Neutral => {
                "bg-zinc-100 text-zinc-800 dark:bg-zinc-900 dark:text-zinc-200"
            }
        }
    }
}

#[component]
pub fn Badge(props: BadgeProps) -> Element {
    rsx! {
        span {
            class: format!("px-3 py-1 text-sm rounded-full font-medium {}", props.variant.classes()),
            {props.children}
        }
    }
}
