use dioxus::prelude::*;

#[component]
pub fn TextareaInput(
    label: String,
    value: Signal<String>,
    oninput: EventHandler<String>,
    placeholder: Option<String>,
    help_text: Option<String>,
    rows: Option<usize>,
    disabled: Option<bool>,
) -> Element {
    let disabled = disabled.unwrap_or(false);
    let rows = rows.unwrap_or(3);

    rsx! {
        div {
            label {
                class: "block text-sm font-medium text-zinc-900 dark:text-white mb-2",
                {label}
            }
            textarea {
                value: "{value()}",
                oninput: move |evt| oninput.call(evt.value()),
                placeholder: placeholder.unwrap_or_default(),
                rows: rows,
                disabled: disabled,
                class: "block w-full rounded-md border-0 py-1.5 text-zinc-900 dark:text-white shadow-sm ring-1 ring-inset ring-zinc-300 dark:ring-zinc-600 placeholder:text-zinc-400 dark:placeholder:text-zinc-500 focus:ring-2 focus:ring-inset focus:ring-blue-600 dark:bg-zinc-700 sm:text-sm sm:leading-6 disabled:opacity-50 disabled:cursor-not-allowed font-mono"
            }
            if let Some(help) = help_text {
                p { class: "mt-1 text-xs text-zinc-500 dark:text-zinc-400",
                    {help}
                }
            }
        }
    }
}
