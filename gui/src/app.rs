use crate::components::{
    ContextPanel, ErrorsPanel, OutputLogsPanel, SettingsScreen, Sidebar, Toast, WorkflowInfoCard,
};
use crate::services::workflow::{create_output_channel, run_workflow};
use crate::state::{AppStateProvider, SidebarTab};
use dioxus::prelude::*;
use rfd::FileDialog;
use see_core::{AppSettings, AuditStore, RedbStore, Theme};
use std::sync::Arc;

#[component]
pub fn App() -> Element {
    // 1. Initialize store and provide as context with proper error handling
    let store = use_hook(|| match RedbStore::new_default() {
        Ok(store) => Some(Arc::new(store)),
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            None
        }
    });
    use_context_provider(|| store.clone());

    // 2. Create state provider with separated state
    let mut state_provider = use_hook(AppStateProvider::new);
    use_context_provider(|| state_provider.clone());

    // 3. Show database initialization error notification if needed
    let store_clone_for_notification = store.clone();
    use_effect(move || {
        if store_clone_for_notification.is_none() {
            state_provider.ui.write().show_toast(
                "⚠️ Database unavailable - workflow history and settings will not be saved"
                    .to_string(),
            );
        }
    });

    // 3. Load settings once on mount
    let store_clone = store.clone();
    use_effect(move || {
        let store = store_clone.clone();
        let mut settings = state_provider.settings;
        spawn(async move {
            if let Some(ref s) = store {
                if let Ok(Some(loaded)) = s.load_settings().await {
                    settings.write().apply_loaded_settings(loaded);
                }
            }
        });
    });

    // 4. Load history - reactive to needs_history_reload flag
    let store_clone2 = store.clone();
    use_effect(move || {
        let needs_reload = state_provider.history.read().needs_history_reload;
        if needs_reload {
            let store = store_clone2.clone();
            let mut history_state = state_provider.history;
            spawn(async move {
                if let Some(s) = store {
                    match s.list_workflow_executions(50).await {
                        Ok(history) => {
                            history_state.write().set_history(history);
                        }
                        Err(e) => {
                            state_provider
                                .ui
                                .write()
                                .show_toast(format!("Failed to load history: {}", e));
                        }
                    }
                }
            });
        }
    });

    // 5. Compute dark mode from settings (for UI classes)
    let dark_mode_signal = use_memo(
        move || match state_provider.settings.read().settings.theme {
            Theme::Dark => true,
            Theme::Light => false,
            Theme::System => matches!(dark_light::detect(), dark_light::Mode::Dark),
        },
    );

    // 6. Settings modal state
    let mut show_settings = use_signal(|| false);

    // 7. Settings handlers
    let mut open_settings = move || {
        show_settings.set(true);
    };

    let mut close_settings = move || {
        show_settings.set(false);
    };

    let store_clone_theme = store.clone();
    let change_theme = move |new_theme: Theme| {
        state_provider.settings.write().change_theme(new_theme);

        // Save immediately with error handling
        let store_clone = store_clone_theme.clone();
        let mut ui_state = state_provider.ui;
        spawn(async move {
            if let Some(ref s) = store_clone {
                match s.save_settings(&AppSettings { theme: new_theme }).await {
                    Ok(_) => {
                        ui_state
                            .write()
                            .show_toast("Settings saved successfully".to_string());
                    }
                    Err(e) => {
                        ui_state
                            .write()
                            .show_toast(format!("Failed to save settings: {}", e));
                    }
                }
            }
        });
    };

    let workflow_result_signal =
        use_memo(move || state_provider.workflow.read().workflow_result.clone());
    let current_step_signal = use_memo(move || state_provider.workflow.read().current_step);

    let mut on_next_step = move || {
        let current = state_provider.workflow.read().current_step;
        let total =
            if let Some(ref history_item) = state_provider.history.read().viewing_history_item {
                history_item.tasks.len()
            } else {
                state_provider.workflow.read().tasks.len()
            };
        if current < total.saturating_sub(1) {
            state_provider.workflow.write().current_step = current + 1;
        }
    };

    let mut on_prev_step = move || {
        let current = state_provider.workflow.read().current_step;
        if current > 0 {
            state_provider.workflow.write().current_step = current - 1;
        }
    };

    let on_jump_to_step = move |step: usize| {
        let total =
            if let Some(ref history_item) = state_provider.history.read().viewing_history_item {
                history_item.tasks.len()
            } else {
                state_provider.workflow.read().tasks.len()
            };
        if step < total {
            state_provider.workflow.write().current_step = step;
        }
    };

    let mut pick_file = move || {
        state_provider.ui.write().set_picking_file(true);
        spawn(async move {
            if let Some(path) = FileDialog::new()
                .add_filter("JSON files", &["json"])
                .set_title("Select Workflow File")
                .pick_file()
            {
                if let Some(path_str) = path.to_str() {
                    state_provider.workflow.write().workflow_file = path_str.to_string();
                    // Clear viewing history item when picking a new file
                    state_provider.history.write().clear_viewing();
                }
            }
            state_provider.ui.write().set_picking_file(false);
        });
    };

    let copy_to_clipboard = move |text: String| {
        println!("Copy to clipboard: {}", text);
    };

    let mut dismiss_toast = move || {
        state_provider.ui.write().dismiss_toast();
    };

    let switch_tab = move |tab: SidebarTab| {
        let is_upload_tab = matches!(tab, SidebarTab::Upload);
        state_provider.ui.write().switch_tab(tab);
        // Clear viewing history item when switching to upload tab
        if is_upload_tab {
            state_provider.history.write().clear_viewing();
        }
    };

    let store_clone3 = store.clone();
    let load_execution = move |id: String| {
        let store_clone = store_clone3.clone();
        let mut history_state = state_provider.history;
        let mut ui_state = state_provider.ui;
        spawn(async move {
            if let Some(s) = store_clone {
                match s.get_workflow_execution(&id).await {
                    Ok(execution) => {
                        history_state.write().load_execution(execution, id);
                    }
                    Err(e) => {
                        ui_state
                            .write()
                            .show_toast(format!("Failed to load execution: {}", e));
                    }
                }
            }
        });
    };

    let store_clone4 = store.clone();
    let delete_execution = move |id: String| {
        let store_clone = store_clone4.clone();
        let mut history_state = state_provider.history;
        let mut ui_state = state_provider.ui;
        spawn(async move {
            if let Some(s) = store_clone {
                match s.delete_workflow_execution(&id).await {
                    Ok(_) => {
                        history_state.write().delete_execution(&id);
                        ui_state
                            .write()
                            .show_toast("Workflow execution deleted".to_string());
                    }
                    Err(e) => {
                        ui_state
                            .write()
                            .show_toast(format!("Failed to delete execution: {}", e));
                    }
                }
            }
        });
    };

    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }
        div {
            class: format!("min-h-screen bg-white dark:bg-zinc-900 text-zinc-950 dark:text-white {}", if *dark_mode_signal.read() { "dark" } else { "" }),
            onkeydown: move |evt| { match evt.key() { dioxus::events::Key::ArrowLeft | dioxus::events::Key::ArrowUp => on_prev_step(), dioxus::events::Key::ArrowRight | dioxus::events::Key::ArrowDown => on_next_step(), _ => {} } },

            Toast { message: state_provider.ui.read().toast_message.clone(), on_dismiss: move |_| dismiss_toast() }

            div { class: "flex flex-col lg:flex-row min-h-svh w-full bg-white lg:bg-zinc-100 dark:bg-zinc-900 dark:lg:bg-zinc-950",
                Sidebar {
                    workflow_file: state_provider.workflow.read().workflow_file.clone(),
                    on_workflow_file_change: move |value| {
                        state_provider.workflow.write().workflow_file = value;
                        // Clear viewing history item when selecting a new workflow file
                        state_provider.history.write().clear_viewing();
                    },
                    is_picking_file: state_provider.ui.read().is_picking_file,
                    on_pick_file: move |_| pick_file(),
                    on_open_settings: move |_| open_settings(),
                    execution_status: state_provider.workflow.read().execution_status.clone(),
                    on_execute: move |_| {
                        let store_clone5 = store.clone();
                        let mut workflow_state = state_provider.workflow;
                        let mut ui_state = state_provider.ui;
                        let mut history_state = state_provider.history;
                        spawn(async move {
                            let file_path = workflow_state.read().workflow_file.clone();
                            workflow_state.write().reset_before_run();

                            let (output_callback, mut handles) = create_output_channel();

                            let mut workflow_state_clone = workflow_state;
                            spawn(async move {
                                while let Some(msg) = handles.receiver.recv().await {
                                    workflow_state_clone.write().output_logs.push(msg);
                                }
                            });

                            match run_workflow(file_path, output_callback, store_clone5.map(|s| s as Arc<dyn AuditStore>)).await {
                                Ok(result) => {
                                    workflow_state.write().apply_success(&result);
                                    ui_state.write().show_toast("Workflow completed successfully!".to_string());
                                    history_state.write().needs_history_reload = true;
                                }
                                Err(e) => {
                                    workflow_state.write().apply_failure(&e.to_string());
                                    ui_state.write().show_toast(format!("Workflow failed: {}", e));
                                }
                            }
                        });
                    },
                    is_viewing_history: state_provider.history.read().viewing_history_item.is_some(),
                    sidebar_tab: state_provider.ui.read().sidebar_tab.clone(),
                    on_tab_change: switch_tab,
                    workflow_history: state_provider.history.read().workflow_history.clone(),
                    on_load_execution: load_execution,
                    on_delete_execution: delete_execution,
                    selected_history_id: state_provider.history.read().selected_history_id.clone(),
                }

                main { class: "flex flex-1 flex-col pb-2 lg:min-w-0 lg:pt-2 lg:pr-2 lg:ml-64",
                    div { class: "grow p-6 lg:rounded-lg lg:bg-white lg:p-10 lg:shadow-xs lg:ring-1 lg:ring-zinc-950/5 dark:lg:bg-zinc-900 dark:lg:ring-white/10",
                        div { class: "mx-auto max-w-6xl",
                            if let Some(ref history_item) = state_provider.history.read().viewing_history_item {
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
                                    current_step: current_step_signal(),
                                    on_next_step: on_next_step,
                                    on_prev_step: on_prev_step,
                                    on_jump_to_step: on_jump_to_step
                                }
                            } else if let Some(result) = workflow_result_signal.read().clone() {
                                WorkflowInfoCard {
                                    result: ReadOnlySignal::new(Signal::new(result)),
                                    tasks: state_provider.workflow.read().tasks.clone(),
                                    current_step: current_step_signal(),
                                    on_next_step: on_next_step,
                                    on_prev_step: on_prev_step,
                                    on_jump_to_step: on_jump_to_step
                                }
                            }
                            div { class: "space-y-6",
                                OutputLogsPanel {
                                    per_task_logs: if let Some(ref history_item) = state_provider.history.read().viewing_history_item {
                                        history_item.per_task_logs.clone()
                                    } else {
                                        state_provider.workflow.read().per_task_logs.clone()
                                    },
                                    tasks: if let Some(ref history_item) = state_provider.history.read().viewing_history_item {
                                        history_item.tasks.clone()
                                    } else {
                                        state_provider.workflow.read().tasks.clone()
                                    },
                                    current_step: current_step_signal(),
                                    show_logs: state_provider.ui.read().show_logs,
                                    on_toggle: move |_| {
                                        let current = state_provider.ui.read().show_logs;
                                        state_provider.ui.write().show_logs = !current;
                                    },
                                    on_copy: copy_to_clipboard
                                }
                                if let Some(ref result) = state_provider.workflow.read().workflow_result {
                                    ContextPanel {
                                        context: result.final_context.clone(),
                                        show_context: state_provider.ui.read().show_context,
                                        on_toggle: move |_| {
                                            let current = state_provider.ui.read().show_context;
                                            state_provider.ui.write().show_context = !current;
                                        },
                                        on_copy: move |_| { println!("Copy context to clipboard"); }
                                    }
                                }
                                if let Some(ref result) = state_provider.workflow.read().workflow_result {
                                    ErrorsPanel { errors: result.errors.clone() }
                                } else if let Some(ref history_item) = state_provider.history.read().viewing_history_item {
                                    ErrorsPanel { errors: history_item.errors.clone() }
                                }
                            }
                        }
                    }
                }
            }

            if *show_settings.read() {
                SettingsScreen {
                    settings: state_provider.settings,
                    on_theme_change: change_theme,
                    on_close: move |_| close_settings(),
                }
            }
        }
    }
}
