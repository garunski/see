use dioxus::prelude::*;

/// Node editor modal component for editing workflow nodes
/// This modal is controlled via JavaScript and doesn't need props
#[component]
pub fn NodeEditorModal() -> Element {
    rsx! {
        div {
            id: "node-editor-modal",
            class: "fixed inset-0 z-50 hidden items-center justify-center bg-black bg-opacity-50",
            style: "display: none;",
            div {
                class: "bg-white dark:bg-zinc-800 rounded-xl shadow-xl p-6 max-w-md w-full mx-4",
                h3 { class: "text-lg font-semibold text-zinc-900 dark:text-white mb-4", "Edit Node" }

                div { class: "space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-zinc-900 dark:text-white mb-1", "Node Name" }
                        input {
                            id: "node-name-input",
                            type: "text",
                            class: "w-full px-3 py-2 border border-zinc-300 dark:border-zinc-600 rounded-md text-zinc-900 dark:text-white bg-zinc-50 dark:bg-zinc-700 focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            placeholder: "Enter node name"
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-zinc-900 dark:text-white mb-1", "Function Type" }
                        select {
                            id: "node-function-select",
                            class: "w-full px-3 py-2 border border-zinc-300 dark:border-zinc-600 rounded-md text-zinc-900 dark:text-white bg-zinc-50 dark:bg-zinc-700 focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            option { value: "cli_command", "CLI Command" }
                            option { value: "cursor_agent", "Cursor Agent" }
                        }
                    }

                    div { id: "cli-fields",
                        div {
                            label { class: "block text-sm font-medium text-zinc-900 dark:text-white mb-1", "Command" }
                            input {
                                id: "node-command-input",
                                type: "text",
                                class: "w-full px-3 py-2 border border-zinc-300 dark:border-zinc-600 rounded-md text-zinc-900 dark:text-white bg-zinc-50 dark:bg-zinc-700 focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                placeholder: "e.g., echo, ls, curl"
                            }
                        }

                        div {
                            label { class: "block text-sm font-medium text-zinc-900 dark:text-white mb-1", "Arguments (comma-separated)" }
                            input {
                                id: "node-args-input",
                                type: "text",
                                class: "w-full px-3 py-2 border border-zinc-300 dark:border-zinc-600 rounded-md text-zinc-900 dark:text-white bg-zinc-50 dark:bg-zinc-700 focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                                placeholder: "e.g., Hello World, -l, /path/to/file"
                            }
                        }
                    }

                    div { id: "cursor-fields", style: "display: none;",
                        label { class: "block text-sm font-medium text-zinc-900 dark:text-white mb-1", "Prompt" }
                        textarea {
                            id: "node-prompt-input",
                            rows: 4,
                            class: "w-full px-3 py-2 border border-zinc-300 dark:border-zinc-600 rounded-md text-zinc-900 dark:text-white bg-zinc-50 dark:bg-zinc-700 focus:ring-2 focus:ring-blue-500 focus:border-transparent",
                            placeholder: "Enter your prompt for the Cursor agent"
                        }
                    }
                }

                div { class: "flex gap-3 justify-end mt-6",
                    button {
                        id: "node-editor-cancel",
                        class: "px-4 py-2 text-sm font-medium text-zinc-700 dark:text-zinc-300 bg-zinc-100 dark:bg-zinc-700 hover:bg-zinc-200 dark:hover:bg-zinc-600 rounded-md transition-colors",
                        "Cancel"
                    }
                    button {
                        id: "node-editor-save",
                        class: "px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 rounded-md transition-colors",
                        "Save Changes"
                    }
                }
            }
        }
    }
}
