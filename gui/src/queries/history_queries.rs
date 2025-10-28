use crate::services::history::HistoryService;
use dioxus_query::prelude::*;
use s_e_e_core::{WorkflowExecutionSummary, WorkflowMetadata};

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct GetWorkflowHistory;

impl QueryCapability for GetWorkflowHistory {
    type Ok = Vec<WorkflowExecutionSummary>;
    type Err = String;
    type Keys = ();

    async fn run(&self, _: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        HistoryService::fetch_workflow_executions(100) // Reasonable limit
            .await
            .map_err(|e| e.to_string())
    }
}

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct GetRunningWorkflows;

impl QueryCapability for GetRunningWorkflows {
    type Ok = Vec<WorkflowMetadata>;
    type Err = String;
    type Keys = ();

    async fn run(&self, _: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        HistoryService::fetch_running_workflows(100) // Reasonable limit
            .await
            .map_err(|e| e.to_string())
    }
}
