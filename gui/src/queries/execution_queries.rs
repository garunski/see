use crate::services::execution::ExecutionService;
use dioxus_query::prelude::*;
use s_e_e_core::{TaskExecution, WorkflowExecution, WorkflowExecutionSummary, WorkflowMetadata};

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct GetWorkflowExecutions;

impl QueryCapability for GetWorkflowExecutions {
    type Ok = Vec<WorkflowExecutionSummary>;
    type Err = String;
    type Keys = ();

    async fn run(&self, _: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        ExecutionService::fetch_workflow_executions(100)
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
        ExecutionService::fetch_running_workflows(100)
            .await
            .map_err(|e| e.to_string())
    }
}

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct GetWorkflowExecution;

impl QueryCapability for GetWorkflowExecution {
    type Ok = WorkflowExecution;
    type Err = String;
    type Keys = String;

    async fn run(&self, execution_id: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        ExecutionService::fetch_workflow_execution(execution_id)
            .await
            .map_err(|e| e.to_string())
    }
}

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct GetTaskDetails;

impl QueryCapability for GetTaskDetails {
    type Ok = Option<TaskExecution>;
    type Err = String;
    type Keys = (String, String); // (execution_id, task_id)

    async fn run(&self, keys: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        let (execution_id, task_id) = keys;

        ExecutionService::fetch_task_details(execution_id, task_id)
            .await
            .map_err(|e| e.to_string())
    }
}
