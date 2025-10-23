use dioxus::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Danger,
    Ghost,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

#[component]
pub fn Button(
    variant: ButtonVariant,
    size: ButtonSize,
    disabled: Option<bool>,
    loading: Option<bool>,
    onclick: Option<EventHandler<()>>,
    class: Option<String>,
    children: Element,
) -> Element {
    let disabled = disabled.unwrap_or(false);
    let loading = loading.unwrap_or(false);
    let is_disabled = disabled || loading;

    let base_classes = "inline-flex items-center justify-center font-medium transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 disabled:opacity-50 disabled:cursor-not-allowed";

    let variant_classes = match variant {
        ButtonVariant::Primary => "bg-blue-600 text-white hover:bg-blue-500 focus-visible:outline-blue-600",
        ButtonVariant::Secondary => "bg-zinc-100 text-zinc-900 hover:bg-zinc-200 dark:bg-zinc-800 dark:text-zinc-100 dark:hover:bg-zinc-700 focus-visible:outline-zinc-600",
        ButtonVariant::Danger => "bg-red-600 text-white hover:bg-red-500 focus-visible:outline-red-600",
        ButtonVariant::Ghost => "text-zinc-600 hover:text-zinc-900 hover:bg-zinc-100 dark:text-zinc-400 dark:hover:text-white dark:hover:bg-zinc-800 focus-visible:outline-zinc-600",
    };

    let size_classes = match size {
        ButtonSize::Small => "px-3 py-1.5 text-sm rounded-md",
        ButtonSize::Medium => "px-4 py-2 text-sm rounded-lg",
        ButtonSize::Large => "px-6 py-3 text-base rounded-lg",
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
                svg {
                    class: "w-4 h-4 animate-spin mr-2",
                    view_box: "0 0 20 20",
                    fill: "currentColor",
                    path { d: "M10 2a8 8 0 100 16 8 8 0 000-16zM8.5 10a1.5 1.5 0 113 0 1.5 1.5 0 01-3 0z" }
                }
            }
            {children}
        }
    }
}
