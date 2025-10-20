use crate::components::{
    ContextPanel, ErrorsPanel, OutputLogsPanel, SettingsScreen, Sidebar, Toast, WorkflowInfoCard,
};
use crate::services::workflow::{create_output_channel, run_workflow};
use crate::state::{AppState, SidebarTab};
use dioxus::prelude::*;
use rfd::FileDialog;
use see_core::{AppSettings, AuditStore, RedbStore, Theme};
use std::sync::Arc;

#[component]
pub fn App() -> Element {
    // 1. Initialize store and provide as context (NOT reactive state)
    let store = use_hook(|| {
        RedbStore::new_default()
            .ok()
            .map(Arc::new)
    });
    
    // Clone store for use in closures
    let store_clone = store.clone();
    let store_clone2 = store.clone();
    let store_clone3 = store.clone();
    let store_clone4 = store.clone();
    let store_clone5 = store.clone();
    let store_clone6 = store.clone();
    
    // Provide store via context so all components can access it
    use_context_provider(|| store.clone());
    
    // 2. Create independent settings signal
    let mut settings = use_signal(|| {
        // System-detected default
        AppSettings {
            theme: match dark_light::detect() {
                dark_light::Mode::Dark => Theme::Dark,
                dark_light::Mode::Light => Theme::Light,
            },
        }
    });
    
    // 3. Load settings once on mount
    use_effect(move || {
        let store = store_clone.clone();
        spawn(async move {
            if let Some(ref s) = store {
                if let Ok(Some(loaded)) = s.load_settings().await {
                    settings.set(loaded);
                }
            }
        });
    });
    
    // 4. Keep existing UI state
    let mut state = use_signal(|| AppState::default());
    
    // Load history - pass store directly, don't read from state
    use_effect(move || {
        let store = store_clone2.clone();
        spawn(async move {
            if let Some(s) = store {
                match s.list_workflow_executions(50).await {
                    Ok(history) => {
                        state.write().workflow_history = history;
                    }
                    Err(e) => {
                        state.write().toast_message = Some(format!("Failed to load history: {}", e));
                    }
                }
            }
        });
    });
    
    // 5. Compute dark mode from settings (for UI classes)
    let dark_mode_signal = use_memo(move || {
        match settings.read().theme {
            Theme::Dark => true,
            Theme::Light => false,
            Theme::System => matches!(dark_light::detect(), dark_light::Mode::Dark),
        }
    });
    
    // 6. Settings modal state
    let mut show_settings = use_signal(|| false);
    
    // 7. Settings handlers
    let mut open_settings = move || {
        show_settings.set(true);
    };

    let mut close_settings = move || {
        show_settings.set(false);
    };

    let mut change_theme = move |new_theme: Theme| {
        settings.write().theme = new_theme;
        
        // Save immediately
        if let Some(ref s) = store_clone3 {
            let s = s.clone();
            spawn(async move {
                let _ = s.save_settings(&AppSettings { theme: new_theme }).await;
            });
        }
    };

    let workflow_result_signal = use_memo(move || state.read().workflow_result.clone());

    let mut on_next_step = move || {
        let current = state.read().current_step;
        let total = if let Some(ref history_item) = state.read().viewing_history_item {
            history_item.tasks.len()
        } else {
            state.read().tasks.len()
        };
        if current < total.saturating_sub(1) {
            state.write().current_step = current + 1;
        }
    };

    let mut on_prev_step = move || {
        let current = state.read().current_step;
        if current > 0 {
            state.write().current_step = current - 1;
        }
    };

    let on_jump_to_step = move |step: usize| {
        let total = if let Some(ref history_item) = state.read().viewing_history_item {
            history_item.tasks.len()
        } else {
            state.read().tasks.len()
        };
        if step < total {
            state.write().current_step = step;
        }
    };

    let mut pick_file = move || {
        state.write().is_picking_file = true;
        spawn(async move {
            if let Some(path) = FileDialog::new()
                .add_filter("JSON files", &["json"])
                .set_title("Select Workflow File")
                .pick_file()
            {
                if let Some(path_str) = path.to_str() {
                    state.write().workflow_file = path_str.to_string();
                    // Clear viewing history item when picking a new file
                    state.write().viewing_history_item = None;
                    state.write().selected_history_id = None;
                }
            }
            state.write().is_picking_file = false;
        });
    };

    let copy_to_clipboard = move |text: String| {
        println!("Copy to clipboard: {}", text);
    };
    let mut dismiss_toast = move || {
        state.write().toast_message = None;
    };

    let mut switch_tab = move |tab: SidebarTab| {
        let is_upload_tab = matches!(tab, SidebarTab::Upload);
        state.write().sidebar_tab = tab;
        // Clear viewing history item when switching to upload tab
        if is_upload_tab {
            state.write().viewing_history_item = None;
            state.write().selected_history_id = None;
        }
    };

    let load_execution = move |id: String| {
        let store = store_clone4.clone();
        spawn(async move {
            if let Some(s) = store {
                state.write().load_execution(&id, &s).await;
            }
        });
    };

    let delete_execution = move |id: String| {
        let store = store_clone5.clone();
        spawn(async move {
            if let Some(s) = store {
                state.write().delete_execution(&id, &s).await;
            }
        });
    };

    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }
        div {
            class: format!("min-h-screen bg-white dark:bg-zinc-900 text-zinc-950 dark:text-white {}", if *dark_mode_signal.read() { "dark" } else { "" }),
            onkeydown: move |evt| { match evt.key() { dioxus::events::Key::ArrowLeft | dioxus::events::Key::ArrowUp => on_prev_step(), dioxus::events::Key::ArrowRight | dioxus::events::Key::ArrowDown => on_next_step(), _ => {} } },

            Toast { message: state.read().toast_message.clone(), on_dismiss: move |_| dismiss_toast() }

            div { class: "flex flex-col lg:flex-row min-h-svh w-full bg-white lg:bg-zinc-100 dark:bg-zinc-900 dark:lg:bg-zinc-950",
                Sidebar {
                    workflow_file: state.read().workflow_file.clone(),
                    on_workflow_file_change: move |value| {
                        state.write().workflow_file = value;
                        // Clear viewing history item when selecting a new workflow file
                        state.write().viewing_history_item = None;
                        state.write().selected_history_id = None;
                    },
                    is_picking_file: state.read().is_picking_file,
                    on_pick_file: move |_| pick_file(),
                    on_open_settings: move |_| open_settings(),
                    execution_status: state.read().execution_status.clone(),
                    on_execute: move |_| {
                        let store_clone = store_clone6.clone();
                        let mut state_clone = state;
                        spawn(async move {
                            let file_path = state_clone.read().workflow_file.clone();
                            state_clone.write().reset_before_run();

                            let (output_callback, mut handles) = create_output_channel();

                            let mut state_clone2 = state_clone;
                            spawn(async move {
                                while let Some(msg) = handles.receiver.recv().await {
                                    state_clone2.write().output_logs.push(msg);
                                }
                            });

                            match run_workflow(file_path, output_callback, store_clone.map(|s| s as Arc<dyn AuditStore>)).await {
                                Ok(result) => {
                                    state_clone.write().apply_success(&result);
                                }
                                Err(e) => {
                                    state_clone.write().apply_failure(&e.to_string());
                                }
                            }
                        });
                    },
                    is_viewing_history: state.read().viewing_history_item.is_some(),
                    sidebar_tab: state.read().sidebar_tab.clone(),
                    on_tab_change: move |tab| switch_tab(tab),
                    workflow_history: state.read().workflow_history.clone(),
                    on_load_execution: move |id| load_execution(id),
                    on_delete_execution: move |id| delete_execution(id),
                    selected_history_id: state.read().selected_history_id.clone(),
                }

                main { class: "flex flex-1 flex-col pb-2 lg:min-w-0 lg:pt-2 lg:pr-2 lg:ml-64",
                    div { class: "grow p-6 lg:rounded-lg lg:bg-white lg:p-10 lg:shadow-xs lg:ring-1 lg:ring-zinc-950/5 dark:lg:bg-zinc-900 dark:lg:ring-white/10",
                        div { class: "mx-auto max-w-6xl",
                            if let Some(result) = workflow_result_signal.read().clone() {
                                WorkflowInfoCard { result: ReadOnlySignal::new(Signal::new(result)), tasks: state.read().tasks.clone(), current_step: state.read().current_step, on_next_step: on_next_step, on_prev_step: on_prev_step, on_jump_to_step: on_jump_to_step }
                            } else if let Some(ref history_item) = state.read().viewing_history_item {
                                // Convert WorkflowExecution to WorkflowResult for display
                                WorkflowInfoCard {
                                    result: ReadOnlySignal::new(Signal::new(see_core::WorkflowResult {
                                        success: history_item.success,
                                        workflow_name: history_item.workflow_name.clone(),
                                        task_count: history_item.tasks.len(),
                                        execution_id: history_item.id.clone(),
                                        tasks: history_item.tasks.clone(),
                                        final_context: serde_json::Value::Object(serde_json::Map::new()),
                                        audit_trail: history_item.audit_trail.clone(),
                                        per_task_logs: history_item.per_task_logs.clone(),
                                        errors: history_item.errors.clone(),
                                        output_logs: Vec::new(),
                                    })),
                                    tasks: history_item.tasks.clone(),
                                    current_step: state.read().current_step,
                                    on_next_step: on_next_step,
                                    on_prev_step: on_prev_step,
                                    on_jump_to_step: on_jump_to_step
                                }
                            }
                            div { class: "space-y-6",
                                OutputLogsPanel {
                                    per_task_logs: if let Some(ref history_item) = state.read().viewing_history_item {
                                        history_item.per_task_logs.clone()
                                    } else {
                                        state.read().per_task_logs.clone()
                                    },
                                    tasks: if let Some(ref history_item) = state.read().viewing_history_item {
                                        history_item.tasks.clone()
                                    } else {
                                        state.read().tasks.clone()
                                    },
                                    current_step: state.read().current_step,
                                    show_logs: state.read().show_logs,
                                    on_toggle: move |_| { let current = state.read().show_logs; state.write().show_logs = !current; },
                                    on_copy: copy_to_clipboard
                                }
                                if let Some(ref result) = state.read().workflow_result {
                                    ContextPanel { context: result.final_context.clone(), show_context: state.read().show_context, on_toggle: move |_| { let current = state.read().show_context; state.write().show_context = !current; }, on_copy: move |_| { println!("Copy context to clipboard"); } }
                                }
                                if let Some(ref result) = state.read().workflow_result {
                                    ErrorsPanel { errors: result.errors.clone() }
                                } else if let Some(ref history_item) = state.read().viewing_history_item {
                                    ErrorsPanel { errors: history_item.errors.clone() }
                                }
                            }
                        }
                    }
                }
            }
            
            if *show_settings.read() {
                SettingsScreen {
                    settings: settings,
                    on_theme_change: move |theme| change_theme(theme),
                    on_close: move |_| close_settings(),
                }
            }
        }
    }
}
