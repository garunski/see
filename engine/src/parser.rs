use crate::errors::*;
use crate::types::*;
use serde_json::Value;
use std::collections::HashSet;
use tracing::{debug, error, instrument, trace, warn};

#[instrument]
pub fn parse_workflow(json: &str) -> Result<EngineWorkflow, ParserError> {
    debug!("Starting workflow JSON parsing");
    let workflow_json: Value = serde_json::from_str(json)?;
    trace!("JSON parsed successfully, delegating to value parser");
    parse_workflow_from_value(&workflow_json)
}

#[instrument(skip(workflow_json))]
pub fn parse_workflow_from_value(workflow_json: &Value) -> Result<EngineWorkflow, ParserError> {
    let id = workflow_json
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let name = workflow_json
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unnamed Workflow")
        .to_string();

    debug!(workflow_id = %id, workflow_name = %name, "Parsing workflow metadata");

    let tasks_array = workflow_json
        .get("tasks")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ParserError::MissingField("tasks".to_string()))?;

    debug!(
        workflow_id = %id,
        task_count = tasks_array.len(),
        "Found tasks array, starting recursive parsing"
    );

    let mut all_tasks = Vec::new();
    let mut task_ids = HashSet::new();

    for (i, task_json) in tasks_array.iter().enumerate() {
        trace!(
            workflow_id = %id,
            task_index = i,
            "Parsing root-level task"
        );
        parse_task_recursive(task_json, &mut all_tasks, None, &mut task_ids)?;
    }

    if all_tasks.len() != task_ids.len() {
        warn!(
            workflow_id = %id,
            all_tasks_count = all_tasks.len(),
            unique_ids_count = task_ids.len(),
            "Duplicate task IDs detected"
        );
        return Err(ParserError::InvalidTask(
            "Duplicate task IDs found".to_string(),
        ));
    }

    debug!(
        workflow_id = %id,
        total_tasks = all_tasks.len(),
        unique_task_ids = task_ids.len(),
        "Workflow parsing completed successfully"
    );

    Ok(EngineWorkflow {
        id,
        name,
        tasks: all_tasks,
    })
}

#[instrument(skip(all_tasks, task_ids), fields(parent_id = ?parent_id))]
fn parse_task_recursive(
    task_json: &Value,
    all_tasks: &mut Vec<EngineTask>,
    parent_id: Option<&str>,
    task_ids: &mut HashSet<String>,
) -> Result<EngineTask, ParserError> {
    let task_id = task_json
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ParserError::MissingField("id".to_string()))?
        .to_string();

    trace!(
        task_id = %task_id,
        parent_id = ?parent_id,
        "Parsing task"
    );

    if !task_ids.insert(task_id.clone()) {
        warn!(
            task_id = %task_id,
            "Duplicate task ID detected"
        );
        return Err(ParserError::InvalidTask(format!(
            "Duplicate task ID: {}",
            task_id
        )));
    }

    let task_name = task_json
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unnamed Task")
        .to_string();

    trace!(task_id = %task_id, "Parsing task function");
    let function = parse_task_function(task_json)?;

    let mut next_tasks = Vec::new();
    if let Some(next_tasks_array) = task_json.get("next_tasks").and_then(|v| v.as_array()) {
        trace!(
            task_id = %task_id,
            next_tasks_count = next_tasks_array.len(),
            "Found next_tasks, parsing recursively"
        );

        for (i, next_task_json) in next_tasks_array.iter().enumerate() {
            trace!(
                task_id = %task_id,
                next_task_index = i,
                "Parsing next task"
            );

            let next_task =
                parse_task_recursive(next_task_json, all_tasks, Some(&task_id), task_ids)?;
            next_tasks.push(next_task);
        }

        debug!(
            task_id = %task_id,
            next_tasks_count = next_tasks.len(),
            "Successfully parsed all next_tasks"
        );
    } else {
        trace!(task_id = %task_id, "No next_tasks found");
    }

    let task = EngineTask {
        id: task_id.clone(),
        name: task_name,
        function,
        next_tasks,
        status: TaskStatus::Pending,
        is_root: parent_id.is_none(),
    };

    trace!(
        task_id = %task_id,
        task_name = %task.name,
        next_tasks_count = task.next_tasks.len(),
        is_root = task.is_root,
        "Created task structure"
    );

    debug!(
        task_id = %task_id,
        task_name = %task.name,
        next_tasks_count = task.next_tasks.len(),
        "Task parsing completed"
    );

    all_tasks.push(task.clone());
    trace!(
        task_id = %task_id,
        total_tasks_so_far = all_tasks.len(),
        "Added task to all_tasks collection"
    );

    Ok(task)
}

#[instrument(skip(task_json), fields(task_id = ?task_json.get("id").and_then(|v| v.as_str())))]
fn parse_task_function(task_json: &Value) -> Result<TaskFunction, ParserError> {
    trace!("Starting task function parsing");

    let function = task_json.get("function").ok_or_else(|| {
        error!("Missing function field in task");
        ParserError::MissingField("function".to_string())
    })?;

    let function_type = function
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("custom");

    debug!(
        function_type = %function_type,
        "Parsing function of type"
    );

    match function_type {
        "cli_command" => {
            trace!("Parsing CLI command function");

            let input = function.get("input").ok_or_else(|| {
                error!(function_type = %function_type, "Missing input field for CLI command");
                ParserError::MissingField("function.input".to_string())
            })?;

            let command = input
                .get("command")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    error!(function_type = %function_type, "Missing command field");
                    ParserError::MissingField("function.input.command".to_string())
                })?
                .to_string();

            let args = input
                .get("args")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    let args: Vec<String> = arr
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(String::from)
                        .collect();
                    trace!(
                        command = %command,
                        args_count = args.len(),
                        args = ?args,
                        "Parsed CLI command arguments"
                    );
                    args
                })
                .unwrap_or_else(|| {
                    trace!(command = %command, "No arguments provided for CLI command");
                    Vec::new()
                });

            debug!(
                function_type = %function_type,
                command = %command,
                args_count = args.len(),
                "Successfully parsed CLI command function"
            );

            Ok(TaskFunction::CliCommand { command, args })
        }
        "cursor_agent" => {
            trace!("Parsing Cursor agent function");

            let input = function.get("input").ok_or_else(|| {
                error!(function_type = %function_type, "Missing input field for Cursor agent");
                ParserError::MissingField("function.input".to_string())
            })?;

            let prompt = input
                .get("prompt")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    error!(function_type = %function_type, "Missing prompt field");
                    ParserError::MissingField("function.input.prompt".to_string())
                })?
                .to_string();

            let config = input.clone();

            trace!(
                function_type = %function_type,
                prompt_length = prompt.len(),
                prompt_preview = %prompt.chars().take(100).collect::<String>(),
                config_keys = ?config.as_object().map(|o| o.keys().collect::<Vec<_>>()),
                "Parsed Cursor agent function"
            );

            debug!(
                function_type = %function_type,
                prompt_length = prompt.len(),
                "Successfully parsed Cursor agent function"
            );

            Ok(TaskFunction::CursorAgent { prompt, config })
        }
        "user_input" => {
            trace!("Parsing user input function");

            let input = function.get("input").ok_or_else(|| {
                error!(function_type = %function_type, "Missing input field for user input");
                ParserError::MissingField("function.input".to_string())
            })?;

            let prompt = input
                .get("prompt")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    error!(function_type = %function_type, "Missing prompt field");
                    ParserError::MissingField("function.input.prompt".to_string())
                })?
                .to_string();

            let input_type = input
                .get("input_type")
                .and_then(|v| v.as_str())
                .unwrap_or("string")
                .to_string();

            let required = input
                .get("required")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            let default = input.get("default").cloned();

            debug!(
                function_type = %function_type,
                prompt_length = prompt.len(),
                "Successfully parsed user input function"
            );

            Ok(TaskFunction::UserInput {
                prompt,
                input_type,
                required,
                default,
            })
        }
        _ => {
            trace!(
                function_type = %function_type,
                "Parsing custom function"
            );

            let name = function_type.to_string();
            let input = function.get("input").cloned().unwrap_or_else(|| {
                trace!(function_type = %function_type, "No input provided, using empty object");
                Value::Object(serde_json::Map::new())
            });

            trace!(
                function_type = %function_type,
                function_name = %name,
                input_type = ?input,
                "Parsed custom function"
            );

            debug!(
                function_type = %function_type,
                function_name = %name,
                "Successfully parsed custom function"
            );

            Ok(TaskFunction::Custom { name, input })
        }
    }
}
