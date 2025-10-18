# see
Speculative Execution Engine

A lightweight workflow execution engine built on [dataflow-rs](https://crates.io/crates/dataflow-rs) that processes JSON-based workflows with support for CLI commands and context passing between tasks.

## Features

- **Multi-step workflow execution** - All tasks execute in sequence with full context passing
- **CLI command execution** - Execute shell commands as workflow tasks
- **Audit trail** - Full tracking of task execution with timestamps and changes
- **Context management** - Task outputs are automatically stored and available for inspection
- **JSON-based workflows** - Define workflows using dataflow-rs's JSON format

## Installation

### Prerequisites

- Rust 1.70 or later
- Cargo

### Build

```bash
cargo build --release
```

## Usage

### Running a Workflow

```bash
cargo run -- workflow.json
```

Or if you've built the binary:

```bash
./target/release/simple_workflow_app workflow.json
```

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

## Testing

Run all tests:

```bash
cargo test
```

Run only CLI integration tests:

```bash
cargo test --test integration_test
```

Run only GUI integration tests:

```bash
cargo test --test gui_integration_test
```

Run tests with output:

```bash
cargo test -- --nocapture
```

### Test Types

- **CLI Integration Tests** (`tests/integration_test.rs`) - Test the CLI binary by spawning subprocesses
- **GUI Integration Tests** (`tests/gui_integration_test.rs`) - Test all Dioxus UI components with VirtualDom rendering
  - **WorkflowInfoCard** - Tests success/failure states, data rendering, structure
  - **ErrorsPanel** - Tests error display, empty state, styling
  - **ContextPanel** - Tests collapsed/expanded states, JSON rendering, copy functionality
  - **OutputLogsPanel** - Tests log display, collapsed/expanded states, empty handling
  - **Toast** - Tests message display, empty state, positioning
  - **Sidebar** - Tests all states (idle/running), theme toggle, file input, structure
- **JSON Parser Tests** (`tests/json_parser_test.rs`) - Test JSON parsing utilities

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
- **Custom Function Handler**: `CliCommandHandler` implements the `AsyncFunctionHandler` trait to execute CLI commands
- **Message & Context**: Each workflow execution maintains a message with context that stores task outputs
- **Audit Trail**: Every task execution is tracked with status, timestamp, and changes

## JSON Parser

The project includes a utility module (`json_parser`) for extracting JSON payloads from CLI command outputs, supporting:
- Pure JSON parsing
- JSON extraction from text with surrounding content
- Multiple JSON objects extraction
- Nested structures and escaped characters

## License

See LICENSE file for details.
