use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct JsonEditorProps {
    pub content: Signal<String>,
    pub workflow_name: Signal<String>,
    pub validation_error: Signal<String>,
    pub on_content_change: EventHandler<String>,
}

#[component]
pub fn JsonEditor(props: JsonEditorProps) -> Element {
    let JsonEditorProps {
        content,
        workflow_name,
        validation_error,
        on_content_change,
    } = props;

    rsx! {
        div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 p-8 shadow-sm",
            div { class: "space-y-6",
                div {
                    label { class: "block text-sm font-medium text-zinc-900 dark:text-white mb-2",
                        "Workflow Name"
                    }
                    div { class: "block w-full rounded-md border-0 py-1.5 text-zinc-900 dark:text-white shadow-sm ring-1 ring-inset ring-zinc-300 dark:ring-zinc-600 bg-zinc-50 dark:bg-zinc-700 sm:text-sm sm:leading-6",
                        {workflow_name()}
                    }
                    p { class: "mt-1 text-xs text-zinc-500 dark:text-zinc-400",
                        "Name is extracted from the JSON 'name' field"
                    }
                }

                div {
                    label { class: "block text-sm font-medium text-zinc-900 dark:text-white mb-2",
                        "Workflow Definition (JSON)"
                    }
                    textarea {
                        value: "{content()}",
                        oninput: move |evt| on_content_change.call(evt.value()),
                        placeholder: "Enter workflow JSON definition",
                        rows: 20,
                        class: "block w-full rounded-md border-0 py-1.5 text-zinc-900 dark:text-white shadow-sm ring-1 ring-inset ring-zinc-300 dark:ring-zinc-600 placeholder:text-zinc-400 dark:placeholder:text-zinc-500 focus:ring-2 focus:ring-inset focus:ring-blue-600 dark:bg-zinc-700 sm:text-sm sm:leading-6 font-mono"
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
}
