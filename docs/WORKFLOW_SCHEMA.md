# Workflow JSON Schema Documentation

This document describes the JSON schema for workflow definitions in the See workflow system.

## Overview

A workflow is defined as a JSON object with the following structure:
- `id` (string, required): Unique identifier for the workflow
- `name` (string, required): Human-readable name for the workflow  
- `tasks` (array, required): Array of root-level tasks that define the workflow execution

## Task Structure

Each task is defined with the following fields:

- `id` (string, required): Unique identifier for the task within the workflow
- `name` (string, required): Human-readable name for the task
- `function` (object, required): Task function definition with:
  - `name` (string, required): Type of function (`cli_command`, `cursor_agent`, `user_input`, or `custom`)
  - `input` (object, required): Function-specific input data
- `next_tasks` (array, optional): Array of dependent tasks that execute after this task completes

### Task Functions

The `function` object has two required fields:
- `name` (string): Function type identifier
- `input` (object): Function-specific parameters

#### CLI Command

Execute a command-line command.

```json
{
  "name": "cli_command",
  "input": {
    "command": "echo",
    "args": ["Hello World"]
  }
}
```

**Parameters:**
- `command` (string, **required**): The command to execute
- `args` (array of strings, **optional**): Command arguments

**Example:**
```json
{
  "name": "cli_command",
  "input": {
    "command": "git",
    "args": ["status", "--short"]
  }
}
```

#### Cursor Agent

Execute a Cursor AI agent task.

```json
{
  "name": "cursor_agent",
  "input": {
    "prompt": "Fix the bug in src/main.rs",
    "config": {}
  }
}
```

**Parameters:**
- `prompt` (string, **required**): The prompt to send to the agent
- `config` (object, **optional**): Agent configuration (any JSON object)

**Example:**
```json
{
  "name": "cursor_agent",
  "input": {
    "prompt": "Add error handling to the API endpoint",
    "config": {
      "model": "gpt-4",
      "temperature": 0.7
    }
  }
}
```

#### User Input

Request input from the user during workflow execution.

```json
{
  "name": "user_input",
  "input": {
    "prompt": "Enter your name:",
    "input_type": "string",
    "required": true,
    "default": null
  }
}
```

**Parameters:**
- `prompt` (string, **required**): The prompt to display to the user
- `input_type` (string, **required**): Type of input expected (`string`, `number`, `boolean`, etc.)
- `required` (boolean, **optional**, default: `true`): Whether the input is required
- `default` (any, **optional**): Default value if user doesn't provide input

**Example:**
```json
{
  "name": "user_input",
  "input": {
    "prompt": "How many retries?",
    "input_type": "number",
    "required": false,
    "default": 3
  }
}
```

#### Custom Function

Execute a custom function handler.

```json
{
  "name": "custom",
  "input": {
    "endpoint": "/api/process",
    "data": { "key": "value" }
  }
}
```

**Parameters:**
- `input` (object, **required**): Custom function input data (any valid JSON)

**Note:** Custom functions allow flexible input structures. The input can be any valid JSON object.

**Example:**
```json
{
  "name": "custom",
  "input": {
    "handler": "processData",
    "params": {
      "timeout": 5000,
      "format": "json"
    }
  }
}
```

## Example Workflows

### Simple Sequential Workflow

```json
{
  "id": "simple",
  "name": "Simple Workflow",
  "tasks": [
    {
      "id": "task1",
      "name": "First Task",
      "function": {
        "name": "cli_command",
        "input": {
          "command": "echo",
          "args": ["Hello"]
        }
      },
      "next_tasks": [
        {
          "id": "task2",
          "name": "Second Task",
          "function": {
            "name": "cli_command",
            "input": {
              "command": "echo",
              "args": ["World"]
            }
          }
        }
      ]
    }
  ]
}
```

### Nested Dependencies

```json
{
  "id": "nested",
  "name": "Nested Workflow",
  "tasks": [
    {
      "id": "root",
      "name": "Root Task",
      "function": {
        "name": "cli_command",
        "input": {
          "command": "echo",
          "args": ["Starting"]
        }
      },
      "next_tasks": [
        {
          "id": "level1a",
          "name": "Level 1A",
          "function": {
            "name": "cli_command",
            "input": {
              "command": "echo",
              "args": ["Level 1A"]
            }
          },
          "next_tasks": [
            {
              "id": "level2",
              "name": "Level 2",
              "function": {
                "name": "cli_command",
                "input": {
                  "command": "echo",
                  "args": ["Level 2"]
                }
              }
            }
          ]
        }
      ]
    }
  ]
}
```

### Workflow with User Input

```json
{
  "id": "user-input",
  "name": "User Input Workflow",
  "tasks": [
    {
      "id": "get-name",
      "name": "Get User Name",
      "function": {
        "name": "user_input",
        "input": {
          "prompt": "Enter your name:",
          "input_type": "string",
          "required": true
        }
      },
      "next_tasks": [
        {
          "id": "greet",
          "name": "Greet User",
          "function": {
            "name": "cli_command",
            "input": {
              "command": "echo",
              "args": ["Hello!"]
            }
          }
        }
      ]
    }
  ]
}
```

## Validation Rules

### Required Fields

1. **Workflow level**: `id`, `name`, `tasks`
2. **Task level**: `id`, `name`, `function`
3. **Function level**: `name`, `input`

### Data Types

- `id` and `name`: Must be non-empty strings
- `tasks` and `next_tasks`: Must be arrays (can be empty)
- `function.name`: Must be one of the supported function types
- `function.input`: Must be an object (can be empty)

### Constraints

1. **Task ID Uniqueness**: All task IDs within a workflow must be unique, including nested tasks
2. **Recursive Validation**: All nested tasks must follow the same structure as root tasks
3. **Function Type Validation**: Function names must match valid types (`cli_command`, `cursor_agent`, `user_input`, `custom`)

## Common Validation Errors

### Missing Required Fields

**Error**: "Validation failed at /name"  
**Solution**: Add a `name` field to the workflow or task

**Error**: "Validation failed at /tasks"  
**Solution**: Add a `tasks` array to the workflow

### Duplicate Task IDs

**Error**: "Duplicate task ID: task1"  
**Solution**: Use unique IDs for each task in the workflow

### Invalid Function Structure

**Error**: "Validation failed at tasks[0].function"  
**Solution**: Ensure `function` has both `name` and `input` fields

**Error**: "Validation failed at tasks[0].function.input.command"  
**Solution**: CLI commands require `command` field in `input`

### Type Mismatches

**Error**: "Expected string, got null"  
**Solution**: Ensure all string fields have string values

## Schema File

The complete JSON Schema is available at `core/schema/workflow.schema.json`. This file can be used with:
- IDEs for autocomplete and validation
- External validation tools
- API documentation generators

## Generating the Schema

To regenerate the schema file:

```bash
cd core
cargo run --bin export_schema schema/workflow.schema.json
```

