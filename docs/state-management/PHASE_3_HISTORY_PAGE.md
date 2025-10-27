# Phase 3: Replace History Page

**Estimated Time**: 3-4 hours (including human testing)

## Goal

Add history queries and replace the history page with polling. Create ONLY the query files needed for this page.

## Steps

### Step 3.1: Create History Queries (ONLY THESE TWO)

**File**: `gui/src/queries/mod.rs` (add to existing)

```rust
pub mod prompt_queries;
pub mod history_queries;  // NEW

pub use prompt_queries::*;
pub use history_queries::*;  // NEW
```

**File**: `gui/src/queries/history_queries.rs` (create this file)

```rust
use dioxus_query::prelude::*;
use s_e_e_core::{WorkflowExecutionSummary, WorkflowMetadata, get_global_store};

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct GetWorkflowHistory(Captured<()>);

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

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct GetRunningWorkflows(Captured<()>);

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
        
        Ok(metadata.into_iter()
            .filter(|m| m.status == "running")
            .collect())
    }
}
```

**Usage with polling**:
```rust
use dioxus_query::prelude::*;
use std::time::Duration;

let history = use_query(Query::new((), GetWorkflowHistory(Captured(()))));

let running = use_query(
    Query::new((), GetRunningWorkflows(Captured(())))
        .interval(Duration::from_secs(5))  // Poll every 5s
);
```

**Validation**: `task quality`

### Step 3.2: Update History Page

Replace history page to use queries with polling.

**Validation**: Code compiles

### Step 3.3: Human UI Testing (YOU TEST THIS)

**Test Checklist**:
1. Navigate to /executions
2. Verify history loads
3. Start a workflow execution
4. Return to history page
5. Verify it appears in running section
6. Wait 10 seconds WITHOUT clicking anything
7. Verify auto-updates (new system)
8. Complete the workflow
9. Verify it automatically moves to completed section
10. Check network tab - polling every 5s

**Validation**: All tests pass

### Step 3.4: Quality Checks

```bash
task quality
```

## Success Criteria

✅ History queries created  
✅ History page works with polling  
✅ Auto-updates work  
✅ Human testing confirms no regressions  

**Important**: Do NOT create execution details query yet. Wait until Phase 4.

