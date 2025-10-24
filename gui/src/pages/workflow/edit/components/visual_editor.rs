use dioxus::prelude::*;

use super::super::{load_workflow_script, MESSAGE_LISTENER_SCRIPT};

#[derive(Props, PartialEq, Clone)]
pub struct VisualEditorProps {
    pub workflow_json_str: Memo<Option<String>>,
    pub edited_workflow_name: Signal<String>,
    pub selected_node_info: Signal<String>,
}

#[component]
pub fn VisualEditor(props: VisualEditorProps) -> Element {
    let VisualEditorProps {
        workflow_json_str,
        edited_workflow_name,
        selected_node_info,
    } = props;

    rsx! {
        // Message listener for node clicks from iframe
        script {
            dangerous_inner_html: MESSAGE_LISTENER_SCRIPT.to_string()
        }

        div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 shadow-sm",
            div { class: "p-4 border-b border-zinc-200 dark:border-zinc-700",
                div { class: "flex items-center justify-between",
                    h3 { class: "text-lg font-semibold text-zinc-900 dark:text-white",
                        "Visual Editor"
                    }
                    div { class: "text-sm text-zinc-500 dark:text-zinc-400",
                        "Drag nodes to reposition, double-click to edit"
                    }
                }
            }
            div { class: "relative h-[calc(100vh-100px)]",
                // Selected node info display
                div {
                    id: "selected-node-info",
                    class: "absolute top-4 right-4 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 px-3 py-2 rounded-lg text-sm font-medium z-10",
                    "{selected_node_info()}"
                }

                if let Some(json_str) = workflow_json_str() {
                    // Script to send workflow data to iframe and set up click handling
                    script {
                        dangerous_inner_html: load_workflow_script(&json_str, &edited_workflow_name())
                    }

                    iframe {
                        id: "workflow-editor-iframe",
                        srcdoc: format!(
                            r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Workflow Editor</title>
    <link rel="stylesheet" href="{}" />
    <link rel="stylesheet" href="{}" />
    <script>
      // Set mode before React app loads
      window.WORKFLOW_MODE = 'editor';
    </script>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="{}"></script>
  </body>
</html>"#,
                            asset!("/assets/workflow-visualizer/index.css"),
                            asset!("/assets/tailwind.css"),
                            asset!("/assets/workflow-visualizer/index.js")
                        ),
                        class: "w-full h-full border-0 rounded-b-xl min-h-0",
                    }
                } else {
                    div { class: "flex items-center justify-center h-full",
                        div { class: "text-center",
                            div { class: "text-red-600 dark:text-red-400 mb-2", "Invalid Workflow" }
                            p { class: "text-zinc-600 dark:text-zinc-400", "Please fix the JSON before switching to visual mode" }
                        }
                    }
                }
            }
        }
    }
}
