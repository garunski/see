# State Management Architecture

## Overview

This document describes the current state management architecture, identifies issues, and proposes a solution using `dioxus-query`.

## Current Architecture

### State Provider Pattern

The application uses a context-based `AppStateProvider` pattern:

```rust
pub struct AppStateProvider {
    pub workflow: Signal<WorkflowState>,
    pub ui: Signal<UIState>,
    pub history: Signal<HistoryState>,
    pub settings: Signal<SettingsState>,
    pub prompts: Signal<UserPromptState>,
}
```

### Current Data Flow

```
┌─────────────────────────────────────────────────────────┐
│                    AppStateProvider                      │
│  ┌─────────────┐  ┌─────────────┐  ┌──────────────┐   │
│  │  Signal<T>  │  │  Signal<T>  │  │   Signal<T>  │   │
│  │  Workflow   │  │  History    │  │  Prompts     │   │
│  └──────┬──────┘  └──────┬──────┘  └──────┬───────┘   │
│         │                │                 │           │
└─────────┼────────────────┼─────────────────┼──────────┘
           │                │                 │
           ▼                ▼                 ▼
     Component reads → .clone() → use_memo
```

### Current Problems

#### 1. Excessive Cloning

**File**: `gui/src/hooks/use_app_state.rs`

```rust
pub fn use_workflows() -> Memo<Vec<WorkflowDefinition>> {
    let state = use_app_state();
    use_memo(move || state.settings.read().get_workflows().clone())
    //                                                     ^^^^^^ Problem!
}
```

**Issues**:
- Clones entire vector on every read
- Causes compilation errors with iterators
- Performance impact from unnecessary allocations
- Fighting the borrow checker constantly

**Compilation errors seen**:
```
error[E0382]: use of moved value
error[E0502]: cannot borrow `*state` as mutable
error[E0277]: `for` loop cannot iterate over value
```

#### 2. Naive Polling

**File**: `gui/src/pages/executions/details/hooks.rs`

```rust
const POLLING_INTERVAL_SECS: u64 = 2;

pub fn use_workflow_execution(id: String) -> ... {
    use_effect(move || {
        spawn(async move {
            loop {  // ❌ Polls forever, even when complete!
                match store.get_workflow_with_tasks(&id).await {
                    Ok(exec) => {
                        execution.set(Some(exec.clone()));
                        if matches!(exec.status, Complete | Failed) {
                            break;  // Only stops on terminal states
                        }
                    }
                    Err(e) => { break; }
                }
                tokio::time::sleep(Duration::from_secs(POLLING_INTERVAL_SECS)).await;
            }
        });
    });
}
```

**Issues**:
- Polls every 2 seconds unconditionally
- No caching - fetches fresh data each time
- Network waste for completed workflows
- Manual loop management is error-prone

#### 3. Manual Reload Flags

**File**: `gui/src/state/history_state.rs`

```rust
pub struct HistoryState {
    pub workflow_history: Vec<WorkflowExecutionSummary>,
    pub running_workflows: Vec<WorkflowMetadata>,
    pub needs_history_reload: bool,  // ❌ Manual flag
}
```

**File**: `gui/src/layout/app.rs`

```rust
use_effect(move || {
    let needs_reload = state_provider.history.read().needs_history_reload;
    if needs_reload {
        spawn(async move {
            // Fetch and update...
            history_state.write().needs_history_reload = false;  // ❌ Manual reset
        });
    }
});
```

**Issues**:
- Flags scattered everywhere (`needs_reload`, `needs_history_reload`)
- Easy to forget to set/reset flags
- No automatic invalidation
- Prone to bugs

#### 4. Poor Reactivity

**File**: `gui/src/pages/executions/history/page.rs`

```rust
let workflow_history = use_workflow_history();  // Returns Memo
let running_workflows = use_running_workflows();  // Returns Memo

// Separate into categories
let workflow_categories = use_memo(move || {
    let history = workflow_history();  // Read Memo
    // Clone and partition...
});
```

**Issues**:
- `Signal<State>` forces .read() on every access
- Memo wrappers create unnecessary clones
- UI doesn't update when nested state changes
- Borrow checker prevents direct access

#### 5. Effect Soup

**File**: `gui/src/layout/app.rs`

```rust
// Multiple use_effect hooks doing async work
use_effect(move || { /* Load settings */ });
use_effect(move || { /* Load workflows */ });
use_effect(move || { /* Load history */ });
use_effect(move || { /* Save theme */ });
```

**Issues**:
- Effects firing on every render
- No deduplication of requests
- Can cause infinite loops
- Hard to track dependencies

## Proposed Architecture with dioxus-query

### Data Flow with Queries

```
┌─────────────────────────────────────────────────────────┐
│                    Query Client                          │
│         ┌──────────────────────────────────────┐        │
│         │    QueryCapability Implementations   │        │
│         │  - GetWorkflows                      │        │
│         │  - GetPrompts                        │        │
│         │  - GetWorkflowHistory                │        │
│         │  - GetWorkflowExecution (smart poll)│        │
│         └──────┬────────────┬──────────────────┘       │
│                │            │                            │
│                ▼            ▼                            │
│         ┌──────────┐  ┌──────────────┐                 │
│         │  Cache   │  │  Invalidation│                 │
│         └────┬─────┘  └──────┬───────┘                 │
└──────────────┼───────────────┼──────────────────────────┘
               │               │
               ▼               ▼
     Components use use_query hooks
```

### Query Pattern

**Before** (current):
```rust
let state_provider = use_context::<AppStateProvider>();
use_effect(move || {
    if state_provider.prompts.read().needs_reload {
        spawn(async move {
            match fetch_prompts().await {
                Ok(prompts) => state.prompts.write().load_prompts(prompts),
                Err(e) => { /* ... */ }
            }
        });
    }
});
```

**After** (dioxus-query):
```rust
use dioxus_query::{use_query, Query};

#[derive(Clone, PartialEq, Hash, Eq)]
struct GetPrompts;

impl QueryCapability for GetPrompts {
    type Ok = Vec<Prompt>;
    type Err = String;
    type Keys = ();
    
    async fn run(&self, _: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        fetch_prompts().await
    }
}

// In component:
let prompts = use_query(Query::new((), GetPrompts));
match prompts.read().state() {
    QueryState::Loading => rsx! { "Loading..." },
    QueryState::Success(data) => rsx! { PromptsList { prompts: data } },
    QueryState::Error(err) => rsx! { "Error: {err}" },
}
```

### Smart Polling Pattern

**Before** (manual loop):
```rust
loop {
    let exec = fetch_execution(&id).await?;
    execution.set(Some(exec.clone()));
    if is_terminal_state(&exec) { break; }
    sleep(Duration::from_secs(2)).await;
}
```

**After** (declarative polling):
```rust
#[derive(Clone, PartialEq, Hash, Eq)]
struct GetWorkflowExecution;

impl QueryCapability for GetWorkflowExecution {
    type Ok = WorkflowExecution;
    type Err = String;
    type Keys = String;
    
    async fn run(&self, id: &String) -> Result<Self::Ok, Self::Err> {
        fetch_execution(id).await
    }
}

// In component:
let execution = use_query(
    Query::new(id.clone(), GetWorkflowExecution)
        .interval(Duration::from_secs(2))
        .enabled(!is_terminal(execution.status()))
);

// Auto-stops polling when not needed!
```

### Cache Invalidation Pattern

**Before** (manual flags):
```rust
fn create_workflow(new_workflow: Workflow) {
    spawn(async move {
        store.save_workflow(&new_workflow).await?;
        state.workflows.write().needs_reload = true;  // ❌ Manual
    });
}
```

**After** (automatic):
```rust
use dioxus_query::QueriesStorage;

fn create_workflow(new_workflow: Workflow) {
    spawn(async move {
        store.save_workflow(&new_workflow).await?;
        // Auto-invalidates GetWorkflows cache!
        QueriesStorage::<GetWorkflows>::invalidate_matching(&()).await;
    });
}
```

### Mutation Pattern

```rust
pub fn use_create_workflow_mutation() -> Signal<QueryState<WorkflowDefinition>> {
    let query_state = use_signal(QueryState::default);
    
    spawn(async move {
        query_state.write().set_loading(true);
        match save_workflow().await {
            Ok(workflow) => {
                // Invalidate cache
                QueriesStorage::<GetWorkflows>::invalidate_matching(&()).await;
                query_state.write().set_success(workflow);
            }
            Err(err) => query_state.write().set_error(err),
        }
    });
    
    query_state
}
```

## Key Architectural Changes

### 1. Replace Signal<State> with Queries

**Remove**:
- `AppStateProvider` context
- `use_memo(move || state.read().get_workflows().clone())`
- Manual `needs_reload` flags

**Add**:
- `QueryCapability` implementations for each data type
- `use_query` hooks that return `QueryState<T>`
- Automatic caching and invalidation

### 2. Declarative Polling

**Remove**:
- Manual `loop { fetch(); sleep(); }` patterns
- `tokio::time::sleep` in hooks
- Manual status checks to stop polling

**Add**:
- `.interval(Duration)` on query declarations
- `.enabled(bool)` for conditional polling
- Auto-stop when not needed

### 3. Automatic Cache Management

**Remove**:
- Manual cache invalidation flags
- `.clone()` everywhere to avoid borrow errors
- Separate loading/error state management

**Add**:
- Built-in cache with TTL
- `QueriesStorage::invalidate_matching()`
- Unified `QueryState<T>` for loading/error/data

### 4. Simplified Component State

**Before**:
```rust
let state = use_app_state();
let prompts = use_memo(move || state.prompts.read().get_prompts().clone());
let needs_reload = use_memo(move || state.prompts.read().needs_reload);
use_effect(move || {
    if needs_reload() { /* ... */ }
});
```

**After**:
```rust
let prompts = use_query(Query::new((), GetPrompts));
// That's it! Caching, loading, error handling all built-in.
```

## Benefits

✅ **No cloning errors** - Proper ownership with query results  
✅ **Smart polling** - Auto-stops when not needed  
✅ **Automatic caching** - No redundant fetches  
✅ **Better reactivity** - UI updates on cache invalidation  
✅ **Cleaner code** - No boilerplate for async state management  
✅ **Type safety** - Compile-time guarantees for data flow  
✅ **Focus-based refetch** - Auto-refresh on window focus  

## Migration Strategy

1. Start with simplest data (prompts)
2. Add query capabilities incrementally (SRP: one concern at a time)
3. Keep old code alongside during transition
4. Update components one by one
5. Remove old code after all components migrated
6. **Run `task quality` after each phase** to catch issues early
7. **Write unit tests** for helper functions and query logic (separate from UI tests)

This phased approach ensures we can test each piece in isolation before removing the old system.

## Single Responsibility Principle (SRP)

Each module in the new architecture follows SRP:

### Query Capabilities
- **One query = One data type**
- Each `QueryCapability` implementation has single responsibility: fetch specific data
- No business logic in queries, only data fetching

### Helpers Module
- **Pure functions only**
- Status checks and utilities
- No dependencies on UI or business logic
- Fully testable with unit tests

### Mutation Helpers
- **One mutation = One data operation**
- Each mutation:
  1. Saves data (SRP)
  2. Invalidates cache (SRP)
- No UI rendering logic

### Hooks
- **Thin wrappers** around queries
- No complex logic in hooks
- UI-specific hooks stay separate from data hooks

