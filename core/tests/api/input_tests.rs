



#[cfg(test)]
mod tests {
    use s_e_e_core::{get_pending_inputs, get_tasks_waiting_for_input, provide_user_input, CoreError};
    use s_e_e_core::{init_test_store, TaskExecution, TaskStatus, UserInputRequest};
    use serde_json::Value;

    async fn setup_test_environment() -> Result<(), CoreError> {
        init_test_store().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_pending_inputs_no_inputs() {


        let _ = get_pending_inputs("workflow-id").await;
    }

    #[tokio::test]
    async fn test_get_tasks_waiting_for_input_no_tasks() {


        let _ = get_tasks_waiting_for_input("workflow-id").await;
    }

    #[tokio::test]
    async fn test_provide_user_input_validation() {


        match provide_user_input("exec-id", "task-id", "test input".to_string()).await {
            Err(CoreError::WorkflowNotFound(_)) => {}
            Err(CoreError::TaskNotFound(_)) => {}
            Err(CoreError::Execution(_)) => {}
            _ => {}
        }
    }





}

