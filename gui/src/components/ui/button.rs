use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct ButtonProps {
    pub variant: ButtonVariant,
    pub disabled: Option<bool>,
    pub onclick: Option<EventHandler<()>>,
    pub children: Element,
}

#[derive(PartialEq, Clone)]
pub enum ButtonVariant {
    #[allow(dead_code)]
    Primary,
    #[allow(dead_code)]
    Secondary,
    Ghost,
}

impl ButtonVariant {
    fn classes(&self) -> &'static str {
        match self {
            ButtonVariant::Primary => "bg-blue-600 text-white hover:bg-blue-700 dark:bg-blue-500 dark:hover:bg-blue-400",
            ButtonVariant::Secondary => "bg-zinc-100 text-zinc-900 hover:bg-zinc-200 dark:bg-zinc-800 dark:text-zinc-100 dark:hover:bg-zinc-700",
            ButtonVariant::Ghost => "text-zinc-600 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-white hover:bg-zinc-100 dark:hover:bg-zinc-800",
        }
    }
}

#[component]
pub fn Button(props: ButtonProps) -> Element {
    let disabled = props.disabled.unwrap_or(false);
    let base_classes = "flex items-center gap-2 px-3 py-2 transition-colors rounded-lg font-medium";
    let variant_classes = props.variant.classes();
    let disabled_classes = if disabled {
        "opacity-50 cursor-not-allowed"
    } else {
        ""
    };

    rsx! {
        button {
            class: format!("{} {} {}", base_classes, variant_classes, disabled_classes),
            disabled: disabled,
            onclick: move |_| {
                if !disabled {
                    if let Some(handler) = &props.onclick {
                        handler.call(());
                    }
                }
            },
            {props.children}
        }
    }
}
