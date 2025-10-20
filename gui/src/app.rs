use crate::components::{
    ContextPanel, ErrorsPanel, OutputLogsPanel, Sidebar, Toast, WorkflowInfoCard,
};
use crate::services::workflow::{create_output_channel, run_workflow};
use crate::state::{AppState, SidebarTab};
use dioxus::prelude::*;
use rfd::FileDialog;
use see_core::RedbStore;
use std::sync::Arc;

#[component]
pub fn App() -> Element {
    let mut state = use_signal(|| AppState::default());

    // Initialize RedbStore synchronously
    if state.read().store.is_none() {
        match RedbStore::new_default() {
            Ok(store) => {
                state.write().store = Some(Arc::new(store));
            }
            Err(e) => {
                state.write().toast_message = Some(format!("Failed to initialize database: {}", e));
            }
        }
    }

    let workflow_result_signal = use_memo(move || state.read().workflow_result.clone());
    let dark_mode_signal = use_memo(move || state.read().dark_mode);

    // Load initial history asynchronously
    use_effect(move || {
        spawn(async move {
            state.write().load_history().await;
        });
    });

    // Watch for history reload flag
    use_effect(move || {
        if state.read().needs_history_reload {
            spawn(async move {
                state.write().load_history().await;
                state.write().needs_history_reload = false;
            });
        }
    });

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

    let execute_workflow = move || {
        spawn(async move {
            let file_path = state.read().workflow_file.clone();
            state.write().reset_before_run();

            let (output_callback, mut handles) = create_output_channel();

            let mut state_clone = state;
            spawn(async move {
                while let Some(msg) = handles.receiver.recv().await {
                    state_clone.write().output_logs.push(msg);
                }
            });

            let store = state
                .read()
                .store
                .clone()
                .map(|s| s as Arc<dyn see_core::AuditStore>);
            match run_workflow(file_path, output_callback, store).await {
                Ok(result) => {
                    state.write().apply_success(&result);
                }
                Err(e) => {
                    state.write().apply_failure(&e.to_string());
                }
            }
        });
    };

    let mut toggle_dark_mode = move || {
        let current_mode = state.read().dark_mode;
        state.write().dark_mode = !current_mode;
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
        spawn(async move {
            state.write().load_execution(&id).await;
        });
    };

    let delete_execution = move |id: String| {
        spawn(async move {
            state.write().delete_execution(&id).await;
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
                    dark_mode: *dark_mode_signal.read(),
                    on_toggle_dark_mode: move |_| toggle_dark_mode(),
                    execution_status: state.read().execution_status.clone(),
                    on_execute: move |_| execute_workflow(),
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
        }
    }
}
