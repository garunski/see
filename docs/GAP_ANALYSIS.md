# Gap Analysis: GUI Needs vs Engine Provides

## Summary
Comparing GUI requirements with Engine capabilities to identify missing types and needed bridges.

## GUI Needs (from GUI_REQUIREMENTS.md)

### Types GUI Expects from Core
- `WorkflowDefinition` - Workflow templates with metadata
- `WorkflowExecution` - Full execution records  
- `WorkflowExecutionSummary` - Lightweight execution summaries
- `WorkflowMetadata` - Basic workflow metadata
- `TaskExecution` - Individual task execution records
- `TaskInfo` - Task information for UI display
- `UserPrompt` - User-defined prompts
- `AuditEvent` - Audit trail entries
- `AppSettings` - Application configuration
- `WorkflowJson` - Raw workflow JSON content
- `WorkflowResult` - Execution result from core
- `WorkflowStatus` - Execution status enum
- `Theme` - UI theme enum (Light, Dark, System)
- `AuditStatus` - Audit entry status (Success, Failure)

### Store Methods GUI Expects
- Workflow CRUD: `save_workflow`, `get_workflow`, `list_workflows`, `delete_workflow`
- Execution CRUD: `save_workflow_execution`, `get_workflow_execution`, `list_workflow_executions`, `delete_workflow_execution`
- Metadata: `list_workflow_metadata`, `delete_workflow_metadata_and_tasks`, `get_workflow_with_tasks`
- Tasks: `save_task_execution`, `get_tasks_for_workflow`
- Prompts: `save_prompt`, `list_prompts`, `delete_prompt`
- Settings: `load_settings`, `save_settings`
- Audit: `log_audit_event`
- Utility: `clear_all_data`

### Core API Functions GUI Expects
- `execute_workflow_by_id(workflow_id: &str, callback: Option<OutputCallback>) -> Result<WorkflowResult, CoreError>`
- `resume_task(execution_id: &str, task_id: &str) -> Result<(), CoreError>`
- `WorkflowDefinition::get_default_workflows() -> Vec<WorkflowDefinition>`
- `init_tracing(Option<String>) -> Result<TracingGuard, String>`
- `init_global_store() -> Result<(), String>`
- `get_global_store() -> Result<Arc<Store>, String>`

## Engine Provides (from ENGINE_INTERFACE.md)

### Types Engine Exports
- `EngineWorkflow` - Workflow structure for execution
- `EngineTask` - Task structure with recursive next_tasks
- `TaskFunction` - Task function types
- `TaskResult` - Result of task execution
- `WorkflowResult` - Result of workflow execution
- `TaskInfo` - Task information for results
- `AuditEntry` - Audit trail entry
- `ExecutionContext` - Execution context for handlers
- `TaskStatus` - Task execution status enum
- `AuditStatus` - Audit entry status enum

### Functions Engine Exports
- `execute_workflow_from_json(json: &str) -> Result<WorkflowResult, EngineError>`
- `parse_workflow(json: &str) -> Result<EngineWorkflow, ParserError>`

## GAPS IDENTIFIED

### 1. Missing Persistence Types
**GUI needs but Engine doesn't provide:**
- `WorkflowDefinition` - Workflow templates with metadata (id, name, description, content, is_default, is_edited, created_at, updated_at)
- `WorkflowExecution` - Full execution records (id, workflow_name, status, created_at, completed_at, success, tasks, timestamp)
- `WorkflowExecutionSummary` - Lightweight execution summaries
- `WorkflowMetadata` - Basic workflow metadata (id, name, status)
- `TaskExecution` - Individual task execution records (id, workflow_id, name, status, output, error, created_at, completed_at)
- `UserPrompt` - User-defined prompts (id, name, content, created_at, updated_at)
- `AuditEvent` - Audit trail entries (id, task_id, status, timestamp, changes_count, message)
- `AppSettings` - Application configuration (theme, auto_save, notifications, default_workflow)
- `WorkflowJson` - Raw workflow JSON content
- `WorkflowStatus` - Execution status enum (Pending, Running, Complete, Failed)
- `Theme` - UI theme enum (Light, Dark, System)

### 2. Missing Store Interface
**GUI expects but Engine doesn't provide:**
- Complete Store trait with all CRUD operations
- Global store singleton pattern
- Settings management
- Prompt management
- Audit event logging

### 3. Missing Core API Functions
**GUI expects but Engine doesn't provide:**
- `execute_workflow_by_id()` - Execute workflow by stored ID
- `resume_task()` - Resume paused task execution
- `WorkflowDefinition::get_default_workflows()` - Get default workflow templates
- `init_tracing()` - Initialize logging system
- `init_global_store()` - Initialize persistence layer
- `get_global_store()` - Get global store instance

### 4. Type Mismatches
**Engine provides but needs transformation:**
- `EngineWorkflow` → `WorkflowDefinition` (needs metadata fields)
- `WorkflowResult` → `WorkflowExecution` (needs persistence fields)
- `TaskResult` → `TaskExecution` (needs persistence fields)
- `AuditEntry` → `AuditEvent` (needs id field)

### 5. Missing Bridge Types
**Need to create:**
- `CoreError` - Core error type wrapping Engine errors
- `OutputCallback` - Callback type for streaming output
- Conversion functions between Engine and Persistence types

## BRIDGE REQUIREMENTS

### 1. Persistence Layer
- Create `persistence` crate with `redb` database
- Define all missing types with proper serialization
- Implement Store trait with all required methods
- Handle multi-process concurrent access

### 2. Core Orchestration Layer
- Create `core` crate that coordinates Engine + Persistence
- Implement `execute_workflow_by_id()` by loading from persistence, executing via engine, saving results
- Implement `resume_task()` for paused task resumption
- Implement global store singleton pattern
- Re-export all types GUI expects

### 3. Type Conversions
- `EngineWorkflow` ↔ `WorkflowDefinition`
- `WorkflowResult` → `WorkflowExecution`
- `TaskResult` → `TaskExecution`
- `AuditEntry` → `AuditEvent`

### 4. Error Handling
- `CoreError` wrapping `EngineError` and persistence errors
- Consistent error propagation to GUI services

## SUCCESS CRITERIA
✓ All GUI-expected types are available from core
✓ All GUI-expected store methods are implemented
✓ All GUI-expected core API functions are implemented
✓ Type conversions work seamlessly
✓ Multi-process concurrent access works correctly
✓ Error handling is consistent throughout
