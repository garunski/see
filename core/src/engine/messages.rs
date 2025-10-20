use uuid::Uuid;

pub enum EngineMsg {
    RunWorkflow(String),
    CancelWorkflow(Uuid),
}

pub enum UiMsg {
    WorkflowStarted {
        id: Uuid,
        name: String,
    },
    TaskLog {
        id: Uuid,
        task: String,
        line: String,
    },
    WorkflowFinished {
        id: Uuid,
        result: Result<(), crate::errors::CoreError>,
    },
}
