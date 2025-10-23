use crate::components::{Button, ButtonSize, ButtonVariant};
use crate::state::AppStateProvider;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use see_core::WorkflowJson;

#[component]
pub fn WorkflowVisualizerPage(id: String) -> Element {
    let state_provider = use_context::<AppStateProvider>();
    let navigator = use_navigator();

    let mut error_message = use_signal(|| String::new());

    // Load workflow from settings
    let workflow_id_clone = id.clone();
    let workflow_resource = use_resource(move || {
        let workflow_id = workflow_id_clone.clone();
        let settings = state_provider.settings;
        async move {
            let workflows = settings.read().get_workflows().clone();
            workflows.into_iter().find(|w| w.id == workflow_id)
        }
    });

    // Parse and prepare workflow JSON
    let workflow_json_str = use_memo(move || {
        if let Some(Some(workflow_def)) = workflow_resource.read().as_ref() {
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
            div { class: "flex items-center justify-between px-6 py-4 bg-white dark:bg-zinc-900 border-b border-zinc-200 dark:border-zinc-700",
                div { class: "flex items-center gap-4",
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Medium,
                        onclick: move |_| {
                            navigator.go_back();
                        },
                        "← Back"
                    }
                    h1 { class: "text-xl font-semibold text-zinc-900 dark:text-white",
                        "Workflow Visualizer"
                    }
                }
            }

            // Content
            div { class: "flex-1 relative",
                match workflow_resource.read().as_ref() {
                    None => rsx! {
                        div { class: "flex items-center justify-center h-full",
                            div { class: "text-center",
                                div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4" }
                                p { class: "text-zinc-600 dark:text-zinc-400", "Loading workflow..." }
                            }
                        }
                    },
                    Some(None) => rsx! {
                        div { class: "flex items-center justify-center h-full",
                            div { class: "text-center",
                                h2 { class: "text-xl font-semibold text-zinc-900 dark:text-white mb-2",
                                    "Workflow not found"
                                }
                                p { class: "text-zinc-600 dark:text-zinc-400 mb-4",
                                    "The requested workflow could not be found."
                                }
                                Button {
                                    variant: ButtonVariant::Primary,
                                    size: ButtonSize::Medium,
                                    onclick: move |_| {
                                        navigator.go_back();
                                    },
                                    "Go Back"
                                }
                            }
                        }
                    },
                    Some(Some(_workflow_def)) => rsx! {
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
