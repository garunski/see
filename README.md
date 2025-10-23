# see
Speculative Execution Engine

A lightweight workflow execution engine built on [dataflow-rs](https://crates.io/crates/dataflow-rs) that processes JSON-based workflows with support for CLI commands and context passing between tasks.

## Features

- **Multi-step workflow execution** - All tasks execute in sequence with full context passing
- **CLI command execution** - Execute shell commands as workflow tasks
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
cargo run -p cli -- --file workflow.json
```

Or build and run:

```bash
cargo build -p cli --release
./target/release/cli --file workflow.json
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
dx serve --package gui

# Build for production
dx build --package gui

# Run without hot-reloading
dx run --package gui
```

Or using cargo directly:

```bash
# Build visualizer first
cd gui/react-flow-app && npm install && npm run build && cd ../..

# Run GUI
cargo run -p gui
```

**Features:**
- Create and edit workflows
- Visual workflow diagram with interactive nodes
- Drag-and-drop node positioning
- Zoom and pan controls
- Execution history viewer

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

## Testing

Tests were removed as part of the repo repair and will be re-authored later.

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
