# Workflow Engine

A high-performance, fully auditable workflow engine with recursive `next_tasks` dependency system, comprehensive logging, and parallel execution capabilities.

## 🚀 Features

- **Recursive `next_tasks`**: Tasks can have nested `next_tasks` arrays with unlimited depth
- **Parallel Execution**: Multiple tasks in `next_tasks` execute simultaneously for optimal performance
- **Sequential Dependencies**: Tasks execute in proper dependency order
- **Comprehensive Logging**: Complete audit trail with structured logging at debug, trace, and info levels
- **Task Handlers**: Extensible system for different task types (CLI, Cursor Agent, Custom)
- **Error Handling**: Robust error handling with detailed context and recovery
- **Testing**: Comprehensive test suite with unit and integration tests

## 🏗️ Architecture Overview

The engine is built with a clean, modular architecture:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   JSON Parser   │───▶│ Dependency Graph│───▶│ Execution Engine│
│                 │    │                 │    │                 │
│ • Recursive     │    │ • Topo Sort     │    │ • Parallel Exec │
│ • Validation    │    │ • Ready Tasks   │    │ • Task Handlers │
│ • Error Context │    │ • Dependencies  │    │ • Audit Trail   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Task Types    │    │   Task Handlers │    │   Logging       │
│                 │    │                 │    │                 │
│ • CLI Command   │    │ • CLI Handler   │    │ • Structured    │
│ • Cursor Agent  │    │ • Agent Handler │    │ • Trace/Debug   │
│ • Custom        │    │ • Custom Handler│    │ • Performance   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 📁 File Structure

```
engine/
├── README.md                    # This comprehensive documentation
├── Cargo.toml                   # Package configuration and dependencies
├── src/
│   ├── lib.rs                   # Main library exports and public API
│   ├── types.rs                 # Core data structures (EngineTask, EngineWorkflow, etc.)
│   ├── parser.rs                # JSON parsing with recursive next_tasks support
│   ├── graph.rs                 # Dependency graph and execution ordering
│   ├── handlers/                # Task handlers directory
│   │   ├── mod.rs               # Handler registry and trait definitions
│   │   ├── cli_command.rs       # CLI command execution handler
│   │   ├── cursor_agent.rs      # Cursor agent execution handler
│   │   └── custom.rs            # Custom function execution handler
│   ├── engine.rs                # Main execution engine with parallel/sequential logic
│   ├── errors.rs                # Error types and handling
│   ├── bin/
│   │   └── test_engine.rs       # CLI tool for testing workflows
│   └── tests/                   # Comprehensive test suite
│       ├── mod.rs               # Test module declarations
│       ├── parser_tests.rs      # Parser unit tests
│       ├── graph_tests.rs       # Dependency graph tests
│       ├── handler_tests.rs     # Task handler tests
│       ├── engine_tests.rs      # Execution engine tests
│       └── integration.rs       # End-to-end integration tests
└── examples/                    # Example workflow definitions
    ├── simple.json              # Simple sequential workflow
    ├── parallel.json            # Parallel execution workflow
    └── nested.json              # Nested dependencies workflow
```

## 🛠️ Usage

### Basic Usage

```rust
use engine::*;

// Parse and execute a workflow from JSON
let json = r#"
{
  "id": "my_workflow",
  "name": "My Workflow",
  "tasks": [
    {
      "id": "task1",
      "name": "Task 1",
      "function": {
        "name": "cli_command",
        "input": {
          "command": "echo",
          "args": ["Hello World"]
        }
      },
      "next_tasks": [
        {
          "id": "task2",
          "name": "Task 2",
          "function": {
            "name": "cli_command",
            "input": {
              "command": "echo",
              "args": ["Task 2 executing"]
            }
          }
        }
      ]
    }
  ]
}
"#;

let result = execute_workflow_from_json(json).await?;
println!("Workflow completed: {}", result.success);
```

### CLI Testing Tool

```bash
# Run a workflow from file
cargo run --bin test_engine examples/simple.json

# Enable debug logging
RUST_LOG=debug cargo run --bin test_engine examples/parallel.json

# Enable trace logging for maximum detail
RUST_LOG=trace cargo run --bin test_engine examples/nested.json
```

## 📋 Task Types

### CLI Command Tasks
Execute shell commands with arguments:

```json
{
  "id": "cli_task",
  "name": "CLI Task",
  "function": {
    "name": "cli_command",
    "input": {
      "command": "echo",
      "args": ["Hello", "World"]
    }
  }
}
```

### Cursor Agent Tasks
Execute AI-powered tasks with prompts:

```json
{
  "id": "agent_task",
  "name": "Agent Task",
  "function": {
    "name": "cursor_agent",
    "input": {
      "prompt": "Generate a summary of the project",
      "config": {
        "temperature": 0.7,
        "max_tokens": 1000
      }
    }
  }
}
```

### Custom Tasks
Execute custom functions with arbitrary input:

```json
{
  "id": "custom_task",
  "name": "Custom Task",
  "function": {
    "name": "my_custom_function",
    "input": {
      "param1": "value1",
      "param2": 42
    }
  }
}
```

## 🔄 Dependency System

### Recursive `next_tasks`
Tasks can have nested `next_tasks` arrays with unlimited depth:

```json
{
  "id": "root",
  "name": "Root Task",
  "function": { "name": "cli_command", "input": { "command": "echo", "args": ["Root"] } },
  "next_tasks": [
    {
      "id": "level1a",
      "name": "Level 1A",
      "function": { "name": "cli_command", "input": { "command": "echo", "args": ["Level 1A"] } },
      "next_tasks": [
        {
          "id": "level2a",
          "name": "Level 2A",
          "function": { "name": "cli_command", "input": { "command": "echo", "args": ["Level 2A"] } }
        }
      ]
    },
    {
      "id": "level1b",
      "name": "Level 1B",
      "function": { "name": "cli_command", "input": { "command": "echo", "args": ["Level 1B"] } }
    }
  ]
}
```

### Parallel Execution
Multiple tasks in `next_tasks` execute simultaneously:

```json
{
  "id": "root",
  "name": "Root Task",
  "function": { "name": "cli_command", "input": { "command": "echo", "args": ["Root"] } },
  "next_tasks": [
    { "id": "task_a", "name": "Task A", "function": { "name": "cli_command", "input": { "command": "echo", "args": ["A"] } } },
    { "id": "task_b", "name": "Task B", "function": { "name": "cli_command", "input": { "command": "echo", "args": ["B"] } } },
    { "id": "task_c", "name": "Task C", "function": { "name": "cli_command", "input": { "command": "echo", "args": ["C"] } } }
  ]
}
```

## 🔍 Logging and Observability

The engine provides comprehensive logging at multiple levels:

### Log Levels
- **`ERROR`**: Critical errors and failures
- **`WARN`**: Warnings and non-critical issues
- **`INFO`**: Important workflow and task events
- **`DEBUG`**: Detailed execution flow and state changes
- **`TRACE`**: Fine-grained parameter inspection and algorithm steps

### Structured Logging
All logs include structured fields for filtering and analysis:

```rust
debug!(
    execution_id = %context.execution_id,
    task_id = %task.id,
    command = %command,
    args_count = args.len(),
    "Spawning command process"
);
```

### Audit Trail
Complete audit trail of workflow execution:

```
2025-10-25T12:23:39.797520+00:00: Completed task: Task 1 (Success)
2025-10-25T12:23:39.799198+00:00: Completed task: Task 2 (Success)
```

## 🧪 Testing

### Running Tests
```bash
# Run all tests
cargo test

# Run with logging
RUST_LOG=debug cargo test

# Run specific test module
cargo test parser_tests
cargo test graph_tests
cargo test handler_tests
cargo test engine_tests
cargo test integration
```

### Test Coverage
- **Parser Tests**: JSON parsing, validation, error handling
- **Graph Tests**: Dependency resolution, cycle detection, topological sorting
- **Handler Tests**: Task execution, error handling, result processing
- **Engine Tests**: Workflow execution, parallel processing, state management
- **Integration Tests**: End-to-end workflow execution with real examples

## 🚀 Performance

### Parallel Execution
- Tasks with no dependencies execute in parallel
- Multiple `next_tasks` execute simultaneously
- Async/await for non-blocking execution
- Efficient dependency graph traversal

### Memory Management
- Minimal memory allocation during execution
- Efficient task state management
- Clean resource cleanup

### Scalability
- Handles workflows with hundreds of tasks
- Efficient dependency graph algorithms
- Optimized for both small and large workflows

## 🔧 Configuration

### Dependencies
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"
uuid = { version = "1.0", features = ["v4"] }
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1.0"
thiserror = "1.0"
chrono = { version = "0.4", features = ["serde"] }
```

### Logging Configuration
```rust
// Initialize tracing subscriber
tracing_subscriber::fmt::init();

// Set log level via environment variable
RUST_LOG=debug cargo run --bin test_engine examples/simple.json
```

## 🐛 Error Handling

### Error Types
- **`ParserError`**: JSON parsing and validation errors
- **`GraphError`**: Dependency graph construction and validation errors
- **`HandlerError`**: Task execution errors
- **`EngineError`**: Workflow execution errors

### Error Context
All errors include detailed context for debugging:

```rust
error!(
    task_id = %task_id,
    dependency_id = %dep_id,
    "Task depends on non-existent task"
);
```

## 🔮 Future Enhancements

- **Persistence Integration**: Connect with existing database persistence
- **UI Integration**: Connect with existing GUI components
- **Advanced Handlers**: More task handler types (HTTP, database, etc.)
- **Workflow Templates**: Pre-built workflow templates
- **Performance Metrics**: Detailed performance monitoring
- **Workflow Validation**: Advanced workflow validation rules
- **Hot Reloading**: Dynamic workflow updates without restart

## 📚 Examples

See the `examples/` directory for complete workflow examples:

- **`simple.json`**: Basic sequential workflow
- **`parallel.json`**: Parallel execution workflow
- **`nested.json`**: Complex nested dependencies

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## 📄 License

This project is licensed under the same terms as the main project.
