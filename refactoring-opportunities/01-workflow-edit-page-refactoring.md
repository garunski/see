# WorkflowEditPage Refactoring (CRITICAL)

## Current State
- **File**: `gui/src/pages/workflow/edit/page.rs`
- **Lines**: 317 lines
- **Priority**: 🚨 CRITICAL - Immediate refactoring needed

## Problems

### 1. Massive Single Component
- 317 lines in one component violates single responsibility principle
- Handles both Visual and JSON editing modes in one place
- Complex state management with 10+ different signals
- Mixed concerns: UI rendering, validation, state management, business logic

### 2. State Management Complexity
```rust
// Too many signals in one component:
let mut content = use_signal(String::new);
let validation_error = use_signal(String::new);
let is_saving = use_signal(|| false);
let mut can_reset = use_signal(|| false);
let mut workflow_name = use_signal(String::new);
let mut edited_workflow_name = use_signal(String::new);
let mut has_unsaved_changes = use_signal(|| false);
let mut original_content = use_signal(String::new);
let mut original_name = use_signal(String::new);
let edit_mode = use_signal(|| EditMode::Visual);
let selected_node_info = use_signal(|| String::from("No node selected"));
```

### 3. Duplicate Rendering Logic
- Visual editor and JSON editor have completely different rendering
- Iframe setup logic is complex and repeated
- Mode switching logic is scattered throughout

### 4. Complex Effects
- Multiple `use_effect` hooks with complex dependencies
- Validation logic mixed with UI updates
- Unsaved changes tracking is overly complex

## Refactoring Plan

### Phase 1: Extract Core Components
```
workflow/edit/
├── page.rs                    // Main orchestrator (50 lines)
├── components/
│   ├── mod.rs
│   ├── editor_header.rs       // Header with mode toggle and actions
│   ├── visual_editor.rs       // Visual editing mode
│   ├── json_editor.rs         // JSON editing mode
│   ├── validation_display.rs  // Error display component
│   └── iframe_wrapper.rs      // Reusable iframe component
├── hooks/
│   ├── mod.rs
│   ├── use_workflow_edit.rs   // Main edit state management
│   ├── use_validation.rs      // Validation logic
│   ├── use_mode_switching.rs  // Mode switching logic
│   └── use_unsaved_changes.rs // Unsaved changes tracking
└── handlers.rs                // Keep existing handlers
```

### Phase 2: State Management Refactoring

#### Extract `use_workflow_edit` Hook
```rust
pub fn use_workflow_edit(id: String) -> WorkflowEditState {
    // Centralized state management
    // Handles loading, saving, validation
    // Returns structured state object
}
```

#### Extract `use_validation` Hook
```rust
pub fn use_validation(content: Signal<String>) -> ValidationState {
    // JSON validation
    // Field validation
    // Error message management
}
```

#### Extract `use_mode_switching` Hook
```rust
pub fn use_mode_switching() -> ModeSwitchingState {
    // Mode switching logic
    // Content transformation between modes
    // Validation before switching
}
```

### Phase 3: Component Extraction

#### EditorHeader Component
```rust
#[component]
pub fn EditorHeader(
    is_new: bool,
    edit_mode: Signal<EditMode>,
    can_reset: Signal<bool>,
    is_saving: Signal<bool>,
    has_unsaved_changes: Signal<bool>,
    on_mode_switch: EventHandler<EditMode>,
    on_save: EventHandler<()>,
    on_reset: EventHandler<()>,
    on_back: EventHandler<()>,
) -> Element
```

#### VisualEditor Component
```rust
#[component]
pub fn VisualEditor(
    content: Signal<String>,
    workflow_name: Signal<String>,
    selected_node_info: Signal<String>,
) -> Element
```

#### JsonEditor Component
```rust
#[component]
pub fn JsonEditor(
    content: Signal<String>,
    on_content_change: EventHandler<String>,
    validation_error: Signal<String>,
) -> Element
```

## Benefits

1. **Maintainability**: Each component has single responsibility
2. **Testability**: Smaller components are easier to test
3. **Reusability**: Components can be reused in other contexts
4. **Readability**: Code is easier to understand and navigate
5. **Performance**: Smaller components can be optimized individually

## Implementation Steps

1. **Create hook files** - Extract state management logic
2. **Create component files** - Extract UI components
3. **Update main page** - Use extracted hooks and components
4. **Test thoroughly** - Ensure all functionality works
5. **Clean up** - Remove unused code and optimize

## Success Metrics

- Main page component < 100 lines
- Each extracted component < 150 lines
- All functionality preserved
- No performance regression
- Improved test coverage
