use crate::queries::use_create_workflow_mutation;
use dioxus::prelude::*;
use dioxus_query_custom::prelude::MutationState;

pub struct UploadWorkflowMutation {
    pub state: Signal<MutationState<()>>,
    pub upload_fn: std::rc::Rc<dyn Fn(String)>,
}

pub fn use_upload_workflow() -> UploadWorkflowMutation {
    let (state, upload_fn) = use_create_workflow_mutation();

    UploadWorkflowMutation {
        state,
        upload_fn: std::rc::Rc::new(upload_fn),
    }
}
