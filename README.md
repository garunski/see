# `s_e_e` — Speculative Execution Engine

`s_e_e` is a flexible Rust workflow engine with CLI support for orchestrating multi-step processes. It supports sequential and parallel execution, dynamic user input, context passing between tasks, and full audit trails.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)  
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

---

## Table of Contents

- [Releases](#releases)  
- [Quick Start](#quick-start)  
- [Features](#features)  
- [Usage](#usage)  
- [Documentation](#documentation)  
- [Contributing](#contributing)  
- [License](#license)  
- [Code of Conduct](#code-of-conduct)  
- [Security](#security)  

---

## Releases

Prebuilt binaries are available for users who do not want to build `s_e_e` from source.

### Supported Platforms (Alpha)

- **macOS** — Apple Silicon  
- **Windows** — 64-bit  

> These releases are currently in alpha. Features are functional but may change, and users should expect occasional bugs.

### How to Use Prebuilt Releases

1. **Download the latest release** from the [Releases page](https://github.com/garunski/s_e_e/releases).  
2. **Extract the archive**:
   - macOS: `tar -xzf s_e_e-x.y.z-macos.tar.gz`  
   - Windows: Extract the `.zip` file via Explorer or PowerShell  
3. **Run the CLI directly** from the extracted folder:

```bash
# macOS example
./s_e_e_cli --file path/to/workflow.json

# Windows example
s_e_e_cli.exe --file path\to\workflow.json
```

No Rust toolchain or compilation is required to run these binaries.

---

## Quick Start

Clone and build the project (if you want the latest code or Linux support):

```bash
git clone https://github.com/garunski/s_e_e.git
cd s_e_e
cargo build --release
cargo run -p s_e_e_cli -- --file engine/examples/simple.json
```

This executes a JSON-defined workflow demonstrating task sequencing and context passing.

---

## Features

**Workflow Execution**

- Sequential and parallel execution of tasks  
- User input support during workflow execution  
- Context passing between tasks  
- Full audit trail of workflow execution

**CLI-First**

- Lightweight, terminal-based workflow execution  
- Interactive prompts for input-based workflows  
- Pre-defined workflow templates

**Extensible Core**

- Modular architecture: CLI, engine, and persistence layers  
- Custom task types can be added  
- Workflows defined in structured JSON format

**Additional Capabilities**

- GUI editor for workflow visualization and management (React Flow)  
- Task ordering preserves execution dependencies  
- Cloneable workflow templates for rapid prototyping

---

## Usage

### Running a Workflow (CLI)

```bash
cargo run -p s_e_e_cli -- --file workflow.json
```

Or run prebuilt binaries:

```bash
./s_e_e_cli --file workflow.json   # macOS
s_e_e_cli.exe --file workflow.json # Windows
```

### Example JSON Workflow

```json
{
  "id": "example_workflow",
  "tasks": [
    {
      "id": "task1",
      "function": {
        "name": "cli_command",
        "input": { "command": "echo", "args": ["Hello World"] }
      },
      "next_tasks": [
        {
          "id": "task2",
          "function": {
            "name": "cli_command",
            "input": { "command": "date" }
          }
        }
      ]
    }
  ]
}
```

Run it:

```bash
cargo run -p s_e_e_cli -- --file my_workflow.json
```

### User Input Example

```bash
cargo run -p s_e_e_cli -- --file engine/examples/user_input_simple.json
```

Workflows can pause to request user input:

```
Please enter your name:
> John Doe
Input received: John Doe
Workflow completed successfully!
```

See `engine/examples/` for more workflows: `simple.json`, `parallel.json`, `nested.json`, `user_input_simple.json`, etc.

---

## License

Apache License 2.0 — [LICENSE](LICENSE)

