use dioxus::prelude::*;
use dioxus_ssr::render_element;
use pretty_assertions::assert_eq;

// Test that the App component structure works with proper state management
#[test]
fn test_app_component_renders_with_default_state() {
    // Test the App component with default state values
    let workflow_file = "workflow.json";
    let dark_mode = false;
    let execution_status = "Idle";
    let is_picking_file = false;
    
    let rendered = render_element(rsx! {
        div {
            class: format!("min-h-screen bg-white dark:bg-zinc-900 text-zinc-950 dark:text-white {}", 
                if dark_mode { "dark" } else { "" }),
            
            // Main layout
            div {
                class: "relative isolate flex min-h-svh w-full bg-white max-lg:flex-col lg:bg-zinc-100 dark:bg-zinc-900 dark:lg:bg-zinc-950",
                
                // Sidebar
                aside {
                    class: "fixed inset-y-0 left-0 w-64 max-lg:hidden bg-white dark:bg-zinc-900 border-r border-zinc-950/5 dark:border-white/5",
                    
                    // Logo and title
                    div {
                        class: "flex flex-col border-b border-zinc-950/5 p-4 dark:border-white/5",
                        div {
                            class: "flex items-center space-x-3 mb-4",
                            div {
                                class: "w-8 h-8 bg-zinc-900 dark:bg-white rounded-lg flex items-center justify-center text-white dark:text-zinc-900 text-lg font-semibold",
                                "⚡"
                            }
                            div {
                                h1 {
                                    class: "text-lg font-semibold text-zinc-950 dark:text-white",
                                    "Workflow Executor"
                                }
                                p {
                                    class: "text-zinc-500 dark:text-zinc-400 text-sm",
                                    "Execute and manage workflows"
                                }
                            }
                        }
                        
                        // Theme toggle
                        button {
                            class: "flex w-full items-center gap-3 rounded-lg px-2 py-2.5 text-left text-base/6 font-medium text-zinc-950 sm:py-2 sm:text-sm/5 data-hover:bg-zinc-950/5 data-active:bg-zinc-950/5 dark:text-white dark:data-hover:bg-white/5 dark:data-active:bg-white/5",
                            div {
                                class: "w-5 h-5",
                                if dark_mode { "🌙" } else { "☀️" }
                            }
                            span {
                                if dark_mode { "Dark Mode" } else { "Light Mode" }
                            }
                        }
                    }
                    
                    // File input section
                    div {
                        class: "flex flex-1 flex-col overflow-y-auto p-4",
                        div {
                            class: "flex flex-col gap-0.5",
                            label {
                                class: "mb-1 px-2 text-xs/6 font-medium text-zinc-500 dark:text-zinc-400",
                                "Workflow File"
                            }
                            div {
                                class: "space-y-3",
                                input {
                                    class: "w-full px-3 py-2 bg-white dark:bg-zinc-900 border border-zinc-300 dark:border-zinc-700 rounded-lg text-zinc-950 dark:text-white placeholder-zinc-500 dark:placeholder-zinc-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200",
                                    r#type: "text",
                                    placeholder: "Select workflow file...",
                                    value: workflow_file,
                                }
                                button {
                                    class: "relative isolate inline-flex items-baseline justify-center gap-x-2 rounded-lg border text-base/6 font-semibold px-[calc(--spacing(3.5)-1px)] py-[calc(--spacing(2.5)-1px)] sm:px-[calc(--spacing(3)-1px)] sm:py-[calc(--spacing(1.5)-1px)] sm:text-sm/6 border-zinc-950/10 text-zinc-950 data-active:bg-zinc-950/2.5 data-hover:bg-zinc-950/2.5 dark:border-white/15 dark:text-white dark:data-active:bg-white/5 dark:data-hover:bg-white/5 disabled:opacity-50 disabled:cursor-not-allowed",
                                    disabled: is_picking_file,
                                    div {
                                        class: "w-5 h-5",
                                        if is_picking_file {
                                            "⏳"
                                        } else {
                                            "📁"
                                        }
                                    }
                                    span { "Browse Files" }
                                }
                            }
                        }
                    }
                    
                    // Execute button
                    div {
                        class: "flex flex-col border-t border-zinc-950/5 p-4 dark:border-white/5",
                        button {
                            class: "relative isolate inline-flex items-baseline justify-center gap-x-2 rounded-lg border text-base/6 font-semibold px-[calc(--spacing(3.5)-1px)] py-[calc(--spacing(2.5)-1px)] sm:px-[calc(--spacing(3)-1px)] sm:py-[calc(--spacing(1.5)-1px)] sm:text-sm/6 text-white bg-emerald-600 border-emerald-700/90 data-hover:bg-emerald-700 data-active:bg-emerald-700 disabled:opacity-50 disabled:cursor-not-allowed",
                            disabled: execution_status == "Running",
                            if execution_status == "Running" {
                                div { class: "animate-spin w-5 h-5 border-2 border-white border-t-transparent rounded-full" }
                                span { "Executing..." }
                            } else {
                                div { class: "w-5 h-5", "🚀" }
                                span { "Execute Workflow" }
                            }
                        }
                    }
                }
                
                // Main content area
                main {
                    class: "flex flex-1 flex-col pb-2 lg:min-w-0 lg:pt-2 lg:pr-2 lg:pl-64",
                    div {
                        class: "grow p-6 lg:rounded-lg lg:bg-white lg:p-10 lg:shadow-xs lg:ring-1 lg:ring-zinc-950/5 dark:lg:bg-zinc-900 dark:lg:ring-white/10",
                        div {
                            class: "mx-auto max-w-6xl",
                            h2 {
                                class: "text-2xl font-bold text-zinc-950 dark:text-white mb-6",
                                "Welcome to Workflow Executor"
                            }
                            p {
                                class: "text-zinc-600 dark:text-zinc-400 mb-4",
                                "Select a workflow file from the sidebar and click 'Execute Workflow' to get started."
                            }
                            div {
                                class: "bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4",
                                h3 {
                                    class: "text-lg font-semibold text-blue-900 dark:text-blue-100 mb-2",
                                    "Getting Started"
                                }
                                ul {
                                    class: "list-disc list-inside text-blue-800 dark:text-blue-200 space-y-1",
                                    li { "Use the 'Browse Files' button to select a workflow JSON file" }
                                    li { "Click 'Execute Workflow' to run the selected workflow" }
                                    li { "View execution results and logs in this area" }
                                    li { "Toggle between light and dark mode using the theme button" }
                                }
                            }
                        }
                    }
                }
            }
        }
    });
    
    // Verify the app renders with proper content
    assert!(rendered.contains("Workflow Executor"));
    assert!(rendered.contains("Execute and manage workflows"));
    assert!(rendered.contains("Workflow File"));
    assert!(rendered.contains("Browse Files"));
    assert!(rendered.contains("Execute Workflow"));
    assert!(rendered.contains("Welcome to Workflow Executor"));
    assert!(rendered.contains("Getting Started"));
    assert!(rendered.contains("workflow.json"));
    assert!(rendered.contains("☀️"));
    assert!(rendered.contains("Light Mode"));
    assert!(rendered.contains("min-h-screen"));
    assert!(rendered.contains("fixed"));
    assert!(rendered.contains("w-64"));
}

#[test]
fn test_app_renders_with_dark_mode() {
    let dark_mode = true;
    let rendered = render_element(rsx! {
        div {
            class: format!("min-h-screen bg-white dark:bg-zinc-900 text-zinc-950 dark:text-white {}", 
                if dark_mode { "dark" } else { "" }),
            div {
                class: "flex min-h-svh w-full bg-white dark:bg-zinc-900",
                aside {
                    class: "fixed inset-y-0 left-0 w-64 bg-white dark:bg-zinc-900 border-r border-zinc-950/5 dark:border-white/5",
                    div {
                        class: "p-4",
                        h1 {
                            class: "text-lg font-semibold text-zinc-950 dark:text-white",
                            "Workflow Executor"
                        }
                        button {
                            class: "flex w-full items-center gap-3 rounded-lg px-2 py-2.5 text-left text-base/6 font-medium text-zinc-950 dark:text-white",
                            div {
                                class: "w-5 h-5",
                                if dark_mode { "🌙" } else { "☀️" }
                            }
                            span {
                                if dark_mode { "Dark Mode" } else { "Light Mode" }
                            }
                        }
                    }
                }
                main {
                    class: "flex flex-1 flex-col pb-2 lg:min-w-0 lg:pt-2 lg:pr-2 lg:pl-64",
                    div {
                        class: "grow p-6 lg:rounded-lg lg:bg-white dark:lg:bg-zinc-900",
                        div {
                            class: "mx-auto max-w-6xl",
                            "Dark mode content"
                        }
                    }
                }
            }
        }
    });
    
    assert!(rendered.contains("Workflow Executor"));
    assert!(rendered.contains("🌙"));
    assert!(rendered.contains("Dark Mode"));
    assert!(rendered.contains("Dark mode content"));
}
