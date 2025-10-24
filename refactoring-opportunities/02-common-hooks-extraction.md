# Common Hooks Extraction

## Current State
- **Files**: Multiple page components
- **Priority**: 🔄 HIGH - Will benefit all pages immediately

## Problems

### 1. Repeated Patterns Across Pages
Every page has similar patterns that could be extracted:

```rust
// Repeated in every page:
let state_provider = use_context::<AppStateProvider>();

// Repeated data loading pattern:
let state_provider_clone = state_provider.clone();
use_effect(move || {
    if state_provider_clone.some_state.read().needs_reload {
        let mut state_provider = state_provider_clone.clone();
        spawn(async move {
            match SomeService::fetch_data().await {
                Ok(data) => {
                    state_provider.some_state.write().load_data(data);
                }
                Err(e) => {
                    tracing::error!("Failed to load data: {}", e);
                }
            }
        });
    }
});
```

### 2. Duplicate Validation Logic
- Similar validation patterns in edit pages
- Repeated error handling
- Inconsistent validation feedback

### 3. Common State Management
- Similar signal patterns
- Repeated async data loading
- Duplicate error state management

## Refactoring Plan

### Create `gui/src/hooks/` Directory
```
hooks/
├── mod.rs
├── use_app_state.rs          // Centralized state access
├── use_async_data.rs         // Generic async data loading
├── use_validation.rs         // Common validation patterns
├── use_confirmation_dialog.rs // Delete confirmation patterns
├── use_error_handling.rs     // Error state management
└── use_form_state.rs         // Form state management
```

### 1. `use_app_state` Hook
```rust
// Centralized state access with convenience methods
pub fn use_app_state() -> AppStateProvider {
    use_context::<AppStateProvider>()
}

pub fn use_workflows() -> Signal<Vec<WorkflowDefinition>> {
    let state = use_app_state();
    use_memo(move || state.settings.read().get_workflows().clone())
}

pub fn use_prompts() -> Signal<Vec<Prompt>> {
    let state = use_app_state();
    use_memo(move || state.prompts.read().get_prompts().clone())
}
```

### 2. `use_async_data` Hook
```rust
pub struct AsyncDataState<T> {
    pub data: Signal<Option<T>>,
    pub loading: Signal<bool>,
    pub error: Signal<Option<String>>,
    pub refresh: impl Fn(),
}

pub fn use_async_data<F, Fut, T>(
    fetcher: F,
    auto_load: bool,
) -> AsyncDataState<T>
where
    F: Fn() -> Fut + 'static,
    Fut: Future<Output = Result<T, String>> + 'static,
    T: Clone + 'static,
{
    // Generic async data loading with loading states
}
```

### 3. `use_validation` Hook
```rust
pub struct ValidationState {
    pub errors: Signal<HashMap<String, String>>,
    pub is_valid: Signal<bool>,
    pub validate: impl Fn() -> bool,
    pub clear_errors: impl Fn(),
    pub set_error: impl Fn(String, String),
}

pub fn use_validation() -> ValidationState {
    // Centralized validation logic
}
```

### 4. `use_confirmation_dialog` Hook
```rust
pub struct ConfirmationDialogState {
    pub show: Signal<bool>,
    pub title: Signal<String>,
    pub message: Signal<String>,
    pub on_confirm: EventHandler<()>,
    pub on_cancel: EventHandler<()>,
    pub show_dialog: impl Fn(String, String, EventHandler<()>),
}

pub fn use_confirmation_dialog() -> ConfirmationDialogState {
    // Reusable confirmation dialog logic
}
```

### 5. `use_error_handling` Hook
```rust
pub struct ErrorHandlingState {
    pub error: Signal<Option<String>>,
    pub show_error: impl Fn(String),
    pub clear_error: impl Fn(),
    pub retry: impl Fn(),
}

pub fn use_error_handling() -> ErrorHandlingState {
    // Centralized error handling
}
```

## Implementation Examples

### Before (PromptsListPage)
```rust
#[component]
pub fn PromptsListPage() -> Element {
    let state_provider = use_context::<AppStateProvider>();
    let prompts = use_memo(move || {
        let prompts = state_provider.prompts.read().get_prompts().clone();
        prompts
    });
    let mut show_delete_dialog = use_signal(|| false);
    let mut prompt_to_delete = use_signal(String::new);

    // Load prompts on mount
    let state_provider_clone = state_provider.clone();
    use_effect(move || {
        if state_provider_clone.prompts.read().needs_reload {
            let mut state_provider = state_provider_clone.clone();
            spawn(async move {
                match PromptService::fetch_prompts().await {
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
    // ... more boilerplate
}
```

### After (PromptsListPage)
```rust
#[component]
pub fn PromptsListPage() -> Element {
    let prompts = use_prompts();
    let async_state = use_async_data(
        || PromptService::fetch_prompts(),
        true,
    );
    let confirmation = use_confirmation_dialog();
    let error_handling = use_error_handling();

    // Much cleaner and more focused
}
```

## Benefits

1. **DRY Principle**: Eliminate repeated code across pages
2. **Consistency**: Standardized patterns across the app
3. **Maintainability**: Changes to common patterns in one place
4. **Testability**: Hooks can be tested independently
5. **Developer Experience**: Easier to write new pages

## Migration Strategy

1. **Create hook files** - Implement common hooks
2. **Update one page at a time** - Start with simplest pages
3. **Test thoroughly** - Ensure functionality is preserved
4. **Refactor remaining pages** - Apply patterns to all pages
5. **Clean up** - Remove old patterns and optimize

## Success Metrics

- Reduce boilerplate code by 60%
- Standardize error handling across all pages
- Improve consistency of user experience
- Faster development of new pages

## Failed Attempts and Lessons Learned

### Attempt 1: Confirmation Dialog Hook (Phase 2)

**What was attempted:**
- Created `use_confirmation_dialog` hook with complex state management
- Tried to use `Box<dyn FnMut>` for closures in struct fields
- Attempted to migrate PromptsListPage and SettingsPage

**Why it failed:**
1. **Closure ownership issues**: The `Box<dyn FnMut>` closures couldn't be moved into the rsx! macro due to Rust's ownership rules
2. **Complex state management**: The hook tried to manage too much state internally, making it difficult to use in different contexts
3. **Field access confusion**: Signal fields needed to be called as functions `(confirmation.show)()` but this was error-prone
4. **Move semantics**: The closures couldn't be moved into the component's rsx! macro because they were captured by the `FnMut` closure

**Error messages encountered:**
```
error[E0507]: cannot move out of value, a captured variable in an `FnMut` closure
error[E0596]: cannot borrow `confirmation.show_dialog` as mutable, as it is not declared as mutable
error[E0277]: the trait bound `dyn FnMut(String, String, String): Clone` is not satisfied
```

**What was learned:**
- Dioxus hooks with complex closure management are difficult to implement
- Simple inline patterns are often better than over-engineered hooks
- The confirmation dialog pattern is better left as inline code rather than extracted

### Attempt 2: App State Hook (Phase 1)

**What was attempted:**
- Created `use_app_state` hook with convenience methods
- Migrated HomePage to use `use_workflows()`

**Why it partially succeeded:**
- This actually worked well and was successful
- The pattern of `use_workflows() -> Memo<Vec<WorkflowDefinition>>` is clean and useful
- HomePage migration was successful

**What was learned:**
- Simple data access hooks work well
- Memoized state access is a good pattern to extract
- Start with the simplest patterns first

### Current State

**What works:**
- `use_app_state` hook with `use_workflows()`, `use_prompts()`, etc.
- HomePage successfully migrated to use the app state hook
- Basic state access patterns are working

**What doesn't work:**
- Complex confirmation dialog hooks with closure management
- Any hook that tries to return closures in struct fields
- Hooks that require complex state management with multiple signals

**Recommendations for next agent:**
1. **Stick to simple patterns**: Only extract hooks that return simple data types or signals
2. **Avoid closure hooks**: Don't try to create hooks that return closures or complex state management
3. **Focus on data access**: The `use_app_state` pattern works well - expand on this
4. **Keep confirmation dialogs inline**: The current inline pattern is actually cleaner and more maintainable
5. **Test incrementally**: Run `task quality` after every single change, not just after major phases

**Next steps:**
- Continue with simple data access hooks (use_workflow_history, use_running_workflows)
- Skip complex state management hooks
- Focus on reducing boilerplate in data access patterns only
