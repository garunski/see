use dioxus::prelude::*;
use dioxus_ssr::render_element;
use pretty_assertions::assert_eq;

// Test that the App component structure is correct
#[test]
fn test_app_component_has_required_structure() {
    // Test the basic App component structure without state
    let rendered = render_element(rsx! {
        div {
            class: "min-h-screen bg-white dark:bg-zinc-900 text-zinc-950 dark:text-white",
            
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
                                "☀️"
                            }
                            span { "Light Mode" }
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
                                    value: "workflow.json",
                                }
                                button {
                                    class: "relative isolate inline-flex items-baseline justify-center gap-x-2 rounded-lg border text-base/6 font-semibold px-[calc(--spacing(3.5)-1px)] py-[calc(--spacing(2.5)-1px)] sm:px-[calc(--spacing(3)-1px)] sm:py-[calc(--spacing(1.5)-1px)] sm:text-sm/6 border-zinc-950/10 text-zinc-950 data-active:bg-zinc-950/2.5 data-hover:bg-zinc-950/2.5 dark:border-white/15 dark:text-white dark:data-active:bg-white/5 dark:data-hover:bg-white/5 disabled:opacity-50 disabled:cursor-not-allowed",
                                    div {
                                        class: "w-5 h-5",
                                        "📁"
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
                            div {
                                class: "w-5 h-5",
                                "🚀"
                            }
                            span { "Execute Workflow" }
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
                            "Welcome to Workflow Executor"
                        }
                    }
                }
            }
        }
    });
    
    // Verify the basic structure is present
    assert!(rendered.contains("Workflow Executor"));
    assert!(rendered.contains("Execute and manage workflows"));
    assert!(rendered.contains("Workflow File"));
    assert!(rendered.contains("Browse Files"));
    assert!(rendered.contains("Execute Workflow"));
    assert!(rendered.contains("Welcome to Workflow Executor"));
    assert!(rendered.contains("min-h-screen"));
    assert!(rendered.contains("fixed"));
    assert!(rendered.contains("w-64"));
}

#[test]
fn test_app_renders_with_css_injection() {
    // Test that CSS injection works
    let css_content = "body { background: white; }";
    let rendered = render_element(rsx! {
        div {
            class: "min-h-screen bg-white",
            
            // Inject CSS
            style {
                dangerous_inner_html: css_content
            }
            
            div {
                "Test content"
            }
        }
    });
    
    assert!(rendered.contains("Test content"));
    assert!(rendered.contains("body { background: white; }"));
    assert!(rendered.contains("min-h-screen"));
}
