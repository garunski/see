use crate::services::execution::ExecutionService;
use dioxus::prelude::Signal;
use s_e_e_core::{TaskExecution, WorkflowExecution, WorkflowExecutionSummary, WorkflowMetadata};
use s_e_e_dioxus_query::prelude::*;
use std::rc::Rc;

pub fn use_workflow_executions_query() -> (QueryState<Vec<WorkflowExecutionSummary>>, impl Fn()) {
    let key = QueryKey::new(&["executions", "list"]);

    let fetcher = move || async move {
        ExecutionService::fetch_workflow_executions(100)
            .await
            .map_err(|e| e.to_string())
    };

    let options = QueryOptions {
        stale_time: Some(0),
        cache_time: Some(60_000),
        refetch_interval: Some(1000),
        ..Default::default()
    };

    use_query(key, fetcher, options)
}

pub fn use_running_workflows_query() -> (QueryState<Vec<WorkflowMetadata>>, impl Fn()) {
    let key = QueryKey::new(&["workflows", "running"]);

    let fetcher = move || async move {
        ExecutionService::fetch_running_workflows(100)
            .await
            .map_err(|e| e.to_string())
    };

    let options = QueryOptions {
        stale_time: Some(0),
        cache_time: Some(60_000),
        refetch_interval: Some(1000),
        ..Default::default()
    };

    use_query(key, fetcher, options)
}

pub fn use_workflow_execution_query(
    execution_id: String,
) -> (QueryState<WorkflowExecution>, impl Fn()) {
    let key = QueryKey::new(&["executions", "detail", &execution_id]);

    let id_clone = execution_id.clone();
    let fetcher = move || {
        let id = id_clone.clone();
        async move {
            ExecutionService::fetch_workflow_execution(&id)
                .await
                .map_err(|e| e.to_string())
        }
    };

    let options = QueryOptions {
        stale_time: Some(5_000),
        cache_time: Some(300_000),
        refetch_interval: Some(2000),
        ..Default::default()
    };

    use_query(key, fetcher, options)
}

pub fn use_task_details_query(
    execution_id: String,
    task_id: String,
) -> (QueryState<Option<TaskExecution>>, impl Fn()) {
    let key = QueryKey::new(&["tasks", "detail", &execution_id, &task_id]);

    let exec_id = execution_id.clone();
    let t_id = task_id.clone();
    let fetcher = move || {
        let execution_id = exec_id.clone();
        let task_id = t_id.clone();
        async move {
            ExecutionService::fetch_task_details(&execution_id, &task_id)
                .await
                .map_err(|e| e.to_string())
        }
    };

    let options = QueryOptions {
        stale_time: Some(0),
        cache_time: Some(60_000),
        ..Default::default()
    };

    use_query(key, fetcher, options)
}

pub fn use_delete_execution_mutation() -> (Signal<MutationState<()>>, impl Fn(String)) {
    let mutation_fn = move |execution_id: String| async move {
        ExecutionService::delete_workflow_execution(&execution_id)
            .await
            .map_err(|e| e.to_string())
    };

    let callbacks = MutationCallbacks {
        on_success: None,
        on_error: None,
        on_settled: Some(Rc::new(|| {
            invalidate_queries_by_prefix("executions:");
        })),
        invalidate_keys: vec![QueryKey::new(&["executions", "list"])],
        optimistic_update: None,
    };

    use_mutation(mutation_fn, callbacks)
}
