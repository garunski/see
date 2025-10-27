# Phase 4: Replace Execution Details

**Estimated Time**: 2-3 hours (including human testing)

## Goal

Add execution details query and replace polling loop. Create helpers when needed.

## Steps

### Step 4.1: Create Helper Functions

**File**: `gui/src/queries/helpers.rs` (create NOW because we need it)

```rust
use s_e_e_core::WorkflowExecutionStatus;

pub fn is_terminal_status(status: &WorkflowExecutionStatus) -> bool {
    matches!(
        status,
        WorkflowExecutionStatus::Complete | WorkflowExecutionStatus::Failed
    )
}
```

**File**: `gui/src/queries/mod.rs` (add helpers)

```rust
pub mod prompt_queries;
pub mod history_queries;
pub mod helpers;  // NEW

pub use prompt_queries::*;
pub use history_queries::*;
pub use helpers::*;  // NEW
```

### Step 4.2: Create GetWorkflowExecution Query

**File**: `gui/src/queries/history_queries.rs` (add to existing file)

Add this query to the existing file:

```rust
#[derive(Clone, PartialEq, Hash, Eq)]
pub struct GetWorkflowExecution(Captured<()>);

impl QueryCapability for GetWorkflowExecution {
    type Ok = WorkflowExecution;
    type Err = String;
    type Keys = String;

    async fn run(&self, id: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        let store = get_global_store()
            .map_err(|e| e.to_string())?;
        store.get_workflow_with_tasks(id)
            .await
            .map_err(|e| e.to_string())
    }
}
```

**Usage with smart polling**:
```rust
use dioxus_query::prelude::*;
use std::time::Duration;

let execution = use_query(
    Query::new(id.clone(), GetWorkflowExecution(Captured(())))
        .interval(Duration::from_secs(2))
);
```

**Validation**: `task quality`

### Step 4.3: Replace Execution Details Hook

**File**: `gui/src/pages/executions/details/hooks.rs`

Replace `use_workflow_execution` function.

**Validation**: Code compiles

### Step 4.4: Human UI Testing (YOU TEST THIS)

**Test Checklist**:
1. Navigate to running execution details
2. Check network tab
3. Verify requests every 2 seconds
4. Complete the workflow
5. Verify requests STOP
6. Refresh page
7. Verify only one request (no polling)

**Validation**: All tests pass

### Step 4.5: Quality Checks

```bash
task quality
```

## Success Criteria

✅ Execution details queries work  
✅ Smart polling stops when complete  
✅ Human testing confirms behavior  

**Important**: Do NOT create workflow mutations yet. Wait until Phase 5.

