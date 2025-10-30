use crate::components::forms::TextareaInput;
use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct JsonEditorProps {
    pub content: Signal<String>,
    pub workflow_name: Signal<String>,
    pub validation_error: Signal<String>,
    pub on_content_change: EventHandler<String>,
    pub is_readonly: Option<bool>,
}

#[component]
pub fn JsonEditor(props: JsonEditorProps) -> Element {
    let JsonEditorProps {
        content,
        workflow_name: _,
        validation_error,
        on_content_change,
        is_readonly,
    } = props;

    let readonly = is_readonly.unwrap_or(false);

    let mut has_formatted = use_signal(|| false);

    use_effect(move || {
        if !has_formatted() {
            let content_str = content();

            match serde_json::from_str::<serde_json::Value>(&content_str) {
                Ok(json_value) => {
                    if let Ok(formatted) = serde_json::to_string_pretty(&json_value) {
                        if formatted != content_str {
                            on_content_change.call(formatted);
                        }
                    }
                }
                Err(_) => {}
            }
            has_formatted.set(true);
        }
    });

    rsx! {
        div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 p-8 shadow-sm",
            div { class: "space-y-6",
                TextareaInput {
                    label: "Workflow Definition (JSON)",
                    value: content,
                    oninput: on_content_change,
                    placeholder: Some("Enter workflow JSON definition".to_string()),
                    rows: Some(20),
                    disabled: Some(readonly),
                }

                if !validation_error().is_empty() {
                    div { class: "mt-2 text-sm text-red-600 dark:text-red-400",
                        {validation_error()}
                    }
                }
            }
        }
    }
}
