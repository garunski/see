use crate::icons::Icon;
use dioxus::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum IconButtonVariant {
    Primary,
    Secondary,
    Danger,
    Ghost,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IconButtonSize {
    Small,
    Medium,
    Large,
}

#[component]
pub fn IconButton(
    variant: IconButtonVariant,
    size: IconButtonSize,
    #[props(default)] disabled: Option<bool>,
    #[props(default)] loading: Option<bool>,
    #[props(default)] onclick: Option<EventHandler<()>>,
    #[props(default)] class: Option<String>,
    #[props(default)] icon: Option<String>,
    #[props(default = "outline".to_string())] icon_variant: String,
    #[props(default = "left".to_string())] icon_position: String,
    children: Element,
) -> Element {
    let disabled = disabled.unwrap_or(false);
    let loading = loading.unwrap_or(false);
    let is_disabled = disabled || loading;

    let base_classes = "inline-flex items-center justify-center font-medium transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 disabled:opacity-50 disabled:cursor-not-allowed";

    let variant_classes = match variant {
        IconButtonVariant::Primary => "bg-blue-600 text-white hover:bg-blue-500 focus-visible:outline-blue-600",
        IconButtonVariant::Secondary => "bg-zinc-100 text-zinc-900 hover:bg-zinc-200 dark:bg-zinc-800 dark:text-zinc-100 dark:hover:bg-zinc-700 focus-visible:outline-zinc-600",
        IconButtonVariant::Danger => "bg-red-600 text-white hover:bg-red-500 focus-visible:outline-red-600",
        IconButtonVariant::Ghost => "text-zinc-600 hover:text-zinc-900 hover:bg-zinc-100 dark:text-zinc-400 dark:hover:text-white dark:hover:bg-zinc-800 focus-visible:outline-zinc-600",
    };

    let size_classes = match size {
        IconButtonSize::Small => "px-3 py-1.5 text-sm rounded-md",
        IconButtonSize::Medium => "px-4 py-2 text-sm rounded-lg",
        IconButtonSize::Large => "px-6 py-3 text-base rounded-lg",
    };

    let combined_classes = format!(
        "{} {} {} {}",
        base_classes,
        variant_classes,
        size_classes,
        class.unwrap_or_default()
    );

    rsx! {
        button {
            class: combined_classes,
            disabled: is_disabled,
            onclick: move |_| {
                if !is_disabled {
                    if let Some(handler) = onclick {
                        handler.call(());
                    }
                }
            },
            if loading {
                Icon {
                    name: "play".to_string(),
                    class: Some("w-4 h-4 animate-spin mr-2".to_string()),
                    size: None,
                    variant: Some("outline".to_string()),
                }
            }
            if !loading {
                if let Some(icon_name) = icon {
                    if icon_position == "left" {
                        Icon {
                            name: icon_name,
                            class: Some("h-4 w-4 mr-1.5".to_string()),
                            size: None,
                            variant: Some(icon_variant.clone()),
                        }
                        {children}
                    } else {
                        {children}
                        Icon {
                            name: icon_name,
                            class: Some("h-4 w-4 ml-1.5".to_string()),
                            size: None,
                            variant: Some(icon_variant.clone()),
                        }
                    }
                } else {
                    {children}
                }
            }
        }
    }
}
