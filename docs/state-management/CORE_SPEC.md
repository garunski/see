# Core Specification - dioxus-query Integration

## Overview

This document specifies the query capabilities, mutations, and cache invalidation strategies for the dioxus-query integration.

## Query Capabilities

### GetWorkflows

Fetches all workflow definitions from the database.

**File**: `gui/src/queries/workflow_queries.rs`

```rust
use dioxus_query::QueryCapability;
use s_e_e_core::{WorkflowDefinition, get_global_store};

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct GetWorkflows;

impl QueryCapability for GetWorkflows {
    type Ok = Vec<WorkflowDefinition>;
    type Err = String;
    type Keys = ();  // No parameters for list

    async fn run(&self, _: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        let store = get_global_store()
            .map_err(|e| e.to_string())?;
        store.list_workflows()
            .await
            .map_err(|e| e.to_string())
    }
}

// Optional: Fetch single workflow
#[derive(Clone, PartialEq, Hash, Eq)]
pub struct GetWorkflow;

impl QueryCapability for GetWorkflow {
    type Ok = WorkflowDefinition;
    type Err = String;
    type Keys = String;  // Workflow ID

    async fn run(&self, id: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        let store = get_global_store()
            .map_err(|e| e.to_string())?;
        store.get_workflow_by_id(id)
            .await
            .map_err(|e| e.to_string())
    }
}
```

**Usage**:
```rust
let workflows = use_query(Query::new((), GetWorkflows));
```

**Cache Invalidation**:
```rust
// After creating a new workflow
QueriesStorage::<GetWorkflows>::invalidate_matching(&()).await;

// After updating a specific workflow
QueriesStorage::<GetWorkflow>::invalidate_matching(id).await;
```

### GetPrompts

Fetches all user prompts from the database.

**File**: `gui/src/queries/prompt_queries.rs`

```rust
use dioxus_query::QueryCapability;
use s_e_e_core::{Prompt, get_global_store};

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct GetPrompts;

impl QueryCapability for GetPrompts {
    type Ok = Vec<Prompt>;
    type Err = String;
    type Keys = ();

    async fn run(&self, _: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        let store = get_global_store()
            .map_err(|e| e.to_string())?;
        
        // Fetch from user_prompts table
        // Implementation depends on existing API
        store.list_prompts()
            .await
            .map_err(|e| e.to_string())
    }
}
```

**Usage**:
```rust
let prompts = use_query(Query::new((), GetPrompts));
```

### GetWorkflowHistory

Fetches completed workflow executions.

**File**: `gui/src/queries/history_queries.rs`

```rust
use dioxus_query::QueryCapability;
use s_e_e_core::{WorkflowExecutionSummary, get_global_store};

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct GetWorkflowHistory;

impl QueryCapability for GetWorkflowHistory {
    type Ok = Vec<WorkflowExecutionSummary>;
    type Err = String;
    type Keys = ();

    async fn run(&self, _: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        let store = get_global_store()
            .map_err(|e| e.to_string())?;
        
        let executions = store.list_workflow_executions()
            .await
            .map_err(|e| e.to_string())?;
        
        // Convert to summaries
        Ok(executions.into_iter().map(|exec| {
            WorkflowExecutionSummary {
                id: exec.id,
                workflow_name: exec.workflow_name,
                status: exec.status,
                created_at: exec.created_at,
                completed_at: exec.completed_at,
                task_count: exec.tasks.len(),
                timestamp: exec.timestamp,
            }
        }).collect())
    }
}
```

**Usage**:
```rust
let history = use_query(Query::new((), GetWorkflowHistory));
```

### GetRunningWorkflows

Fetches currently running workflows with smart polling.

**File**: `gui/src/queries/history_queries.rs`

```rust
use std::time::Duration;
use dioxus_query::QueryCapability;
use s_e_e_core::{WorkflowMetadata, get_global_store};

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct GetRunningWorkflows;

impl QueryCapability for GetRunningWorkflows {
    type Ok = Vec<WorkflowMetadata>;
    type Err = String;
    type Keys = ();

    async fn run(&self, _: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        let store = get_global_store()
            .map_err(|e| e.to_string())?;
        
        let metadata = store.list_workflow_metadata()
            .await
            .map_err(|e| e.to_string())?;
        
        // Filter for only truly running workflows
        let all_execution_ids = store.list_workflow_executions()
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|exec| exec.id)
            .collect::<std::collections::HashSet<_>>();
        
        Ok(metadata.into_iter()
            .filter(|m| m.status == "running" && !all_execution_ids.contains(&m.id))
            .collect())
    }
}
```

**Usage with polling**:
```rust
let running = use_query(
    Query::new((), GetRunningWorkflows)
        .interval(Duration::from_secs(5))  // Poll every 5 seconds
        .cache_time(Duration::from_secs(30))  // Cache for 30 seconds
);
```

### GetWorkflowExecution

Fetches single workflow execution with smart polling that stops when complete.

**File**: `gui/src/queries/history_queries.rs`

```rust
use std::time::Duration;
use dioxus_query::QueryCapability;
use s_e_e_core::{WorkflowExecution, WorkflowExecutionStatus, get_global_store};

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct GetWorkflowExecution;

impl QueryCapability for GetWorkflowExecution {
    type Ok = WorkflowExecution;
    type Err = String;
    type Keys = String;  // Execution ID

    async fn run(&self, id: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        let store = get_global_store()
            .map_err(|e| e.to_string())?;
        store.get_workflow_with_tasks(id)
            .await
            .map_err(|e| e.to_string())
    }
}

// Helper function to check if execution is terminal
pub fn is_terminal_status(status: &WorkflowExecutionStatus) -> bool {
    matches!(status, 
        WorkflowExecutionStatus::Complete | 
        WorkflowExecutionStatus::Failed
    )
}
```

**Usage with smart polling**:
```rust
use dioxus_query::{use_query, Query};
use crate::queries::{GetWorkflowExecution, is_terminal_status};

let execution = use_query({
    let id = execution_id.clone();
    let query = Query::new(id.clone(), GetWorkflowExecution);
    
    // Poll every 2 seconds
    let query = query.interval(Duration::from_secs(2));
    
    // Conditionally enable polling
    if let Some(exec) = execution.data() {
        if is_terminal_status(&exec.status) {
            query.disabled()  // Stop polling when complete
        } else {
            query.enabled()
        }
    } else {
        query.enabled()  // Always enable when loading
    }
});
```

**Better approach** - Use query state to conditionally poll:

```rust
let execution_signal = use_signal(|| None::<WorkflowExecution>);
let execution = use_query(
    Query::new(id.clone(), GetWorkflowExecution)
        .interval(Duration::from_secs(2))
);

use_effect(move || {
    if let Some(exec) = execution.read().data() {
        execution_signal.set(Some(exec.clone()));
        
        // Invalidate query to stop polling when complete
        if is_terminal_status(&exec.status) {
            QueriesStorage::<GetWorkflowExecution>::invalidate_matching(&id).await;
        }
    }
});
```

## Mutation Helpers

### Create Workflow Mutation

**File**: `gui/src/queries/mutations.rs`

```rust
use dioxus_query::QueriesStorage;
use s_e_e_core::WorkflowDefinition;

pub async fn create_workflow_mutation(workflow: WorkflowDefinition) -> Result<WorkflowDefinition, String> {
    let store = get_global_store()
        .map_err(|e| e.to_string())?;
    
    store.save_workflow(&workflow)
        .await
        .map_err(|e| e.to_string())?;
    
    // Invalidate workflows cache
    QueriesStorage::<GetWorkflows>::invalidate_matching(&()).await;
    
    Ok(workflow)
}

// Alternative: Hook-based approach
pub fn use_create_workflow_mutation() -> Signal<QueryState<WorkflowDefinition>> {
    let mut query_state = use_signal(|| QueryState::<WorkflowDefinition>::default());
    
    let mutate_fn = move |workflow: WorkflowDefinition| {
        spawn(async move {
            match create_workflow_mutation(workflow).await {
                Ok(workflow) => {
                    query_state.write().set_success(workflow);
                }
                Err(err) => {
                    query_state.write().set_error(err);
                }
            }
        });
    };
    
    query_state
}
```

### Update Workflow Mutation

```rust
pub async fn update_workflow_mutation(id: String, content: String) -> Result<WorkflowDefinition, String> {
    let store = get_global_store()
        .map_err(|e| e.to_string())?;
    
    let mut workflow = store.get_workflow_by_id(&id)
        .await
        .map_err(|e| e.to_string())?;
    
    workflow.content = content;
    workflow.updated_at = chrono::Utc::now();
    
    store.save_workflow(&workflow)
        .await
        .map_err(|e| e.to_string())?;
    
    // Invalidate both list and specific workflow
    QueriesStorage::<GetWorkflows>::invalidate_matching(&()).await;
    QueriesStorage::<GetWorkflow>::invalidate_matching(&id).await;
    
    Ok(workflow)
}
```

### Create Prompt Mutation

```rust
pub async fn create_prompt_mutation(prompt: Prompt) -> Result<Prompt, String> {
    let store = get_global_store()
        .map_err(|e| e.to_string())?;
    
    store.save_prompt(&prompt)
        .await
        .map_err(|e| e.to_string())?;
    
    // Invalidate prompts cache
    QueriesStorage::<GetPrompts>::invalidate_matching(&()).await;
    
    Ok(prompt)
}
```

## Cache Invalidation Strategy

### Automatic Invalidation

Queries automatically invalidate when:
1. Query data changes
2. Window regains focus (refetch_on_focus)
3. Network reconnects (if implemented)

### Manual Invalidation

Use `QueriesStorage::invalidate_matching()` after mutations:

```rust
// Invalidate all workflows
QueriesStorage::<GetWorkflows>::invalidate_matching(&()).await;

// Invalidate specific workflow
QueriesStorage::<GetWorkflow>::invalidate_matching(&id).await;

// Pattern-based invalidation (if supported)
QueriesStorage::<GetWorkflowExecution>::invalidate_matching(&prefix).await;
```

### Invalidation Patterns

**Pattern 1**: Invalidate list after create
```rust
create_workflow() → invalidate GetWorkflows → UI auto-refreshes
```

**Pattern 2**: Invalidate specific item after update
```rust
update_workflow(id) → invalidate GetWorkflow(id) → UI auto-updates
```

**Pattern 3**: Invalidate multiple related queries
```rust
delete_execution(id) → 
    invalidate GetWorkflowHistory → 
    invalidate GetWorkflowExecution(id)
```

## Query Options

### Cache Configuration

```rust
Query::new((), GetWorkflows)
    .cache_time(Duration::from_secs(60))  // Cache for 60 seconds
    .stale_time(Duration::from_secs(30))  // Consider stale after 30 seconds
```

### Polling Configuration

```rust
Query::new((), GetRunningWorkflows)
    .interval(Duration::from_secs(5))  // Poll every 5 seconds
    .enabled(running_workflows.len() > 0)  // Only poll when running items exist
```

### Retry Configuration

```rust
Query::new(id, GetWorkflowExecution)
    .retry(3)  // Retry 3 times on failure
    .retry_delay(Duration::from_secs(1))  // Wait 1 second between retries
```

## File Structure

Following SRP (Single Responsibility Principle):

```
gui/src/
├── queries/
│   ├── mod.rs                    # Export all queries
│   ├── workflow_queries.rs       # SRP: Workflow data fetching
│   ├── prompt_queries.rs         # SRP: Prompt data fetching
│   ├── history_queries.rs        # SRP: History data fetching
│   ├── helpers.rs                 # SRP: Status check utilities
│   └── mutations.rs               # SRP: Mutation operations + invalidation
├── hooks/
│   └── use_queries.rs            # SRP: Hook wrappers for convenience
└── tests/
    └── queries/
        └── helpers_tests.rs      # SRP: Tests for helper functions
```

**SRP Principles Applied**:
- Each query capability has ONE responsibility (fetching specific data)
- Helpers module has ONE responsibility (status checks, no business logic)
- Mutations have ONE responsibility (data modification + invalidation)
- Tests are separated by concern (not mixed with UI rendering tests)

## Type Safety

All query results are typed at compile time:

```rust
// Type-safe results
QueryState<Vec<WorkflowDefinition>>
QueryState<Vec<Prompt>>
QueryState<WorkflowExecution>

// Compile-time guarantees
prompts.data()  // Returns &Vec<Prompt>
workflows.data()  // Returns &Vec<WorkflowDefinition>
execution.data()  // Returns &WorkflowExecution

// Error types are also typed
prompts.error()  // Returns &String (from QueryCapability::Err)
```

## Testing Strategy

### Unit Tests for Non-UI Code

**Test Location**: `gui/tests/queries/`

**Testable Components** (NOT UI-specific):

1. **Helper Functions** (`helpers.rs`)
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use s_e_e_core::WorkflowExecutionStatus;

    #[test]
    fn is_terminal_status_complete() {
        assert!(is_terminal_status(&WorkflowExecutionStatus::Complete));
    }

    #[test]
    fn is_terminal_status_failed() {
        assert!(is_terminal_status(&WorkflowExecutionStatus::Failed));
    }

    #[test]
    fn is_terminal_status_not_running() {
        assert!(!is_terminal_status(&WorkflowExecutionStatus::Running));
    }

    #[test]
    fn should_poll_active_execution() {
        assert!(should_poll(&WorkflowExecutionStatus::Running));
    }

    #[test]
    fn should_not_poll_complete_execution() {
        assert!(!should_poll(&WorkflowExecutionStatus::Complete));
    }
}
```

2. **Query Capabilities** (Can be tested with mock stores)
```rust
#[cfg(test)]
mod workflow_tests {
    use super::*;

    #[tokio::test]
    async fn get_workflows_returns_results() {
        // Mock store or use test database
        // Test QueryCapability::run() method
    }

    #[tokio::test]
    async fn get_workflows_handles_errors() {
        // Test error handling in run() method
    }
}
```

3. **Mutation Helpers** (Can test logic without UI)
```rust
#[cfg(test)]
mod mutation_tests {
    use super::*;

    #[tokio::test]
    async fn create_workflow_invalidates_cache() {
        // Test that invalidation happens
    }

    #[tokio::test]
    async fn update_workflow_invalidates_specific_cache() {
        // Test specific invalidation
    }
}
```

**NOT Testable** (UI-specific):
- Component rendering
- Hook usage patterns
- Visual loading states
These require integration tests or manual testing.

### Running Tests

```bash
# Run all tests including new query tests
cargo test queries --lib

# Run specific test module
cargo test helpers_tests

# Run all tests
cargo test

# Quality check includes tests
task quality
```

## Error Handling

```rust
let result = use_query(Query::new((), GetWorkflows));

match result.read().state() {
    QueryState::Loading { .. } => {
        rsx! { LoadingSpinner {} }
    }
    QueryState::Success(data) => {
        rsx! { WorkflowsList { workflows: data } }
    }
    QueryState::Error(err) => {
        rsx! { ErrorMessage { error: err.clone() } }
    }
}
```

## Performance Considerations

1. **Cache reuse**: Queries with same key reuse cached data
2. **Background refetch**: Update cache without blocking UI
3. **Conditional polling**: Only poll when necessary
4. **Stale-while-revalidate**: Show cached data while fetching fresh
5. **Request deduplication**: Multiple components requesting same data = one fetch

