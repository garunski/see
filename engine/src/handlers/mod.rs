//! Task handlers for executing different types of tasks

use crate::errors::*;
use crate::types::*;
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::{debug, trace, warn};

/// Trait for task handlers
#[async_trait]
pub trait TaskHandler: Send + Sync {
    async fn execute(
        &self,
        context: &mut ExecutionContext,
        task: &EngineTask,
    ) -> Result<TaskResult, HandlerError>;
}

/// Registry for task handlers
pub struct HandlerRegistry {
    handlers: HashMap<String, Box<dyn TaskHandler>>,
}

impl HandlerRegistry {
    pub fn new() -> Self {
        debug!("Creating new handler registry");
        let mut handlers: HashMap<String, Box<dyn TaskHandler>> = HashMap::new();

        trace!("Registering CLI command handler");
        handlers.insert(
            "cli_command".to_string(),
            Box::new(cli_command::CliCommandHandler),
        );

        trace!("Registering Cursor agent handler");
        handlers.insert(
            "cursor_agent".to_string(),
            Box::new(cursor_agent::CursorAgentHandler),
        );

        trace!("Registering custom handler");
        handlers.insert("custom".to_string(), Box::new(custom::CustomHandler));

        debug!(
            registered_handlers = handlers.len(),
            handler_types = ?handlers.keys().collect::<Vec<_>>(),
            "Handler registry created successfully"
        );

        Self { handlers }
    }

    pub fn get_handler(&self, function_type: &str) -> Option<&dyn TaskHandler> {
        trace!(function_type = %function_type, "Looking up handler");
        let result = self.handlers.get(function_type).map(|h| h.as_ref());

        match result {
            Some(_) => {
                debug!(function_type = %function_type, "Handler found");
            }
            None => {
                warn!(function_type = %function_type, "Handler not found");
            }
        }

        result
    }

    pub fn register_handler(&mut self, name: String, handler: Box<dyn TaskHandler>) {
        debug!(handler_name = %name, "Registering new handler");
        self.handlers.insert(name, handler);
        trace!(
            total_handlers = self.handlers.len(),
            "Handler registered successfully"
        );
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the function type from a task
pub fn get_function_type(task: &EngineTask) -> &'static str {
    let function_type = match &task.function {
        TaskFunction::CliCommand { .. } => "cli_command",
        TaskFunction::CursorAgent { .. } => "cursor_agent",
        TaskFunction::Custom { .. } => "custom",
    };

    trace!(
        task_id = %task.id,
        function_type = %function_type,
        "Determined function type for task"
    );

    function_type
}

// Export individual handlers
pub mod cli_command;
pub mod cursor_agent;
pub mod custom;
