use crate::components::{IconButton, IconButtonSize, IconButtonVariant, PageHeader};
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use s_e_e_core::WorkflowJson;

use super::hooks::use_workflow_visualizer;

#[component]
pub fn WorkflowVisualizerPage(id: String) -> Element {
    let navigator = use_navigator();
    let mut error_message = use_signal(String::new);

    // Load workflow from query
    let workflow = match use_workflow_visualizer(id.clone()) {
        Ok(w) => w,
        Err(e) => {
            return rsx! {
                div { class: "flex items-center justify-center h-full",
                    div { class: "text-center",
                        h2 { class: "text-xl font-semibold text-zinc-900 dark:text-white mb-2",
                            "Failed to load workflow"
                        }
                        p { class: "text-red-600 dark:text-red-400",
                            "{e}"
                        }
                    }
                }
            };
        }
    };

    // Parse and prepare workflow JSON
    let workflow_content_clone = workflow.clone();
    let workflow_json_str = use_memo(move || {
        if let Some(workflow_def) = &workflow_content_clone {
            match serde_json::from_str::<WorkflowJson>(&workflow_def.content) {
                Ok(workflow_json) => match serde_json::to_string(&workflow_json) {
                    Ok(json_str) => Some(json_str),
                    Err(e) => {
                        error_message.set(format!("Failed to serialize workflow: {}", e));
                        None
                    }
                },
                Err(e) => {
                    error_message.set(format!("Failed to parse workflow: {}", e));
                    None
                }
            }
        } else {
            None
        }
    });

    rsx! {
        div { class: "flex flex-col h-screen",
            // Header
            div { class: "px-6 py-4 bg-white dark:bg-zinc-900 border-b border-zinc-200 dark:border-zinc-700",
                PageHeader {
                    title: "Workflow Visualizer".to_string(),
                    description: "Interactive workflow diagram".to_string(),
                    actions: Some(rsx! {
                        IconButton {
                            variant: IconButtonVariant::Ghost,
                            size: IconButtonSize::Medium,
                            onclick: move |_| {
                                navigator.go_back();
                            },
                            icon: Some("arrow_left".to_string()),
                            icon_variant: "outline".to_string(),
                            "Back"
                        }
                    }),
                }
            }

            // Content
            div { class: "flex-1 relative",
                match &workflow {
                    None => rsx! {
                        div { class: "flex items-center justify-center h-full",
                            div { class: "text-center",
                                h2 { class: "text-xl font-semibold text-zinc-900 dark:text-white mb-2",
                                    "Workflow not found"
                                }
                                p { class: "text-zinc-600 dark:text-zinc-400 mb-4",
                                    "The requested workflow could not be found."
                                }
                                IconButton {
                                    variant: IconButtonVariant::Primary,
                                    size: IconButtonSize::Medium,
                                    onclick: move |_| {
                                        navigator.go_back();
                                    },
                                    icon: Some("workflows".to_string()),
                                    icon_variant: "outline".to_string(),
                                    "Go to Workflows"
                                }
                            }
                        }
                    },
                    Some(_workflow_def) => rsx! {
                        if !error_message().is_empty() {
                            div { class: "absolute top-4 left-1/2 transform -translate-x-1/2 z-50",
                                div { class: "bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-700 rounded-lg p-4",
                                    p { class: "text-red-700 dark:text-red-300", "{error_message()}" }
                                }
                            }
                        }

                        // Script to send workflow data to iframe
                        if let Some(json_str) = workflow_json_str() {
                            script {
                                dangerous_inner_html: format!(
                                    r#"
                                    setTimeout(function() {{
                                        try {{
                                            const iframe = document.getElementById('workflow-visualizer-iframe');
                                            if (iframe && iframe.contentWindow) {{
                                                const workflowData = {};
                                                iframe.contentWindow.postMessage({{
                                                    type: 'LOAD_WORKFLOW',
                                                    payload: {{ workflow: workflowData }}
                                                }}, '*');
                                            }} else {{
                                                console.error('Iframe or contentWindow not available');
                                            }}
                                        }} catch (e) {{
                                            console.error('Failed to send workflow to iframe:', e);
                                        }}
                                    }}, 800);
                                    "#,
                                    json_str
                                )
                            }
                        }

                        iframe {
                            id: "workflow-visualizer-iframe",
                            srcdoc: format!(
                                r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Workflow Visualizer</title>
    <link rel="stylesheet" href="{}" />
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="{}"></script>
  </body>
</html>"#,
                                asset!("/assets/workflow-visualizer/index.css"),
                                asset!("/assets/workflow-visualizer/index.js")
                            ),
                            class: "w-full h-full border-0",
                        }
                    }
                }
            }
        }
    }
}
