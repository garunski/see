use crate::services::workflow::WorkflowService;
use dioxus_query::prelude::*;
use s_e_e_core::{WorkflowDefinition, WorkflowResult};

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct GetWorkflows;

impl QueryCapability for GetWorkflows {
    type Ok = Vec<WorkflowDefinition>;
    type Err = String;
    type Keys = ();

    async fn run(&self, _: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        WorkflowService::fetch_workflows()
            .await
            .map_err(|e| e.to_string())
    }
}

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct GetWorkflow;

impl QueryCapability for GetWorkflow {
    type Ok = Option<WorkflowDefinition>;
    type Err = String;
    type Keys = String;

    async fn run(&self, id: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        WorkflowService::fetch_workflow(id)
            .await
            .map_err(|e| e.to_string())
    }
}

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct CreateWorkflowMutation;

impl MutationCapability for CreateWorkflowMutation {
    type Ok = ();
    type Err = String;
    type Keys = String; // We'll pass JSON string instead

    async fn run(&self, json: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        let workflow: WorkflowDefinition =
            serde_json::from_str(json).map_err(|e| format!("Invalid workflow JSON: {}", e))?;

        WorkflowService::create_workflow(workflow)
            .await
            .map_err(|e| e.to_string())
    }

    async fn on_settled(&self, json: &Self::Keys, _: &Result<Self::Ok, Self::Err>) {
        if let Ok(workflow) = serde_json::from_str::<WorkflowDefinition>(json) {
            QueriesStorage::<GetWorkflows>::invalidate_matching(()).await;
            QueriesStorage::<GetWorkflow>::invalidate_matching(workflow.id.clone()).await;
        }
    }
}

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct UpdateWorkflowMutation;

impl MutationCapability for UpdateWorkflowMutation {
    type Ok = ();
    type Err = String;
    type Keys = String; // We'll pass JSON string instead

    async fn run(&self, json: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        let workflow: WorkflowDefinition =
            serde_json::from_str(json).map_err(|e| format!("Invalid workflow JSON: {}", e))?;

        WorkflowService::update_workflow(workflow)
            .await
            .map_err(|e| e.to_string())
    }

    async fn on_settled(&self, json: &Self::Keys, _: &Result<Self::Ok, Self::Err>) {
        if let Ok(workflow) = serde_json::from_str::<WorkflowDefinition>(json) {
            QueriesStorage::<GetWorkflows>::invalidate_matching(()).await;
            QueriesStorage::<GetWorkflow>::invalidate_matching(workflow.id.clone()).await;
        }
    }
}

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct DeleteWorkflowMutation;

impl MutationCapability for DeleteWorkflowMutation {
    type Ok = ();
    type Err = String;
    type Keys = String;

    async fn run(&self, id: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        WorkflowService::delete_workflow(id)
            .await
            .map_err(|e| e.to_string())
    }

    async fn on_settled(&self, id: &Self::Keys, _: &Result<Self::Ok, Self::Err>) {
        QueriesStorage::<GetWorkflows>::invalidate_matching(()).await;
        QueriesStorage::<GetWorkflow>::invalidate_matching(id.clone()).await;
    }
}

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct ExecuteWorkflowMutation;

impl MutationCapability for ExecuteWorkflowMutation {
    type Ok = WorkflowResult;
    type Err = String;
    type Keys = String; // workflow_id

    async fn run(&self, workflow_id: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        use crate::services::workflow::run_workflow_by_id;

        run_workflow_by_id(workflow_id.clone(), None)
            .await
            .map_err(|e| e.to_string())
    }

    async fn on_settled(&self, _: &Self::Keys, result: &Result<Self::Ok, Self::Err>) {
        // Invalidate workflow history when execution completes
        if let Ok(_) = result {
            QueriesStorage::<crate::queries::GetWorkflowHistory>::invalidate_matching(()).await;
            QueriesStorage::<crate::queries::GetRunningWorkflows>::invalidate_matching(()).await;
        }
    }
}
