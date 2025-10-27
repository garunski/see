# System-Defined Workflows and Prompts

## Overview

System-defined workflows and prompts are pre-built templates that ship with the application. These templates provide starting points for common workflows and can be cloned by users to create editable versions.

## Features

- **Pre-installed Templates**: System templates are loaded from `/system` directory on application startup
- **Version Management**: Each template has a version field for automatic updates
- **Read-Only**: System templates cannot be edited directly by users
- **Cloning**: Users can clone system templates to create editable copies
- **Auto-Update**: Templates automatically update to the latest version on startup

## System Templates Location

```
/system
├── workflows/
│   ├── setup-project.json
│   ├── deploy-app.json
│   └── code-review.json
└── prompts/
    ├── code-review.json
    ├── bug-fix.json
    └── documentation.json
```

## Usage

### Viewing System Templates

System templates are loaded automatically when the application starts. They are stored in separate database tables (`system_workflows` and `system_prompts`) and are accessible through:

- **GUI**: System workflows appear in the workflows list (with distinct badges)
- **CLI**: Use `see_cli list-system-workflows` and `see_cli list-system-prompts`

### Cloning System Templates

Users can create editable copies of system templates by cloning them:

**CLI**:
```bash
# Clone a system workflow
see_cli clone-workflow --system-id system:setup-project --name "My Setup Project"

# Clone a system prompt
see_cli clone-prompt --system-id system:code-review --name "Custom Code Review"
```

**GUI**: Use the clone button next to each system template

## Creating Custom System Templates

To add custom system templates:

1. Create a JSON file in `/system/workflows/` or `/system/prompts/`
2. Follow the structure shown in existing templates
3. Include version information
4. Restart the application

Example workflow structure:
```json
{
  "id": "system:custom-workflow",
  "name": "Custom Workflow",
  "description": "A custom system workflow",
  "version": "1.0.0",
  "content": {
    "id": "custom",
    "name": "Custom Workflow",
    "tasks": [...]
  }
}
```

Example prompt structure:
```json
{
  "id": "system:custom-prompt",
  "name": "Custom Prompt",
  "description": "A custom system prompt",
  "version": "1.0.0",
  "content": "Template content with {{variables}}",
  "template": "Template content with {{variables}}",
  "variables": ["var1", "var2"],
  "tags": ["tag1"],
  "metadata": {}
}
```

## Database Schema

System templates are stored in two separate tables:

**system_workflows**:
- `id` (TEXT PRIMARY KEY)
- `data` (JSON) - Serialized `SystemWorkflow` struct

**system_prompts**:
- `id` (TEXT PRIMARY KEY)
- `data` (JSON) - Serialized `SystemPrompt` struct

## API

### Loading Templates

```rust
// Load all system templates
load_all_system_templates().await?;

// Load only workflows
load_system_workflows().await?;

// Load only prompts
load_system_prompts().await?;
```

### Cloning Templates

```rust
// Clone a system workflow
let user_workflow = clone_system_workflow(&system_id, Some("My Name")).await?;

// Clone a system prompt
let user_prompt = clone_system_prompt(&system_id, Some("My Name")).await?;
```

## Testing

Run system template tests:
```bash
cargo test --package persistence system_workflow
cargo test --package persistence system_prompt
```

## Technical Details

### Auto-Update Mechanism

On application startup, system templates are loaded from files and compared with existing database entries. If a template's version has changed, it is automatically replaced with the new version.

### Version Format

Version strings follow semantic versioning (e.g., "1.0.0"). The system will update templates when versions differ.

### Read-Only Protection

System templates cannot be directly edited through the API. Attempts to modify system templates result in errors. Users must clone templates to create editable versions.

## CLI Commands

```bash
# List all system workflows
see_cli list-system-workflows

# List all system prompts
see_cli list-system-prompts

# Clone a system workflow
see_cli clone-workflow --system-id <ID> [--name <NAME>]

# Clone a system prompt
see_cli clone-prompt --system-id <ID> [--name <NAME>]
```

## Related Documentation

- [Architecture](../task-ordering/ARCHITECTURE.md) - System architecture overview
- [User Input](../user-input/README.md) - User input workflows
- [Task Ordering](../task-ordering/README.md) - Workflow task ordering

