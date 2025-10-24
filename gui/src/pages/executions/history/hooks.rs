use crate::services::history::HistoryService;
use crate::state::AppStateProvider;
use dioxus::prelude::*;

pub fn use_history_data() -> (Signal<bool>, Signal<Option<String>>, impl Fn() + Clone) {
    let state_provider = use_context::<AppStateProvider>();
    let is_loading = use_signal(|| true);
    let error = use_signal(|| None::<String>);

    let refresh_data = {
        let state_provider = state_provider.clone();
        move || {
            tracing::info!("🔄 REFRESH DATA CALLED");
            let mut state_provider = state_provider.clone();
            let mut is_loading = is_loading;
            let mut error = error;

            spawn(async move {
                is_loading.set(true);
                error.set(None);

                match HistoryService::refresh_all(50).await {
                    Ok((executions, running)) => {
                        tracing::info!(
                            "🔄 HISTORY DATA REFRESH: executions_count={}, running_count={}, execution_ids={:?}, running_ids={:?}",
                            executions.len(),
                            running.len(),
                            executions.iter().map(|e| &e.id).collect::<Vec<_>>(),
                            running.iter().map(|r| &r.id).collect::<Vec<_>>()
                        );
                        state_provider.history.write().set_history(executions);
                        state_provider
                            .history
                            .write()
                            .set_running_workflows(running);
                    }
                    Err(e) => {
                        error.set(Some(format!("{:?}", e)));
                    }
                }

                is_loading.set(false);
            });
        }
    };

    (is_loading, error, refresh_data)
}

pub fn use_deletion_handlers() -> (impl Fn(String) + Clone, impl Fn(String) + Clone) {
    let state_provider = use_context::<AppStateProvider>();

    let delete_execution = {
        let state_provider = state_provider.clone();
        move |id: String| {
            let mut history_state = state_provider.history;
            spawn(async move {
                if let Err(e) = HistoryService::delete_execution(&id).await {
                    eprintln!("Failed to delete execution: {:?}", e);
                } else {
                    history_state.write().delete_execution(&id);
                }
            });
        }
    };

    let delete_running_workflow = {
        let state_provider = state_provider.clone();
        move |id: String| {
            let mut history_state = state_provider.history;
            spawn(async move {
                if let Err(e) = HistoryService::delete_running_workflow(&id).await {
                    eprintln!("Failed to delete running workflow: {:?}", e);
                } else {
                    history_state.write().remove_running_workflow(&id);
                }
            });
        }
    };

    (delete_execution, delete_running_workflow)
}
