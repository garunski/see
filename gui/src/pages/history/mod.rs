use crate::router::Route;
use crate::state::AppStateProvider;
use dioxus::prelude::*;
use dioxus_router::prelude::Link;
use see_core::{WorkflowExecutionSummary, WorkflowMetadata};
use std::time::{Duration, SystemTime};

#[component]
fn HistoryItem(
    execution: WorkflowExecutionSummary,
    on_delete_execution: EventHandler<String>,
) -> Element {
    let execution_id_for_delete = execution.id.clone();
    rsx! {
        div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 shadow-sm hover:bg-zinc-50 dark:hover:bg-zinc-700 transition-colors",
            div { class: "flex items-center justify-between p-6",
                Link {
                    to: Route::WorkflowDetailsPage { id: execution.id.clone() },
                    class: "flex-1 min-w-0 cursor-pointer",
                    div { class: "flex items-center gap-4 mb-3",
                        h4 { class: "text-base font-semibold text-zinc-900 dark:text-white truncate", "{execution.workflow_name}" }
                        div {
                            class: format!("px-3 py-1 text-sm rounded-full font-medium {}",
                                if execution.success {
                                    "bg-emerald-100 text-emerald-800 dark:bg-emerald-900 dark:text-emerald-200"
                                } else {
                                    "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200"
                                }
                            ),
                            if execution.success { "Success" } else { "Failed" }
                        }
                    }
                    div { class: "text-sm text-zinc-500 dark:text-zinc-400 mb-2",
                        "Executed: {execution.timestamp}"
                    }
                    div { class: "text-sm text-zinc-500 dark:text-zinc-400",
                        "{execution.task_count} tasks completed"
                    }
                }
                button {
                    class: "ml-4 p-2 text-zinc-400 hover:text-red-600 dark:hover:text-red-400 transition-colors rounded-lg hover:bg-red-50 dark:hover:bg-red-900/20",
                    onclick: move |_| {
                        on_delete_execution.call(execution_id_for_delete.clone());
                    },
                    // TrashIcon SVG
                    svg {
                        class: "w-5 h-5",
                        view_box: "0 0 20 20",
                        fill: "currentColor",
                        path { d: "M6.28 5.22a.75.75 0 00-1.06 1.06L8.94 10l-3.72 3.72a.75.75 0 101.06 1.06L10 11.06l3.72 3.72a.75.75 0 101.06-1.06L11.06 10l3.72-3.72a.75.75 0 00-1.06-1.06L10 8.94 6.28 5.22z" }
                    }
                }
            }
        }
    }
}

#[component]
fn RunningWorkflowItem(
    workflow: WorkflowMetadata,
    on_delete_workflow: EventHandler<String>,
) -> Element {
    let workflow_id_for_delete = workflow.id.clone();
    rsx! {
        div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 shadow-sm hover:bg-zinc-50 dark:hover:bg-zinc-700 transition-colors",
            div { class: "flex items-center justify-between p-6",
                Link {
                    to: Route::WorkflowDetailsPage { id: workflow.id.clone() },
                    class: "flex-1 min-w-0 cursor-pointer",
                    div { class: "flex items-center gap-4 mb-3",
                        h4 { class: "text-base font-semibold text-zinc-900 dark:text-white truncate", "{workflow.workflow_name}" }
                        div {
                            class: "px-3 py-1 text-sm rounded-full font-medium bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
                            "In Progress"
                        }
                    }
                    div { class: "text-sm text-zinc-500 dark:text-zinc-400 mb-2",
                        "Started: {workflow.start_timestamp}"
                    }
                    div { class: "text-sm text-zinc-500 dark:text-zinc-400",
                        "{workflow.task_ids.len()} tasks"
                    }
                }
                div { class: "ml-4 flex items-center gap-2",
                    // Spinning loading icon
                    svg {
                        class: "w-5 h-5 text-blue-600 dark:text-blue-400 animate-spin",
                        view_box: "0 0 20 20",
                        fill: "currentColor",
                        path { d: "M10 2a8 8 0 100 16 8 8 0 000-16zM8.5 10a1.5 1.5 0 113 0 1.5 1.5 0 01-3 0z" }
                    }
                    button {
                        class: "p-2 text-zinc-400 hover:text-red-600 dark:hover:text-red-400 transition-colors rounded-lg hover:bg-red-50 dark:hover:bg-red-900/20",
                        onclick: move |_| {
                            on_delete_workflow.call(workflow_id_for_delete.clone());
                        },
                        // TrashIcon SVG
                        svg {
                            class: "w-5 h-5",
                            view_box: "0 0 20 20",
                            fill: "currentColor",
                            path { d: "M6.28 5.22a.75.75 0 00-1.06 1.06L8.94 10l-3.72 3.72a.75.75 0 101.06 1.06L10 11.06l3.72 3.72a.75.75 0 101.06-1.06L11.06 10l3.72-3.72a.75.75 0 00-1.06-1.06L10 8.94 6.28 5.22z" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn LoadingSkeleton() -> Element {
    rsx! {
        div { class: "space-y-4",
            for _ in 0..3 {
                div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 p-6 animate-pulse",
                    div { class: "flex items-center justify-between",
                        div { class: "flex-1 min-w-0",
                            div { class: "flex items-center gap-4 mb-3",
                                div { class: "h-4 bg-zinc-200 dark:bg-zinc-700 rounded w-48" }
                                div { class: "h-6 bg-zinc-200 dark:bg-zinc-700 rounded-full w-16" }
                            }
                            div { class: "h-3 bg-zinc-200 dark:bg-zinc-700 rounded w-32 mb-2" }
                            div { class: "h-3 bg-zinc-200 dark:bg-zinc-700 rounded w-24" }
                        }
                        div { class: "h-8 w-8 bg-zinc-200 dark:bg-zinc-700 rounded" }
                    }
                }
            }
        }
    }
}

#[component]
fn ErrorBanner(error: String, on_retry: EventHandler<()>) -> Element {
    rsx! {
        div { class: "bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 mb-6",
            div { class: "flex items-center justify-between",
                div { class: "flex items-center gap-3",
                    svg {
                        class: "w-5 h-5 text-red-600 dark:text-red-400 flex-shrink-0",
                        view_box: "0 0 20 20",
                        fill: "currentColor",
                        path { d: "M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" }
                    }
                    div {
                        h3 { class: "text-sm font-medium text-red-800 dark:text-red-200", "Failed to load history" }
                        p { class: "text-sm text-red-700 dark:text-red-300 mt-1", "{error}" }
                    }
                }
                button {
                    class: "px-3 py-1 text-sm font-medium text-red-800 dark:text-red-200 bg-red-100 dark:bg-red-900/30 hover:bg-red-200 dark:hover:bg-red-900/50 rounded-md transition-colors",
                    onclick: move |_| on_retry.call(()),
                    "Retry"
                }
            }
        }
    }
}

#[component]
pub fn HistoryPage() -> Element {
    let state_provider = use_context::<AppStateProvider>();
    // Store is now managed internally by core

    // Local state for this page
    let is_loading = use_signal(|| true);
    let error = use_signal(|| None::<String>);
    let last_updated = use_signal(|| None::<SystemTime>);
    let is_manual_refresh = use_signal(|| false);

    // Get current data from state
    let workflow_history = use_memo(move || state_provider.history.read().workflow_history.clone());
    let running_workflows =
        use_memo(move || state_provider.history.read().running_workflows.clone());

    // Manual refresh function
    let refresh_data = {
        let state_provider = state_provider.clone();

        move || {
            let mut state_provider = state_provider.clone();
            let mut is_loading = is_loading;
            let mut error = error;
            let mut last_updated = last_updated;
            let mut is_manual_refresh = is_manual_refresh;

            spawn(async move {
                is_loading.set(true);
                error.set(None);
                is_manual_refresh.set(true);

                match see_core::get_global_store() {
                    Ok(store) => {
                        // Load completed workflows
                        match store.list_workflow_executions(50).await {
                            Ok(history) => {
                                state_provider.history.write().set_history(history);
                            }
                            Err(e) => {
                                error.set(Some(format!(
                                    "Failed to load completed workflows: {}",
                                    e
                                )));
                            }
                        }

                        // Load running workflows
                        match store.list_workflow_metadata(50).await {
                            Ok(metadata) => {
                                // Filter to only running workflows
                                let running: Vec<_> = metadata
                                    .into_iter()
                                    .filter(|m| {
                                        m.status
                                            == see_core::persistence::models::WorkflowStatus::Running
                                    })
                                    .collect();
                                state_provider
                                    .history
                                    .write()
                                    .set_running_workflows(running);
                            }
                            Err(e) => {
                                error.set(Some(format!("Failed to load running workflows: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        error.set(Some(format!("Database not available: {}", e)));
                    }
                }

                is_loading.set(false);
                last_updated.set(Some(SystemTime::now()));
                is_manual_refresh.set(false);
            });
        }
    };

    // Clone refresh_data for multiple uses
    let refresh_data_1 = refresh_data.clone();
    let refresh_data_2 = refresh_data.clone();
    let refresh_data_3 = refresh_data.clone();
    let refresh_data_4 = refresh_data.clone();

    // Initial load on mount
    use_effect(move || {
        refresh_data_1();
    });

    // Polling effect - only run once on mount
    let mut cancel_tx_ref = use_signal(|| None::<tokio::sync::oneshot::Sender<()>>);
    use_effect(move || {
        // Start new polling loop
        let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel();
        *cancel_tx_ref.write() = Some(cancel_tx);

        let refresh_data = refresh_data_2.clone();
        spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(3));
            // Skip the first tick to avoid immediate execution
            interval.tick().await;

            loop {
                // Check for cancellation first
                if cancel_rx.try_recv().is_ok() {
                    break;
                }

                interval.tick().await;
                refresh_data();
            }
        });

        // Cleanup on unmount
        if let Some(cancel_tx) = cancel_tx_ref.write().take() {
            let _ = cancel_tx.send(());
        }
    });

    // Delete execution handler
    let delete_execution = {
        let state_provider = state_provider.clone();
        move |id: String| {
            let mut history_state = state_provider.history;
            spawn(async move {
                match see_core::get_global_store() {
                    Ok(store) => match store.delete_workflow_execution(&id).await {
                        Ok(_) => {
                            history_state.write().delete_execution(&id);
                        }
                        Err(e) => {
                            eprintln!("Failed to delete execution: {}", e);
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to create store for deletion: {}", e);
                    }
                }
            });
        }
    };

    // Delete running workflow handler
    let delete_running_workflow = {
        let state_provider = state_provider.clone();
        move |id: String| {
            let mut history_state = state_provider.history;
            spawn(async move {
                match see_core::get_global_store() {
                    Ok(store) => match store.delete_workflow_metadata_and_tasks(&id).await {
                        Ok(_) => {
                            history_state.write().remove_running_workflow(&id);
                        }
                        Err(e) => {
                            eprintln!("Failed to delete running workflow: {}", e);
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to create store for deletion: {}", e);
                    }
                }
            });
        }
    };

    // Format last updated time
    let last_updated_text = use_memo(move || {
        if let Some(time) = last_updated() {
            let elapsed = time.elapsed().unwrap_or(Duration::from_secs(0));
            if elapsed.as_secs() < 60 {
                format!("{} seconds ago", elapsed.as_secs())
            } else if elapsed.as_secs() < 3600 {
                format!("{} minutes ago", elapsed.as_secs() / 60)
            } else {
                format!("{} hours ago", elapsed.as_secs() / 3600)
            }
        } else {
            "Never".to_string()
        }
    });

    rsx! {
        div { class: "space-y-8",
            // Header with refresh button and last updated
            div { class: "flex items-center justify-between",
                div {
                    h1 { class: "text-xl font-bold text-zinc-900 dark:text-white", "Workflow History" }
                    p { class: "mt-2 text-zinc-600 dark:text-zinc-400", "View and manage your previous workflow executions" }
                }
                div { class: "flex items-center gap-4",
                    if let Some(_) = last_updated() {
                        div { class: "text-sm text-zinc-500 dark:text-zinc-400",
                            "Last updated: {last_updated_text()}"
                        }
                    }
                    button {
                        class: "flex items-center gap-2 px-3 py-2 text-zinc-600 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-white transition-colors rounded-lg hover:bg-zinc-100 dark:hover:bg-zinc-800",
                        onclick: move |_| refresh_data_4(),
                        disabled: is_loading() || is_manual_refresh(),
                        svg {
                            class: format!("w-4 h-4 {}", if is_manual_refresh() { "animate-spin" } else { "" }),
                            view_box: "0 0 20 20",
                            fill: "currentColor",
                            path { d: "M10 2a8 8 0 100 16 8 8 0 000-16zM8.5 10a1.5 1.5 0 113 0 1.5 1.5 0 01-3 0z" }
                        }
                        span { "Refresh" }
                    }
                }
            }

            // Error banner
            if let Some(err) = error() {
                ErrorBanner {
                    error: err,
                    on_retry: move |_| refresh_data_3()
                }
            }

            // Loading state
            if is_loading() && workflow_history().is_empty() && running_workflows().is_empty() {
                LoadingSkeleton {}
            } else {
                // Running Workflows
                if !running_workflows().is_empty() {
                    div { class: "space-y-4",
                        div { class: "flex items-center justify-between",
                            h2 { class: "text-lg font-semibold text-zinc-900 dark:text-white", "Running Workflows" }
                            div { class: "flex items-center gap-2",
                                div { class: "w-2 h-2 bg-blue-500 rounded-full animate-pulse" }
                                span { class: "text-sm text-zinc-500 dark:text-zinc-400", "Live updates" }
                            }
                        }
                        div { class: "grid gap-6",
                            for workflow in running_workflows().iter() {
                                RunningWorkflowItem {
                                    workflow: workflow.clone(),
                                    on_delete_workflow: delete_running_workflow.clone(),
                                }
                            }
                        }
                    }
                }

                // Completed Workflows
                div { class: "space-y-4",
                    div { class: "flex items-center justify-between",
                        h2 { class: "text-lg font-semibold text-zinc-900 dark:text-white", "Completed Workflows" }
                    }
                    if workflow_history().is_empty() {
                        div { class: "bg-white dark:bg-zinc-800 rounded-xl border border-zinc-200 dark:border-zinc-700 p-16 text-center shadow-sm",
                            div { class: "text-6xl mb-6", "📋" }
                            h3 { class: "text-base font-semibold text-zinc-900 dark:text-white mb-3", "No completed workflows yet" }
                            p { class: "text-zinc-500 dark:text-zinc-400", "Execute your first workflow to see it appear here" }
                        }
                    } else {
                        div { class: "grid gap-6",
                            for execution in workflow_history().iter() {
                                HistoryItem {
                                    execution: execution.clone(),
                                    on_delete_execution: delete_execution.clone(),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
