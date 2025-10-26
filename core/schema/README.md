# Workflow Schema Maintenance

## Overview

This directory contains the JSON Schema for workflow definitions used by the See workflow system.

## Schema File

- **File:** `workflow.schema.json`
- **Version:** 1.0.0
- **Draft:** JSON Schema Draft 07
- **Format:** JSON

## When to Update Schema

Update the schema when:
- Adding new function types (e.g., new task function handlers)
- Changing required/optional fields for existing function types
- Modifying field types or constraints (e.g., string length requirements)
- Adding new validation rules
- Changing the workflow structure

## Update Process

### 1. Edit the Schema

Edit `workflow.schema.json` to reflect the changes.

**Common changes:**
- Adding a new function type: Add to `Function.oneOf` array
- Changing required fields: Update `required` array in function definition
- Changing field types: Update `properties` definitions

### 2. Update Version

Increment the version number in the schema:
```json
{
  "version": "1.0.1",  // ← Increment this
  // ...
}
```

### 3. Update Documentation

Update `../../docs/WORKFLOW_SCHEMA.md`:
- Document new fields or function types
- Add examples
- Update version references

### 4. Add Tests

Add tests in `../tests/validation_tests.rs`:
- Test that existing workflows still pass
- Test invalid configurations are rejected
- Test new function types work correctly

### 5. Run Tests

```bash
cd ../..
cargo test --manifest-path core/Cargo.toml validation_tests
```

Ensure all tests pass.

### 6. Verify Schema Validity

The schema must be valid JSON Schema Draft 07. You can validate it using:
- Online validators (e.g., https://www.jsonschemavalidator.net/)
- VS Code with a JSON Schema extension

### 7. Commit Changes

```bash
git add core/schema/workflow.schema.json
git add docs/WORKFLOW_SCHEMA.md
git add core/tests/validation_tests.rs
git commit -m "chore: update workflow schema to v1.x.x"
```

## Schema Structure

```json
{
  "type": "object",           // Root workflow object
  "required": ["id", "name", "tasks"],
  "properties": {
    "id": { /* ... */ },
    "name": { /* ... */ },
    "tasks": { /* ... */ }     // Array of Task
  },
  "definitions": {
    "Task": { /* ... */ },
    "Function": {
      "oneOf": [
        "CliCommandFunction",      // CLI command handler
        "CursorAgentFunction",     // Cursor AI agent
        "UserInputFunction",       // User input prompts
        "CustomFunction"           // Custom handlers
      ]
    },
    "CliCommandFunction": { /* ... */ },
    "CursorAgentFunction": { /* ... */ },
    "UserInputFunction": { /* ... */ },
    "CustomFunction": { /* ... */ }
  }
}
```

## Testing the Schema

Run the validation test suite:

```bash
cd ../..
cargo test --manifest-path core/Cargo.toml validation_tests -- --nocapture
```

This will:
- Validate all example workflows in `engine/examples/`
- Test all function types
- Test invalid configurations
- Verify error messages

## Example Workflows

All example workflows in `engine/examples/` should validate against this schema:
- `simple.json`
- `nested.json`
- `parallel.json`
- `user_input_simple.json`
- `user_input_nested.json`
- `user_input_parallel.json`
- `user_input_deep.json`

## Integration

The schema is used by:
- **Core validation** (`core/src/validation/validator.rs`) - Validates workflows before execution
- **GUI upload** (`gui/src/services/workflow.rs`) - Validates uploaded workflows
- **GUI editor** (`gui/src/pages/workflows/edit/handlers.rs`) - Validates edited workflows

All validation happens through `s_e_e_core::validate_workflow_json()`.

