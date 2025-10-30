use super::Store;
use crate::logging::{
    log_db_operation_error, log_db_operation_start, log_db_operation_success, log_deserialization,
    log_serialization,
};
use crate::models::{enums::InputRequestStatus, UserInputRequest};
use chrono::Utc;
use sqlx::Row;

impl Store {
    pub async fn save_input_request(&self, request: &UserInputRequest) -> Result<(), String> {
        log_db_operation_start("save_input_request", "user_input_requests");

        let json_data = serde_json::to_string(request).map_err(|e| {
            log_db_operation_error("save_input_request", "user_input_requests", &e.to_string());
            format!("Serialization error: {}", e)
        })?;

        log_serialization("UserInputRequest", json_data.len());

        sqlx::query("INSERT OR REPLACE INTO user_input_requests (id, data) VALUES (?, ?)")
            .bind(&request.id)
            .bind(&json_data)
            .execute(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error("save_input_request", "user_input_requests", &e.to_string());
                format!("Database error: {}", e)
            })?;

        log_db_operation_success("save_input_request", "user_input_requests", 0);
        Ok(())
    }

    pub async fn get_input_request(&self, id: &str) -> Result<Option<UserInputRequest>, String> {
        log_db_operation_start("get_input_request", "user_input_requests");

        let row = sqlx::query("SELECT data FROM user_input_requests WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error("get_input_request", "user_input_requests", &e.to_string());
                format!("Database error: {}", e)
            })?;

        let Some(row) = row else {
            log_db_operation_success("get_input_request", "user_input_requests", 0);
            return Ok(None);
        };

        let json_data: String = row.get("data");
        log_deserialization("UserInputRequest", json_data.len());

        let request: UserInputRequest = serde_json::from_str(&json_data).map_err(|e| {
            log_db_operation_error("get_input_request", "user_input_requests", &e.to_string());
            format!("Deserialization error: {}", e)
        })?;

        log_db_operation_success("get_input_request", "user_input_requests", 0);
        Ok(Some(request))
    }

    pub async fn get_input_request_by_task(
        &self,
        task_id: &str,
    ) -> Result<Option<UserInputRequest>, String> {
        log_db_operation_start("get_input_request_by_task", "user_input_requests");

        let rows = sqlx::query("SELECT data FROM user_input_requests")
            .fetch_all(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error(
                    "get_input_request_by_task",
                    "user_input_requests",
                    &e.to_string(),
                );
                format!("Database error: {}", e)
            })?;

        for row in rows {
            let json_data: String = row.get("data");
            log_deserialization("UserInputRequest", json_data.len());

            let request: UserInputRequest = serde_json::from_str(&json_data).map_err(|e| {
                log_db_operation_error(
                    "get_input_request_by_task",
                    "user_input_requests",
                    &e.to_string(),
                );
                format!("Deserialization error: {}", e)
            })?;

            if request.task_execution_id == task_id {
                log_db_operation_success("get_input_request_by_task", "user_input_requests", 0);
                return Ok(Some(request));
            }
        }

        log_db_operation_success("get_input_request_by_task", "user_input_requests", 0);
        Ok(None)
    }

    pub async fn get_pending_inputs_for_workflow(
        &self,
        workflow_id: &str,
    ) -> Result<Vec<UserInputRequest>, String> {
        log_db_operation_start("get_pending_inputs_for_workflow", "user_input_requests");

        let rows = sqlx::query("SELECT data FROM user_input_requests")
            .fetch_all(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error(
                    "get_pending_inputs_for_workflow",
                    "user_input_requests",
                    &e.to_string(),
                );
                format!("Database error: {}", e)
            })?;

        let mut requests = Vec::new();
        for row in rows {
            let json_data: String = row.get("data");
            log_deserialization("UserInputRequest", json_data.len());

            let request: UserInputRequest = serde_json::from_str(&json_data).map_err(|e| {
                log_db_operation_error(
                    "get_pending_inputs_for_workflow",
                    "user_input_requests",
                    &e.to_string(),
                );
                format!("Deserialization error: {}", e)
            })?;

            if request.workflow_execution_id == workflow_id
                && matches!(request.status, InputRequestStatus::Pending)
            {
                requests.push(request);
            }
        }

        log_db_operation_success("get_pending_inputs_for_workflow", "user_input_requests", 0);
        Ok(requests)
    }

    pub async fn get_all_pending_inputs(&self) -> Result<Vec<UserInputRequest>, String> {
        log_db_operation_start("get_all_pending_inputs", "user_input_requests");

        let rows = sqlx::query("SELECT data FROM user_input_requests")
            .fetch_all(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error(
                    "get_all_pending_inputs",
                    "user_input_requests",
                    &e.to_string(),
                );
                format!("Database error: {}", e)
            })?;

        let mut requests = Vec::new();
        for row in rows {
            let json_data: String = row.get("data");
            log_deserialization("UserInputRequest", json_data.len());

            let request: UserInputRequest = serde_json::from_str(&json_data).map_err(|e| {
                log_db_operation_error(
                    "get_all_pending_inputs",
                    "user_input_requests",
                    &e.to_string(),
                );
                format!("Deserialization error: {}", e)
            })?;

            if matches!(request.status, InputRequestStatus::Pending) {
                requests.push(request);
            }
        }

        log_db_operation_success("get_all_pending_inputs", "user_input_requests", 0);
        Ok(requests)
    }

    pub async fn fulfill_input_request(&self, id: &str, value: String) -> Result<(), String> {
        log_db_operation_start("fulfill_input_request", "user_input_requests");

        let mut request = self
            .get_input_request(id)
            .await?
            .ok_or_else(|| "Input request not found".to_string())?;

        request.status = InputRequestStatus::Fulfilled;
        request.fulfilled_at = Some(Utc::now());
        request.fulfilled_value = Some(value.clone());

        request
            .validate()
            .map_err(|e| format!("Validation error: {}", e))?;

        self.save_input_request(&request).await?;

        log_db_operation_success("fulfill_input_request", "user_input_requests", 0);
        Ok(())
    }

    pub async fn delete_input_request(&self, id: &str) -> Result<(), String> {
        log_db_operation_start("delete_input_request", "user_input_requests");

        sqlx::query("DELETE FROM user_input_requests WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(|e| {
                log_db_operation_error(
                    "delete_input_request",
                    "user_input_requests",
                    &e.to_string(),
                );
                format!("Database error: {}", e)
            })?;

        log_db_operation_success("delete_input_request", "user_input_requests", 0);
        Ok(())
    }
}
