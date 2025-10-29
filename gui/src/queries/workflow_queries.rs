use crate::services::workflow::WorkflowService;
use dioxus::prelude::Signal;
use dioxus_query_custom::prelude::*;
use s_e_e_core::{WorkflowDefinition, WorkflowResult};
use std::rc::Rc;

// ==================== QUERIES ====================

/// Hook to fetch all workflows
pub fn use_workflows_query() -> (QueryState<Vec<WorkflowDefinition>>, impl Fn()) {
    let key = QueryKey::new(&["workflows", "list"]);

    let fetcher = move || async move {
        WorkflowService::fetch_workflows()
            .await
            .map_err(|e| e.to_string())
    };

    let options = QueryOptions {
        stale_time: Some(30_000),  // 30 seconds
        cache_time: Some(300_000), // 5 minutes
        ..Default::default()
    };

    use_query(key, fetcher, options)
}

/// Hook to fetch a single workflow
pub fn use_workflow_query(id: String) -> (QueryState<Option<WorkflowDefinition>>, impl Fn()) {
    let key = QueryKey::new(&["workflows", "detail", &id]);

    let id_clone = id.clone();
    let fetcher = move || {
        let id = id_clone.clone();
        async move {
            WorkflowService::fetch_workflow(&id)
                .await
                .map_err(|e| e.to_string())
        }
    };

    let options = QueryOptions {
        stale_time: Some(30_000),
        cache_time: Some(300_000),
        ..Default::default()
    };

    use_query(key, fetcher, options)
}

// ==================== MUTATIONS ====================

/// Hook to create a new workflow
pub fn use_create_workflow_mutation() -> (Signal<MutationState<()>>, impl Fn(String)) {
    let mutation_fn = move |json: String| async move {
        let workflow: WorkflowDefinition =
            serde_json::from_str(&json).map_err(|e| format!("Invalid workflow JSON: {}", e))?;

        WorkflowService::create_workflow(workflow)
            .await
            .map_err(|e| e.to_string())
    };

    let callbacks = MutationCallbacks {
        on_success: None,
        on_error: None,
        on_settled: Some(Rc::new(|| {
            invalidate_queries_by_prefix("workflows:");
        })),
        invalidate_keys: vec![QueryKey::new(&["workflows", "list"])],
        optimistic_update: None,
    };

    use_mutation(mutation_fn, callbacks)
}

/// Hook to execute a workflow
pub fn use_execute_workflow_mutation() -> (
    Signal<MutationState<WorkflowResult>>,
    std::rc::Rc<dyn Fn(String)>,
) {
    let mutation_fn = move |workflow_id: String| async move {
        tracing::debug!(
            "[ExecuteWorkflowMutation] Starting workflow execution for ID: {}",
            workflow_id
        );

        use s_e_e_core::execute_workflow_by_id;

        match execute_workflow_by_id(&workflow_id, None).await {
            Ok(result) => {
                tracing::info!(
                    "[ExecuteWorkflowMutation] Workflow executed successfully: {}",
                    result.workflow_name
                );
                Ok(result)
            }
            Err(e) => {
                tracing::error!(
                    "[ExecuteWorkflowMutation] Workflow execution failed: {:?}",
                    e
                );
                Err(format!("Workflow execution failed: {:?}", e))
            }
        }
    };

    let callbacks = MutationCallbacks {
        on_success: None,
        on_error: None,
        on_settled: None, // No invalidation - UI handles refreshing via polling
        invalidate_keys: vec![],
        optimistic_update: None,
    };

    let (state, mutate_fn) = use_mutation(mutation_fn, callbacks);
    (state, std::rc::Rc::new(mutate_fn))
}
