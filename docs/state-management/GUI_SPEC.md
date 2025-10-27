# GUI Specification - State Management Refactor

## Overview

This document maps out component changes and UI patterns for the dioxus-query integration.

## Component Changes

### Prompts List Page

**File**: `gui/src/pages/prompts/list/page.rs`

**Before**:
```rust
use crate::hooks::use_prompts;
use crate::state::AppStateProvider;

pub fn UserPromptsListPage() -> Element {
    let state_provider = use_context::<AppStateProvider>();
    let prompts = use_prompts();  // Returns Memo<Vec<Prompt>>

    // Manual loading logic
    let state_provider_clone = state_provider.clone();
    use_effect(move || {
        if state_provider_clone.prompts.read().needs_reload {
            let mut state_provider = state_provider_clone.clone();
            spawn(async move {
                match UserPromptService::fetch_prompts().await {
                    Ok(loaded_prompts) => {
                        state_provider.prompts.write().load_prompts(loaded_prompts);
                    }
                    Err(e) => {
                        tracing::error!("Failed to load prompts: {}", e);
                    }
                }
            });
        }
    });

    rsx! {
        div { class: "space-y-8",
            // Render prompts...
        }
    }
}
```

**After**:
```rust
use dioxus::prelude::*;
use dioxus_query::{use_query, Query, QueryState};
use crate::queries::GetPrompts;

pub fn UserPromptsListPage() -> Element {
    let prompts = use_query(Query::new((), GetPrompts));

    rsx! {
        div { class: "space-y-8",
            match prompts.read().state() {
                QueryState::Loading => {
                    rsx! { LoadingSpinner {} }
                }
                QueryState::Success(data) => {
                    rsx! { PromptsList { prompts: data.clone() } }
                }
                QueryState::Error(err) => {
                    rsx! { ErrorMessage { error: err.clone() } }
                }
            }
        }
    }
}
```

**Changes**:
- ❌ Remove `use_context::<AppStateProvider>`
- ❌ Remove `use_prompts()` memo hook
- ❌ Remove `use_effect` manual loading
- ❌ Remove `needs_reload` flag check
- ✅ Add `use_query` with GetPrompts
- ✅ Handle loading/error states declaratively

### History Page

**File**: `gui/src/pages/executions/history/page.rs`

**Before**:
```rust
use crate::hooks::use_running_workflows;

pub fn HistoryPage() -> Element {
    let (is_loading, error, refresh_data) = use_history_data();
    
    // Manual refresh trigger
    let refresh_data_for_effect = refresh_data.clone();
    use_effect(move || {
        refresh_data_for_effect();
    });

    let workflow_history = use_workflow_history();
    let running_workflows = use_running_workflows();

    rsx! {
        // ...
    }
}
```

**After**:
```rust
use dioxus_query::{use_query, Query};
use std::time::Duration;
use crate::queries::{GetWorkflowHistory, GetRunningWorkflows};

pub fn HistoryPage() -> Element {
    let history = use_query(Query::new((), GetWorkflowHistory));
    let running = use_query(
        Query::new((), GetRunningWorkflows)
            .interval(Duration::from_secs(5))  // Auto-polls every 5s
    );

    rsx! {
        div { class: "space-y-8",
            // Render running workflows (auto-refreshes)
            if let Some(running_data) = running.read().data() {
                rsx! {
                    RunningWorkflowsList { workflows: running_data }
                }
            }
            
            // Render history
            if let Some(history_data) = history.read().data() {
                rsx! {
                    CompletedWorkflowsList { executions: history_data }
                }
            }
        }
    }
}
```

**Changes**:
- ❌ Remove `use_history_data()` hook
- ❌ Remove manual `refresh_data()` call
- ❌ Remove `use_effect` for refresh
- ❌ Remove loading/error state management
- ✅ Add `use_query` with polling for running workflows
- ✅ No manual refresh button needed (auto-polls)

### Execution Details Page

**File**: `gui/src/pages/executions/details/page.rs`

**Before**:
```rust
pub fn use_workflow_execution(id: String) -> (Signal<Option<WorkflowExecution>>, Signal<bool>, Signal<Option<String>>) {
    let execution = use_signal(|| None::<WorkflowExecution>);
    let loading = use_signal(|| true);
    let error = use_signal(|| None::<String>);

    use_effect(move || {
        spawn(async move {
            loop {  // ❌ Polls forever!
                match store.get_workflow_with_tasks(&id).await {
                    Ok(exec) => {
                        execution.set(Some(exec.clone()));
                        loading.set(false);
                        if matches!(exec.status, Complete | Failed) {
                            break;
                        }
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed: {}", e)));
                        loading.set(false);
                        break;
                    }
                }
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        });
    });

    (execution, loading, error)
}
```

**After**:
```rust
use dioxus_query::{use_query, Query, QueriesStorage};
use std::time::Duration;
use crate::queries::{GetWorkflowExecution, is_terminal_status};

pub fn use_workflow_execution(id: String) -> Signal<QueryState<WorkflowExecution>> {
    let execution = use_query(
        Query::new(id.clone(), GetWorkflowExecution)
            .interval(Duration::from_secs(2))
    );

    // Stop polling when execution completes
    use_effect(move || {
        if let Some(exec) = execution.read().data() {
            if is_terminal_status(&exec.status) {
                spawn(async move {
                    QueriesStorage::<GetWorkflowExecution>::invalidate_matching(&id).await;
                });
            }
        }
    });

    execution
}

// In component:
pub fn ExecutionDetailsPage() -> Element {
    let execution = use_workflow_execution(id.clone());

    match execution.read().state() {
        QueryState::Loading => rsx! { "Loading execution..." },
        QueryState::Success(data) => rsx! { ExecutionView { execution: data } },
        QueryState::Error(err) => rsx! { "Error: {err}" },
    }
}
```

**Changes**:
- ❌ Remove manual polling loop
- ❌ Remove `tokio::time::sleep`
- ❌ Remove manual loading/error management
- ✅ Add `use_query` with `.interval()`
- ✅ Auto-stops polling on terminal states
- ✅ Cleaner error handling

## Hook Replacements

### use_workflows()

**File**: `gui/src/hooks/use_app_state.rs`

**Before**:
```rust
pub fn use_workflows() -> Memo<Vec<WorkflowDefinition>> {
    let state = use_app_state();
    use_memo(move || state.settings.read().get_workflows().clone())  // ❌ Clones!
}
```

**After**:
```rust
use dioxus_query::{use_query, Query};
use crate::queries::GetWorkflows;

pub fn use_workflows_query() -> Signal<QueryState<Vec<WorkflowDefinition>>> {
    use_query(Query::new((), GetWorkflows))
}
```

### use_prompts()

**Before**:
```rust
pub fn use_prompts() -> Memo<Vec<Prompt>> {
    let state = use_app_state();
    use_memo(move || state.prompts.read().get_prompts().clone())  // ❌ Clones!
}
```

**After**:
```rust
use dioxus_query::{use_query, Query};
use crate::queries::GetPrompts;

pub fn use_prompts_query() -> Signal<QueryState<Vec<Prompt>>> {
    use_query(Query::new((), GetPrompts))
}
```

### use_workflow_history()

**Before**:
```rust
pub fn use_workflow_history() -> Memo<Vec<WorkflowExecutionSummary>> {
    let state = use_app_state();
    use_memo(move || state.history.read().workflow_history.clone())  // ❌ Clones!
}
```

**After**:
```rust
use dioxus_query::{use_query, Query};
use crate::queries::GetWorkflowHistory;

pub fn use_workflow_history_query() -> Signal<QueryState<Vec<WorkflowExecutionSummary>>> {
    use_query(Query::new((), GetWorkflowHistory))
}
```

### use_running_workflows()

**Before**:
```rust
pub fn use_running_workflows() -> Memo<Vec<WorkflowMetadata>> {
    let state = use_app_state();
    use_memo(move || state.history.read().running_workflows.clone())  // ❌ Clones!
}
```

**After**:
```rust
use dioxus_query::{use_query, Query};
use std::time::Duration;
use crate::queries::GetRunningWorkflows;

pub fn use_running_workflows_query() -> Signal<QueryState<Vec<WorkflowMetadata>>> {
    use_query(
        Query::new((), GetRunningWorkflows)
            .interval(Duration::from_secs(5))  // Auto-polls
    )
}
```

## UI Patterns

### Loading State

**Pattern**:
```rust
match query.read().state() {
    QueryState::Loading => {
        rsx! {
            div { class: "flex items-center justify-center p-8",
                div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" }
            }
        }
    }
    QueryState::Success(data) => { /* ... */ }
    QueryState::Error(err) => { /* ... */ }
}
```

### Error State

**Pattern**:
```rust
match query.read().state() {
    QueryState::Error(err) => {
        rsx! {
            div { class: "bg-red-50 dark:bg-red-900/20 p-4 rounded",
                h3 { "Error loading data" }
                p { "{err}" }
            }
        }
    }
    _ => {}
}
```

### Empty State

**Pattern**:
```rust
match query.read().state() {
    QueryState::Success(data) if data.is_empty() => {
        rsx! {
            EmptyState { message: "No items yet" }
        }
    }
    QueryState::Success(data) => {
        rsx! {
            ItemsList { items: data }
        }
    }
    _ => {}
}
```

## Mutation Integration

### Create Workflow

**File**: `gui/src/pages/workflows/edit/handlers.rs`

**Before**:
```rust
fn handle_save_workflow() {
    spawn(async move {
        match save_workflow(&workflow).await {
            Ok(_) => {
                // Navigate away
            }
            Err(e) => {
                error_message.set(Some(e.to_string()));
            }
        }
    });
}
```

**After**:
```rust
use crate::queries::mutations::create_workflow_mutation;

fn handle_save_workflow() {
    spawn(async move {
        match create_workflow_mutation(workflow.clone()).await {
            Ok(_) => {
                // Cache automatically invalidated, UI auto-refreshes
                navigator.push(Route::WorkflowsListPage);
            }
            Err(e) => {
                error_message.set(Some(e));
            }
        }
    });
}
```

### Delete Workflow

```rust
use crate::queries::mutations::{delete_workflow, QueriesStorage};
use crate::queries::GetWorkflows;

fn handle_delete_workflow(id: String) {
    spawn(async move {
        match delete_workflow(&id).await {
            Ok(_) => {
                // Invalidate cache - UI auto-refreshes
                QueriesStorage::<GetWorkflows>::invalidate_matching(&()).await;
            }
            Err(e) => { /* ... */ }
        }
    });
}
```

## App Initialization

**File**: `gui/src/layout/app.rs`

**Before**:
```rust
pub fn AppContent() -> Element {
    let mut state_provider = use_hook(AppStateProvider::new);
    use_context_provider(|| state_provider.clone());

    // Multiple use_effect hooks for loading data
    use_effect(move || { /* Load settings */ });
    use_effect(move || { /* Load workflows */ });
    use_effect(move || { /* Load history */ });

    rsx! { /* ... */ }
}
```

**After**:
```rust
pub fn AppContent() -> Element {
    // No initialization needed!
    // Queries are loaded on-demand when components mount
    // This is a significant simplification

    rsx! { 
        Router::<Route> {}
    }
}
```

## State Management Files

### Remove

- `gui/src/hooks/use_app_state.rs` - No longer needed
- `gui/src/state/mod.rs` - Simplified
- Manual reload flags in state structs

### Keep (for UI-only state)

- `gui/src/state/ui_state.rs` - For UI state (file picker, etc.)
- Small local signals for component-specific state

## Benefits Summary

### Code Reduction

- ❌ Remove ~200 lines of state management boilerplate
- ❌ Remove manual polling loops
- ❌ Remove `needs_reload` flag management
- ❌ Remove excessive `use_effect` hooks
- ✅ Declarative query declarations
- ✅ Automatic loading/error handling

### Performance

- ✅ No redundant clones
- ✅ Automatic request deduplication
- ✅ Smart caching
- ✅ Conditional polling

### Maintainability

- ✅ Centralized query logic
- ✅ Easy to add new queries
- ✅ Consistent error handling
- ✅ Type-safe throughout

