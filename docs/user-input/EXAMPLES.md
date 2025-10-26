# Example Workflows - User Input

## Overview

This document describes the example workflows included with the SEE project that demonstrate user input functionality.

**Location:** `engine/examples/`

## Basic Examples

### simple.json

A basic workflow demonstrating sequential CLI command execution.

**Structure:**
- Single task with `cli_command` function
- No dependencies or nesting

**Use Case:**
Demonstrate basic workflow execution with CLI commands.

**Running:**
```bash
cargo run -p s_e_e_cli -- --file engine/examples/simple.json
```

### parallel.json

Demonstrates parallel task execution.

**Structure:**
- Multiple tasks that execute in parallel
- No input requirements

**Use Case:**
Test parallel task execution capabilities.

**Running:**
```bash
cargo run -p s_e_e_cli -- --file engine/examples/parallel.json
```

### nested.json

Demonstrates nested task dependencies.

**Structure:**
- Sequential tasks with dependencies
- Tasks waiting on other tasks

**Use Case:**
Test task dependency handling.

**Running:**
```bash
cargo run -p s_e_e_cli -- --file engine/examples/nested.json
```

## User Input Examples

### user_input_simple.json

A simple workflow with a single user input request.

**Workflow ID:** `simple-input`

**Description:**
Demonstrates basic user input collection with a greeting, input request, and thank you message.

**Structure:**
```
greeting (CLI) 
  └─> get-name (User Input)
        └─> thank-you (CLI)
```

**Tasks:**
1. **greeting** - Displays a greeting message
   - Function: `cli_command`
   - Output: "Hello! What's your name?"

2. **get-name** - Requests user name input
   - Function: `user_input`
   - Prompt: "Please enter your name:"
   - Type: `string`
   - Required: `true`

3. **thank-you** - Displays thank you message
   - Function: `cli_command`
   - Output: "Thank you for your input!"

**Expected Behavior:**
1. Workflow starts
2. "Hello! What's your name?" is displayed
3. Workflow pauses, waiting for input
4. User enters name (e.g., "John Doe")
5. "Thank you for your input!" is displayed
6. Workflow completes

**Running:**
```bash
# CLI
cargo run -p s_e_e_cli -- --file engine/examples/user_input_simple.json

# GUI
# Workflow can be loaded and executed through the GUI
```

**Example Session:**
```bash
$ cargo run -p s_e_e_cli -- --file engine/examples/user_input_simple.json

Starting workflow: Simple User Input Workflow
[Task 1/3] Display Greeting: Complete
[Task 2/3] Get User Name: Waiting for input...

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Task: get-name
Status: Waiting for Input

Prompt: Please enter your name:
Type: string
Required: true
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Enter input:
> John Doe

Input received: John Doe
[Task 2/3] Get User Name: Complete
[Task 3/3] Thank You: Complete

Workflow completed successfully!
```

### user_input_parallel.json

Demonstrates parallel user input requests.

**Workflow ID:** `parallel-input`

**Description:**
Shows how multiple input tasks can request input simultaneously and independently.

**Structure:**
```
start (CLI)
  ├─> input-a (User Input)
  └─> input-b (User Input)
```

**Tasks:**
1. **start** - Initial message
   - Function: `cli_command`
   - Output: "Starting parallel input tasks"

2. **input-a** - First parallel input
   - Function: `user_input`
   - Prompt: "Enter value A:"
   - Type: `string`
   - Required: `true`

3. **input-b** - Second parallel input
   - Function: `user_input`
   - Prompt: "Enter value B:"
   - Type: `string`
   - Required: `true`

**Expected Behavior:**
1. "Starting parallel input tasks" is displayed
2. Both `input-a` and `input-b` pause, waiting for input
3. User can provide inputs in any order
4. Each input is processed independently
5. Workflow completes when both inputs are provided

**Running:**
```bash
cargo run -p s_e_e_cli -- --file engine/examples/user_input_parallel.json
```

**Example Session:**
```bash
$ cargo run -p s_e_e_cli -- --file engine/examples/user_input_parallel.json

Starting workflow: Parallel User Input Workflow
[Task 1/3] Start: Complete
[Task 2/3] Input A: Waiting for input...
[Task 3/3] Input B: Waiting for input...

Pending inputs:
  1. Task "Input A" (required)
  2. Task "Input B" (required)

Enter input for "Input A":
> Apple

Input received for Input A
Enter input for "Input B":
> Banana

Input received for Input B
[Task 2/3] Input A: Complete
[Task 3/3] Input B: Complete

Workflow completed successfully!
```

### user_input_nested.json

Demonstrates sequential nested user input requests.

**Workflow ID:** `nested-input`

**Description:**
Shows how inputs can be requested sequentially in a nested workflow structure.

**Structure:**
```
step1 (CLI)
  └─> step2-input (User Input)
        └─> step3 (CLI)
```

**Tasks:**
1. **step1** - Initial step
   - Function: `cli_command`
   - Output: "Step 1 complete"

2. **step2-input** - First input
   - Function: `user_input`
   - Prompt: "Enter value for step 2:"
   - Type: `string`
   - Required: `true`

3. **step3** - Final step
   - Function: `cli_command`
   - Output: "Step 3 complete"

**Expected Behavior:**
1. "Step 1 complete" is displayed
2. Workflow pauses for step 2 input
3. User enters input (e.g., "intermediate")
4. "Step 3 complete" is displayed
5. Workflow completes

**Running:**
```bash
cargo run -p s_e_e_cli -- --file engine/examples/user_input_nested.json
```

**Example Session:**
```bash
$ cargo run -p s_e_e_cli -- --file engine/examples/user_input_nested.json

Starting workflow: Nested User Input Workflow
[Task 1/3] Step 1: Complete
[Task 2/3] Step 2 Input: Waiting for input...

Enter value for step 2:
> intermediate

Input received for Step 2 Input
[Task 2/3] Step 2 Input: Complete
[Task 3/3] Step 3: Complete

Workflow completed successfully!
```

## Creating Your Own User Input Workflows

### Basic Structure

A user input task uses the `user_input` function type:

```json
{
  "id": "task-id",
  "name": "Task Name",
  "function": {
    "user_input": {
      "prompt": "Your prompt message",
      "input_type": "string",
      "required": true,
      "default": null
    }
  },
  "next_tasks": []
}
```

### Input Configuration

#### prompt
- **Type:** `string`
- **Required:** Yes
- **Description:** Message displayed to the user when input is required
- **Example:** `"Please enter your name:"`

#### input_type
- **Type:** `string`
- **Required:** Yes
- **Description:** Type of input expected
- **Values:** `"string"`, `"number"`, `"boolean"`

#### required
- **Type:** `boolean`
- **Required:** Yes
- **Description:** Whether input is mandatory
- **Example:** `true`

#### default
- **Type:** `Value` (number, string, boolean, or null)
- **Required:** No
- **Description:** Default value if input is not provided (requires `required: false`)
- **Example:** `"default_value"` or `null`

### Input Types

#### String
```json
{
  "function": {
    "user_input": {
      "prompt": "Enter your name:",
      "input_type": "string",
      "required": true,
      "default": null
    }
  }
}
```

**Accepted:** Any text input
**Examples:** "John", "Hello World", "123abc"

#### Number
```json
{
  "function": {
    "user_input": {
      "prompt": "Enter a number:",
      "input_type": "number",
      "required": true,
      "default": null
    }
  }
}
```

**Accepted:** Integer or floating point numbers
**Examples:** "123", "3.14", "-42"

#### Boolean
```json
{
  "function": {
    "user_input": {
      "prompt": "Continue? (yes/no):",
      "input_type": "boolean",
      "required": true,
      "default": null
    }
  }
}
```

**Accepted:** "true", "false", "1", "0", "yes", "no" (case-insensitive)
**Examples:** "true", "no", "1", "Yes"

### Example: Custom Workflow

**File:** `my_input_workflow.json`

```json
{
  "id": "custom-input",
  "name": "Custom Input Workflow",
  "tasks": [
    {
      "id": "introduction",
      "name": "Introduction",
      "function": {
        "cli_command": {
          "command": "echo",
          "args": ["Welcome to my workflow"]
        }
      },
      "next_tasks": [
        {
          "id": "get-email",
          "name": "Get Email",
          "function": {
            "user_input": {
              "prompt": "Enter your email address:",
              "input_type": "string",
              "required": true,
              "default": null
            }
          },
          "next_tasks": [
            {
              "id": "get-age",
              "name": "Get Age",
              "function": {
                "user_input": {
                  "prompt": "Enter your age:",
                  "input_type": "number",
                  "required": true,
                  "default": null
                }
              },
              "next_tasks": [
                {
                  "id": "confirm",
                  "name": "Confirm",
                  "function": {
                    "user_input": {
                      "prompt": "Proceed? (yes/no):",
                      "input_type": "boolean",
                      "required": true,
                      "default": null
                    }
                  },
                  "next_tasks": [
                    {
                      "id": "finish",
                      "name": "Finish",
                      "function": {
                        "cli_command": {
                          "command": "echo",
                          "args": ["Workflow complete!"]
                        }
                      },
                      "next_tasks": []
                    }
                  ]
                }
              ]
            }
          ]
        }
      ]
    }
  ]
}
```

## Testing Workflows

### Using the Test Engine

The engine includes a test harness for testing workflows:

```bash
cargo run -p engine -- test engine/examples/user_input_simple.json
```

### Manual Testing

1. Save workflow to a `.json` file
2. Run through CLI:
   ```bash
   cargo run -p s_e_e_cli -- --file my_workflow.json
   ```
3. Provide inputs as prompted
4. Verify workflow completes successfully

### Integration Testing

Example workflows are used in integration tests. See:
- `core/tests/integration_user_input_tests.rs`

## Troubleshooting

### Input Not Being Accepted

**Problem:** Input validation fails

**Solution:**
- Check `input_type` matches the value you're entering
- For `number` type, ensure input is numeric
- For `boolean` type, use "true", "false", "yes", "no", "1", or "0"

### Workflow Not Pausing

**Problem:** Workflow completes without requesting input

**Solution:**
- Verify `user_input` function type is used correctly
- Check task ID references in `next_tasks`
- Ensure task is not being skipped due to dependencies

### Multiple Inputs Not Working

**Problem:** Only first input is requested

**Solution:**
- Use `next_tasks` array for multiple parallel inputs
- Each input task must be a separate node in the workflow
- Ensure proper `next_tasks` structure

## See Also

- [API Documentation](./API_DOCUMENTATION.md)
- [Implementation Steps](./IMPLEMENTATION_STEPS.md)
- [Testing Strategy](./TESTING_STRATEGY.md)

