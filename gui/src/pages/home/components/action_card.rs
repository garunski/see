use crate::icons::Icon;
use crate::layout::router::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::Link;

use super::ActionIcon;

#[derive(Props, PartialEq, Clone)]
pub struct ActionCardProps {
    pub title: String,
    pub description: String,
    pub icon: ActionIcon,
    pub route: Route,
}

#[component]
pub fn ActionCard(props: ActionCardProps) -> Element {
    let ActionCardProps {
        title,
        description,
        icon,
        route,
    } = props;

    let (icon_name, icon_class) = match icon {
        ActionIcon::Upload => ("upload", "h-5 w-5 text-green-600 dark:text-green-400"),
        ActionIcon::History => ("history", "h-5 w-5 text-purple-600 dark:text-purple-400"),
        ActionIcon::Workflows => ("workflows", "h-5 w-5 text-orange-600 dark:text-orange-400"),
    };

    let bg_class = match icon {
        ActionIcon::Upload => "bg-green-100 dark:bg-green-900/20",
        ActionIcon::History => "bg-purple-100 dark:bg-purple-900/20",
        ActionIcon::Workflows => "bg-orange-100 dark:bg-orange-900/20",
    };

    rsx! {
        Link {
            to: route,
            class: "group relative rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800 p-6 hover:bg-zinc-50 dark:hover:bg-zinc-700 transition-colors",
            div { class: "flex items-center gap-3",
                div { class: "flex h-10 w-10 items-center justify-center rounded-lg {bg_class}",
                    Icon {
                        name: icon_name.to_string(),
                        class: Some(icon_class.to_string()),
                        size: None,
                        variant: Some("outline".to_string()),
                    }
                }
                div {
                    h3 { class: "text-sm font-semibold text-zinc-900 dark:text-white", "{title}" }
                    p { class: "text-xs text-zinc-500 dark:text-zinc-400", "{description}" }
                }
            }
        }
    }
}
