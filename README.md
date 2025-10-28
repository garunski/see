# see
Speculative Execution Engine

A lightweight workflow execution engine built on [dataflow-rs](https://crates.io/crates/dataflow-rs) that processes JSON-based workflows with support for CLI commands and context passing between tasks.

## Features

- **Multi-step workflow execution** - All tasks execute in sequence with full context passing
- **CLI command execution** - Execute shell commands as workflow tasks
- **User input support** - Interactively request and process user input during workflow execution
- **System templates** - Pre-built workflow and prompt templates that can be cloned
- **Task ordering** - Preserves exact workflow structure for correct task display order
- **Audit trail** - Full tracking of task execution with timestamps and changes
- **Context management** - Task outputs are automatically stored and available for inspection
- **JSON-based workflows** - Define workflows using dataflow-rs's JSON format
- **Visual workflow editor** - Interactive React Flow-based diagram editor with drag-and-drop nodes

## Installation

### Prerequisites

- Rust 1.70 or later
- Cargo
- Dioxus CLI (`dx`) - Install with: `cargo binstall dioxus-cli`
- Node.js 18+ and npm (for building the workflow visualizer)

### Build

```bash
cargo build --release
```

## Usage

### CLI: Run a workflow

From the repo root:

```bash
cargo run -p s_e_e_cli -- --file workflow.json
```

Or build and run:

```bash
cargo build -p s_e_e_cli --release
./target/release/cli --file workflow.json
```

**Interactive Input Support:**
- Workflows with user input tasks will pause execution
- Prompts are displayed in the terminal
- Input validation ensures correct data types
- Workflow automatically resumes after input is provided

**Example Session:**
```bash
$ cargo run -p s_e_e_cli -- --file user_input_simple.json

Starting workflow: Simple User Input Workflow
[Task 1/3] Display Greeting: Complete
[Task 2/3] Get User Name: Waiting for input...

Please enter your name:
> John Doe

Input received: John Doe
[Task 2/3] Get User Name: Complete
[Task 3/3] Thank You: Complete

Workflow completed successfully!
```

### GUI: Desktop app

The GUI includes a visual workflow editor powered by React Flow.

Using Task (recommended):

```bash
# Development with hot-reloading (builds React Flow app automatically)
task serve-gui

# Run GUI
task run-gui

# Build for production
task build-release
```

Using dx CLI directly:

```bash
# Build React Flow visualizer first
task build-visualizer

# Development with hot-reloading
dx serve --package s_e_e_gui

# Build for production
dx build --package s_e_e_gui

# Run without hot-reloading
dx run --package s_e_e_gui
```

Or using cargo directly:

```bash
# Build visualizer first
cd gui/react-flow-app && npm install && npm run build && cd ../..

# Run GUI
cargo run -p s_e_e_gui
```

**Features:**
- Create and edit workflows
- Visual workflow diagram with interactive nodes
- Drag-and-drop node positioning
- Zoom and pan controls
- Execution list viewer
- Interactive user input forms for workflows requiring input
- Visual indicators for tasks waiting for input (amber/yellow highlighting)
- Pending input count display and filtering

### Workflow Format

Workflows are defined in JSON using the dataflow-rs format:

```json
{
  "id": "my_workflow",
  "name": "My Workflow",
  "tasks": [
    {
      "id": "task_1",
      "name": "First Task",
      "function": {
        "name": "cli_command",
        "input": {
          "command": "echo",
          "args": ["Hello World"]
        }
      }
    },
    {
      "id": "task_2",
      "name": "Second Task",
      "function": {
        "name": "cli_command",
        "input": {
          "command": "date",
          "args": []
        }
      }
    }
  ]
}
```

### User Input Workflows

Workflows can request user input during execution. When a task requires input, the workflow pauses until the input is provided.

**Example: Simple Input Workflow**

```json
{
  "id": "simple-input",
  "name": "Simple User Input Workflow",
  "tasks": [
    {
      "id": "greeting",
      "name": "Display Greeting",
      "function": {
        "cli_command": {
          "command": "echo",
          "args": ["Hello! What's your name?"]
        }
      },
      "next_tasks": [
        {
          "id": "get-name",
          "name": "Get User Name",
          "function": {
            "user_input": {
              "prompt": "Please enter your name:",
              "input_type": "string",
              "required": true,
              "default": null
            }
          },
          "next_tasks": [
            {
              "id": "thank-you",
              "name": "Thank You",
              "function": {
                "cli_command": {
                  "command": "echo",
                  "args": ["Thank you for your input!"]
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
```

**Input Types:**
- `string` - Any text input
- `number` - Numeric input (integer or float)
- `boolean` - Boolean input (true/false, yes/no, 1/0)

**Input Properties:**
- `prompt` - The message shown to the user
- `input_type` - Type of input expected (string, number, boolean)
- `required` - Whether input is required (true/false)
- `default` - Optional default value

**Workflow Behavior:**
- Tasks with `user_input` pause workflow execution
- Input can be provided via CLI prompts or GUI forms
- After input is provided, the workflow automatically resumes
- Multiple parallel input tasks can request input simultaneously

## Workflow Visualizer

The GUI includes an interactive workflow visualizer built with React Flow. See [WORKFLOW_VISUALIZER.md](./WORKFLOW_VISUALIZER.md) for detailed documentation.

**Quick Start:**
1. Open the GUI application
2. Navigate to "Workflows" page
3. Click "Visualize" next to any workflow
4. View and interact with the workflow diagram

**Features:**
- Sequential task visualization with auto-generated edges
- Drag nodes to reposition them
- Zoom and pan controls
- Node positions persist in workflow metadata
- Fully responsive and interactive

## Example Workflows

The project includes example workflows demonstrating different features:

**Location:** `engine/examples/`

### Basic Examples
- `simple.json` - Basic workflow with CLI commands
- `parallel.json` - Parallel task execution
- `nested.json` - Nested task dependencies

### User Input Examples
- `user_input_simple.json` - Single input request
- `user_input_parallel.json` - Multiple parallel input requests
- `user_input_nested.json` - Sequential nested input requests

### Running Examples

**Simple CLI workflow:**
```bash
cargo run -p s_e_e_cli -- --file engine/examples/simple.json
```

**User input workflow:**
```bash
cargo run -p s_e_e_cli -- --file engine/examples/user_input_simple.json
```

**Parallel input workflow:**
```bash
cargo run -p s_e_e_cli -- --file engine/examples/user_input_parallel.json
```

**Nested input workflow:**
```bash
cargo run -p s_e_e_cli -- --file engine/examples/user_input_nested.json
```

## System Templates

System templates are pre-built workflows and prompts that ship with the application. Users can clone these templates to create editable copies.

### Using System Templates

**Viewing Templates:**
```bash
# List system workflows
cargo run -p s_e_e_cli -- list-system-workflows

# List system prompts
cargo run -p s_e_e_cli -- list-system-prompts
```

**Cloning Templates:**
```bash
# Clone a system workflow
cargo run -p s_e_e_cli -- clone-workflow --system-id system:setup-project --name "My Setup"

# Clone a system prompt
cargo run -p s_e_e_cli -- clone-prompt --system-id system:code-review --name "Custom Review"
```

**GUI**: System templates appear in the GUI with distinct badges and can be cloned using the clone button.

See [System Templates Documentation](./docs/system-templates/README.md) for complete details.

## Task Ordering

Workflow executions now preserve the exact workflow structure for correct task ordering in the GUI.

### Features

- Complete workflow snapshot stored in each execution record
- Tasks display in correct execution order
- Self-contained execution records with full context
- Historical audit trail preserved

### How It Works

Each workflow execution automatically includes a `workflow_snapshot` that stores the complete workflow JSON structure at the time of execution. This allows the GUI to display tasks in the correct order based on the workflow's hierarchical structure, ensuring users see tasks in the exact sequence they executed.

The snapshot preserves:
- Complete task structure with nested `next_tasks`
- Original function definitions
- Task relationships and dependencies
- Execution order information

## Testing

## Development

The project includes a Taskfile for common development tasks:

```bash
# Install Task (if not already installed)
brew install go-task/tap/go-task

# Available tasks
task build           # Build the application
task run             # Run the application
task test            # Run all tests
task test-verbose    # Run tests with output
task test-integration # Run CLI integration tests only
task test-gui        # Run GUI integration tests only
task check           # Check code without building
task fmt             # Format code
task lint            # Run clippy linter
task clean           # Clean build artifacts
```

## Architecture

- **Engine**: Built on dataflow-rs, processes messages through workflows
- **Custom Function Handlers**: 
  - `CliCommandHandler` - Executes CLI commands
  - `UserInputHandler` - Handles user input requests and pauses/resumes workflow execution
- **Message & Context**: Each workflow execution maintains a message with context that stores task outputs
- **Audit Trail**: Every task execution is tracked with status, timestamp, and changes
- **Input Management**: User inputs are tracked with requests, validation, and fulfillment status

## JSON Parser

The project includes a utility module (`json_parser`) for extracting JSON payloads from CLI command outputs, supporting:
- Pure JSON parsing
- JSON extraction from text with surrounding content
- Multiple JSON objects extraction
- Nested structures and escaped characters

## Refactoring Lessons Learned

This project has undergone significant refactoring to improve code quality and maintainability. Key lessons learned:

### ✅ What Works Well

**Simple Data Access Hooks:**
- Functions that return `Memo<T>` are clean, reliable, and reactive
- Pattern: `use_workflows() -> Memo<Vec<WorkflowDefinition>>`
- Avoids complex state management while maintaining reactivity

**Incremental Migration:**
- Test after each small change with `cargo check --package s_e_e_gui`
- Run `task quality` frequently to catch issues early
- Migrate one page/component at a time

**Type Safety First:**
- Always verify correct types from state modules
- Use proper imports for complex types (`WorkflowExecutionSummary`, `WorkflowMetadata`)
- Both `lib.rs` and `main.rs` need module declarations

### ❌ What to Avoid

**Complex Closure Hooks:**
- Any hook returning closures in struct fields will fail with ownership issues
- `Box<dyn FnMut>` patterns cause move semantics problems in `rsx!` macros
- Keep confirmation dialogs and complex event handlers inline

**Over-Engineering:**
- Simple patterns are often better than "clever" abstractions
- Avoid hooks that try to manage too much state internally
- Don't extract patterns that work well inline

**Premature Abstraction:**
- Extract only when you have 3+ identical patterns
- Start with the simplest, most common patterns first
- Focus on reducing boilerplate, not creating "perfect" abstractions

### 🎯 Best Practices

1. **Keep it simple**: Extract only what's clearly repeated and works reliably
2. **Test incrementally**: Run quality checks after every change
3. **Document failures**: Record what doesn't work to avoid repeating mistakes
4. **Focus on data access**: Simple getter patterns are the safest to extract
5. **Maintain reactivity**: Always verify that extracted patterns maintain proper reactivity

### 📊 Refactoring Success Metrics

- ✅ All code compiles without errors
- ✅ All formatting and linting passes
- ✅ No behavioral changes to existing functionality
- ✅ Reduced boilerplate (3-4 lines → 1-2 lines)
- ✅ Consistent patterns across similar components


## License

See LICENSE file for details.
