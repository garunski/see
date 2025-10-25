# Core and Persistence Implementation Plan

## Testing Requirements

### Universal Testing Standards
- **ALL tests MUST be in separate test files** in `/tests` directory
- **NO tests in the same files as implementation code**
- **Each phase must pass its own unit tests AND all previous phase tests**
- **All tests must be deterministic and isolated**
- **Tests must clean up resources (temp files, databases)**
- **Tests must cover error conditions and edge cases**

### Test Organization Structure
```
<crate>/
├── src/
│   ├── lib.rs
│   ├── <module>.rs
│   └── ...
└── tests/
    ├── <module>_tests.rs
    ├── integration_tests.rs
    ├── concurrency_tests.rs
    └── logging_tests.rs
```

### Logging Requirements
- **ALL major code paths must have appropriate logging**
- Use `tracing` crate with structured logging
- Include TRACE, DEBUG, INFO, WARN, ERROR levels as appropriate
- Log method entry/exit, data transformations, errors
- Log performance metrics (query duration, serialization time)
- All logging must be testable and configurable

## Phase 1: Complete Design Documents

### Step 1: GUI Analysis Document
- [x] Create `GUI_REQUIREMENTS.md` by analyzing all GUI files
- Extract all types imported from `s_e_e_core`
- Document all functions called on global store
- List all struct fields and their types
- Document all enums and their variants
- Record function signatures GUI services expect

### Step 2: Engine Analysis Document  
- [x] Create `ENGINE_INTERFACE.md` by analyzing engine crate
- Document engine's input types (EngineWorkflow, EngineTask)
- Document engine's output types (WorkflowResult, TaskResult, AuditEntry)
- Record engine's execution function signatures
- Document engine's error types

### Step 3: Gap Analysis Document
- [x] Create `GAP_ANALYSIS.md` comparing GUI needs vs Engine provides
- Identify types GUI needs that engine doesn't provide
- Identify types engine provides that need transformation
- List persistence types needed that neither has
- Define bridge/adapter types needed

### Step 4: Type Specifications
- [x] Create `TYPES_SPEC.md` based on analysis
- Define all persistence models with exact field names/types
- Define all core bridge types
- Define all enums with variants
- Specify required derive traits
- Define serialization requirements

### Step 5: Persistence Specification
- [x] Create `PERSISTENCE_SPEC.md` based on types
- Define `redb` database schema (typed tables with key/value types)
- Define Store struct with Database handle
- Define read/write transaction patterns for concurrency
- Define multi-process reader strategy
- Define Store API methods matching GUI expectations
- Define error types
- Define table definitions for workflows, executions, tasks, prompts, audit_events, settings

### Step 6: Core API Specification
- [x] Create `CORE_API_SPEC.md` based on GUI requirements
- Define public API functions
- Define global store singleton pattern
- Define re-exports strategy
- Define error handling

### Step 7: Orchestration Flow Specification
- [x] Create `ORCHESTRATION_SPEC.md` for execution flow
- Define step-by-step execution flow
- Define state transitions
- Define persistence points
- Define type conversions

### Step 8: Bridge Mappings Specification
- [x] Create `BRIDGE_SPEC.md` for type conversions
- Define field-by-field type conversions
- Define transformation rules
- Define data mapping tables

## Phase 2: Create Persistence Crate ✅

### Step 9: Create Persistence Directory Structure
- [ ] Create `persistence/` directory
- [ ] Create `persistence/src/` directory
- [ ] Create `persistence/Cargo.toml` with redb dependencies
- [ ] Create `persistence/src/lib.rs` with public exports
- [ ] Create `persistence/src/models.rs` with all data types
- [ ] Create `persistence/src/errors.rs` with PersistenceError
- [ ] Create `persistence/src/store.rs` with Store implementation

### Step 10: Implement Persistence Models
- [ ] Implement WorkflowDefinition struct
- [ ] Implement WorkflowExecution struct
- [ ] Implement WorkflowExecutionSummary struct
- [ ] Implement WorkflowMetadata struct
- [ ] Implement TaskExecution struct
- [ ] Implement UserPrompt struct
- [ ] Implement AuditEvent struct
- [ ] Implement AppSettings struct
- [ ] Implement WorkflowStatus enum
- [ ] Implement Theme enum
- [ ] Add all required derive traits (Debug, Clone, PartialEq, Serialize, Deserialize)

### Step 11: Implement Persistence Errors
- [ ] Implement PersistenceError enum
- [ ] Add error variants for Database, Serialization, Io, Transaction
- [ ] Implement error conversions from redb errors
- [ ] Implement Display trait for error messages

### Step 12: Implement Store with redb
- [ ] Define table schemas using redb macros
- [ ] Implement Store struct with Database handle
- [ ] Implement Store::new() constructor
- [ ] Implement workflow CRUD methods (save_workflow, get_workflow, list_workflows, delete_workflow)
- [ ] Implement execution CRUD methods (save_workflow_execution, get_workflow_execution, list_workflow_executions, delete_workflow_execution)
- [ ] Implement metadata methods (list_workflow_metadata, delete_workflow_metadata_and_tasks, get_workflow_with_tasks)
- [ ] Implement task methods (save_task_execution, get_tasks_for_workflow)
- [ ] Implement prompt methods (save_prompt, list_prompts, delete_prompt)
- [ ] Implement settings methods (load_settings, save_settings)
- [ ] Implement audit methods (log_audit_event)
- [ ] Implement utility methods (clear_all_data)

### Step 13: Test Persistence Crate
- [ ] Test persistence crate builds - `cargo build -p persistence`
- [ ] **Run ALL unit tests** - `cargo test -p persistence`
- [ ] **Test logging functionality** - verify all major code paths have appropriate logging
- [ ] **Test error handling** - verify all error conditions are properly handled and logged
- [ ] **Test multi-process concurrent access** - verify WAL mode and connection pooling work correctly
- [ ] **Test resource cleanup** - verify temp databases are cleaned up after tests
- [ ] **Test edge cases** - empty results, not found, invalid data, serialization failures
- [ ] **Test performance** - verify query performance and connection pool efficiency
- [ ] **Test isolation** - each test must be independent and deterministic

## Phase 3: Create Core Crate

### Step 14: Create Core Directory Structure
- [ ] Create `core/` directory
- [ ] Create `core/src/` directory
- [ ] Create `core/Cargo.toml` with dependencies (persistence, engine)
- [ ] Create `core/src/lib.rs` with re-exports
- [ ] Create `core/src/errors.rs` with CoreError
- [ ] Create `core/src/bridge.rs` with type conversions
- [ ] Create `core/src/store_singleton.rs` with global store
- [ ] Create `core/src/api.rs` with execution functions

### Step 15: Implement Core Error Types
- [ ] Implement CoreError enum
- [ ] Add error variants for Engine, Persistence, WorkflowNotFound, TaskNotFound, Execution
- [ ] Implement error conversions from engine and persistence errors
- [ ] Implement Display trait for error messages

### Step 16: Implement Type Bridges
- [ ] Implement EngineWorkflow ↔ WorkflowDefinition conversion
- [ ] Implement WorkflowResult → WorkflowExecution conversion
- [ ] Implement TaskResult → TaskExecution conversion
- [ ] Implement AuditEntry → AuditEvent conversion
- [ ] Implement WorkflowDefinition → EngineWorkflow conversion
- [ ] Add conversion functions for all type mappings

### Step 17: Implement Global Store Singleton
- [ ] Implement global store singleton pattern using OnceLock
- [ ] Implement init_store() function
- [ ] Implement get_global_store() function
- [ ] Ensure thread-safe access
- [ ] Handle initialization errors

### Step 18: Implement Core Re-exports
- [ ] Re-export all persistence types
- [ ] Re-export engine types (TaskInfo, TaskStatus, AuditStatus)
- [ ] Re-export bridge types (WorkflowResult, OutputCallback)
- [ ] Re-export error types (CoreError)
- [ ] Re-export function types (OutputCallback)

### Step 19: Test Core Crate
- [ ] Test core crate builds - `cargo build -p core`
- [ ] **Run ALL unit tests** - `cargo test -p core`
- [ ] **Run ALL persistence tests** - `cargo test -p persistence` (must still pass)
- [ ] **Test logging functionality** - verify all major code paths have appropriate logging
- [ ] **Test type conversions** - verify all bridge conversions work correctly
- [ ] **Test global store singleton** - verify thread-safe initialization and access
- [ ] **Test error handling** - verify all error conditions are properly handled and logged
- [ ] **Test resource cleanup** - verify temp databases are cleaned up after tests
- [ ] **Test edge cases** - invalid conversions, missing data, serialization failures
- [ ] **Test isolation** - each test must be independent and deterministic

## Phase 4: Update Workspace

### Step 20: Update Workspace Configuration
- [ ] Update root `Cargo.toml` to include persistence and core members
- [ ] Update workspace dependencies if needed
- [ ] **Test workspace builds** - `cargo build`
- [ ] **Run ALL tests** - `cargo test` (must pass all persistence and core tests)
- [ ] **Test logging functionality** - verify logging works across all crates
- [ ] **Test error handling** - verify error propagation works correctly
- [ ] **Test resource cleanup** - verify all temp resources are cleaned up

## Phase 5: Implement Core API

### Step 21: Implement Workflow Execution API
- [ ] Implement `execute_workflow_by_id()` function
- [ ] Load workflow definition from persistence
- [ ] Convert WorkflowDefinition to EngineWorkflow
- [ ] Create WorkflowExecution record (status: Running)
- [ ] Save initial execution state to persistence
- [ ] Call engine to execute workflow
- [ ] Stream progress updates via callback
- [ ] Convert WorkflowResult to WorkflowExecution
- [ ] Update execution status (Complete/Failed)
- [ ] Save final state to persistence
- [ ] Return WorkflowResult

### Step 22: Implement Task Resumption API
- [ ] Implement `resume_task()` function
- [ ] Load execution and task from persistence
- [ ] Validate task is in WaitingForInput status
- [ ] Resume task execution via engine
- [ ] Update task status in persistence
- [ ] Return success/error result

### Step 23: Implement Default Workflows
- [ ] Implement `WorkflowDefinition::get_default_workflows()` method
- [ ] Define default workflow templates
- [ ] Return Vec<WorkflowDefinition>

### Step 24: Implement Initialization Functions
- [ ] Implement `init_tracing()` function
- [ ] Implement `init_global_store()` function
- [ ] Handle initialization errors
- [ ] Return appropriate result types

### Step 25: Test Core API
- [ ] Test core API builds - `cargo test -p core`
- [ ] **Run ALL tests** - `cargo test` (must pass all previous phase tests)
- [ ] **Test workflow execution end-to-end** - verify complete execution flow works
- [ ] **Test task resumption** - verify task resumption works correctly
- [ ] **Test default workflows** - verify default workflow templates work
- [ ] **Test initialization functions** - verify tracing and store initialization work
- [ ] **Test logging functionality** - verify all API calls are properly logged
- [ ] **Test error handling** - verify all error conditions are properly handled and logged
- [ ] **Test resource cleanup** - verify all temp resources are cleaned up
- [ ] **Test edge cases** - invalid workflows, missing data, execution failures
- [ ] **Test isolation** - each test must be independent and deterministic

## Phase 6: Integration Testing

### Step 26: Test GUI Integration
- [ ] Test GUI can import from core - `cargo build -p gui`
- [ ] **Run ALL tests** - `cargo test` (must pass all previous phase tests)
- [ ] **Test logging functionality** - verify logging works across GUI and core
- [ ] **Test error handling** - verify error propagation from core to GUI works
- [ ] **Test resource cleanup** - verify all temp resources are cleaned up
- [ ] Verify all GUI services work without modification
- [ ] Test workflow execution from GUI
- [ ] Test task resumption from GUI
- [ ] Test settings management from GUI
- [ ] Test prompt management from GUI

### Step 27: Test End-to-End Workflow Execution
- [ ] **Run ALL tests** - `cargo test` (must pass all previous phase tests)
- [ ] **Test logging functionality** - verify complete execution flow is properly logged
- [ ] **Test error handling** - verify all error conditions are properly handled and logged
- [ ] **Test resource cleanup** - verify all temp resources are cleaned up
- [ ] Test complete workflow execution flow
- [ ] Test data persists to database
- [ ] Test execution history retrievable
- [ ] Test audit trail logging
- [ ] Test error handling throughout

### Step 28: Test Multi-Process Database Access
- [ ] **Run ALL tests** - `cargo test` (must pass all previous phase tests)
- [ ] **Test logging functionality** - verify multi-process access is properly logged
- [ ] **Test error handling** - verify concurrent access errors are properly handled
- [ ] **Test resource cleanup** - verify all temp resources are cleaned up
- [ ] Test multiple GUI processes can read simultaneously
- [ ] Test no blocking between readers
- [ ] Test writers don't block readers
- [ ] Test concurrent access works correctly

### Step 29: Verify All Requirements Met
- [ ] **Run ALL tests** - `cargo test` (must pass all tests)
- [ ] **Test logging functionality** - verify all major code paths have appropriate logging
- [ ] **Test error handling** - verify all error conditions are properly handled and logged
- [ ] **Test resource cleanup** - verify all temp resources are cleaned up
- [ ] Verify all GUI-expected types are available from core
- [ ] Verify all GUI-expected store methods are implemented
- [ ] Verify all GUI-expected core API functions are implemented
- [ ] Verify type conversions work seamlessly
- [ ] Verify multi-process concurrent access works correctly
- [ ] Verify error handling is consistent throughout

## Success Criteria

✓ All design documents completed
✓ **ALL tests in separate `/tests` directories** (no tests in implementation files)
✓ **ALL major code paths have appropriate logging** (TRACE, DEBUG, INFO, WARN, ERROR)
✓ **Each phase passes its own tests AND all previous phase tests**
✓ **All tests are deterministic, isolated, and clean up resources**
✓ Persistence crate compiles and passes tests
✓ Core crate compiles and passes tests
✓ Workspace builds successfully
✓ Core API functions work correctly
✓ GUI services work without modification
✓ Workflow execution persists to database
✓ Execution history retrievable from database
✓ Global store accessible from GUI/CLI
✓ Multi-process concurrent access works
✓ Type bridges between engine and persistence work correctly
✓ **Logging works across all crates and major operations**
✓ **Error handling is comprehensive and properly logged**
✓ **All edge cases and error conditions are tested**
