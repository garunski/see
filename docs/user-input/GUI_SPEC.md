# GUI Specification - User Input

## Overview

This document specifies GUI changes for user input support, including component modifications and visual indicators.

## Task Details Panel Modifications

### Input Form Section

**File**: `gui/src/pages/executions/details/components/task_details_panel.rs`

Add input form when task status is `waiting_for_input`:

```rust
if task.status.as_str() == "waiting_for_input" {
    rsx! {
        div {
            class: "mt-4 p-4 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg",
            
            div {
                class: "flex items-center gap-2 mb-3",
                Icon {
                    name: "pause-circle",
                    class: Some("text-amber-600 dark:text-amber-400".to_string()),
                    size: Some("w-5 h-5".to_string()),
                    variant: Some("outline".to_string()),
                }
                h3 {
                    class: "text-amber-900 dark:text-amber-100 font-semibold",
                    "Input Required"
                }
            }
            
            p {
                class: "text-amber-700 dark:text-amber-200 text-sm mb-4",
                "This task is waiting for user input."
            }
            
            UserInputForm {
                task: task.clone(),
                execution_id: execution.clone().map(|e| e.id),
            }
        }
    }
}
```

### UserInputForm Component

**File**: `gui/src/components/forms/user_input_form.rs`

```rust
use dioxus::prelude::*;
use s_e_e_core::{TaskInfo, WorkflowExecution};
use tracing::error;

#[component]
pub fn UserInputForm(task: TaskInfo, execution_id: Option<String>) -> Element {
    let mut input_value = use_signal(String::new);
    let mut is_submitting = use_signal(|| false);
    let mut error_message = use_signal(String::new);

    rsx! {
        div {
            class: "space-y-4",
            
            div {
                class: "space-y-2",
                label {
                    class: "block text-sm font-medium text-amber-900 dark:text-amber-100",
                    "Enter value"
                }
                
                input {
                    r#type: "text",
                    placeholder: "Type your input here...",
                    class: "w-full px-3 py-2 border border-amber-300 dark:border-amber-700 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100",
                    value: "{input_value()}",
                    oninput: move |e| input_value.set(e.value()),
                }
                
                if !error_message().is_empty() {
                    p {
                        class: "text-red-600 dark:text-red-400 text-sm",
                        "{error_message}"
                    }
                }
            }
            
            button {
                class: "w-full px-4 py-2 bg-amber-600 hover:bg-amber-700 text-white rounded-md font-medium transition-colors inline-flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed",
                disabled: is_submitting() || input_value().is_empty(),
                onclick: move |_| {
                    let input = input_value();
                    let exec_id = execution_id.clone();
                    let task_id = task.id.clone();
                    
                    if input.is_empty() {
                        error_message.set("Input cannot be empty".to_string());
                        return;
                    }
                    
                    is_submitting.set(true);
                    
                    spawn(async move {
                        match s_e_e_core::provide_user_input(&exec_id.unwrap_or_default(), &task_id, input).await {
                            Ok(_) => {
                                tracing::info!("Input provided successfully");
                                // TODO: Refresh execution state
                            }
                            Err(e) => {
                                error!("Failed to provide input: {}", e);
                                error_message.set(format!("Failed to provide input: {}", e));
                            }
                        }
                        is_submitting.set(false);
                    });
                },
                Icon {
                    name: "check-circle",
                    class: Some("w-4 h-4".to_string()),
                    size: None,
                    variant: Some("outline".to_string()),
                }
                if is_submitting() {
                    "Submitting..."
                } else {
                    "Submit Input"
                }
            }
        }
    }
}
```

### Registration

**File**: `gui/src/components/forms/mod.rs`

```rust
pub mod user_input_form; // NEW
pub mod workflow_form;

pub use user_input_form::UserInputForm;
```

## Visual Indicators

### Task Status Badge

**File**: `gui/src/pages/executions/details/components/task_details_panel.rs`

Add status badge for `waiting_for_input`:

```rust
match status {
    "complete" => ("bg-green-100 text-green-800", "Complete"),
    "failed" => ("bg-red-100 text-red-800", "Failed"),
    "in-progress" => ("bg-blue-100 text-blue-800", "In Progress"),
    "pending" => ("bg-gray-100 text-gray-800", "Pending"),
    "waiting-for-input" => ("bg-amber-100 text-amber-800 dark:bg-amber-900 dark:text-amber-200", "Waiting for Input"), // NEW
    _ => ("bg-gray-100 text-gray-800", "Unknown"),
}
```

### Workflow Flow Visual Updates

**File**: `gui/src/pages/executions/details/components/workflow_flow.rs`

Update node styling for `WaitingForInput` status:

```rust
// In the node rendering logic
let (background_color, border_color, label_color) = match task.status.as_str() {
    "complete" => ("#D1FAE5", "#34D399", "#065F46"),
    "failed" => ("#FEE2E2", "#F87171", "#991B1B"),
    "in-progress" => ("#DBEAFE", "#60A5FA", "#1E40AF"),
    "waiting-for-input" => ("#FEF3C7", "#F59E0B", "#92400E"), // NEW - Amber colors
    _ => ("#F3F4F6", "#9CA3AF", "#374151"),
};

// Add pulsing animation for waiting state
let className = if status == "waiting-for-input" {
    format!("{} animate-pulse", base_classes)
} else {
    base_classes
};
```

### Task Box Styling

In task list views, add amber border for waiting tasks:

```rust
let border_color = match status {
    "waiting-for-input" => "border-l-4 border-amber-500", // NEW
    "complete" => "border-l-4 border-green-500",
    "failed" => "border-l-4 border-red-500",
    _ => "border-l-4 border-gray-300",
};
```

## Execution Details Page

### Pending Input Count

**File**: `gui/src/pages/executions/details/page.rs`

Add pending input count to header:

```rust
let pending_input_count = execution
    .tasks
    .iter()
    .filter(|t| t.status.as_str() == "waiting_for_input")
    .count();

rsx! {
    div {
        class: "flex items-center justify-between mb-6",
        
        // ... existing header content ...
        
        if pending_input_count > 0 {
            div {
                class: "flex items-center gap-2 px-3 py-1 bg-amber-100 dark:bg-amber-900/30 rounded-full",
                Icon {
                    name: "pause-circle",
                    class: Some("text-amber-600 dark:text-amber-400".to_string()),
                    size: Some("w-4 h-4".to_string()),
                    variant: Some("outline".to_string()),
                }
                span {
                    class: "text-sm font-medium text-amber-800 dark:text-amber-200",
                    "{pending_input_count} input{pending_input_count > 1 {s}} required"
                }
            }
        }
    }
}
```

### Filter/Highlight Tasks

Add toggle to show only tasks waiting for input:

```rust
let mut show_only_waiting = use_signal(|| false);

rsx! {
    div {
        class: "mb-4",
        
        button {
            class: "px-4 py-2 text-sm font-medium rounded-md border border-gray-300 hover:bg-gray-50",
            onclick: move |_| show_only_waiting.set(!show_only_waiting()),
            if show_only_waiting() {
                "Show All Tasks"
            } else {
                "Show Only Waiting for Input"
            }
        }
    }
    
    let filtered_tasks = if show_only_waiting() {
        execution.tasks.iter()
            .filter(|t| t.status.as_str() == "waiting_for_input")
            .cloned()
            .collect()
    } else {
        execution.tasks.clone()
    };
    
    // Render filtered_tasks
}
```

## Icon Updates

Add pause-related icons:

**File**: `gui/src/icons.rs`

```rust
// Add these icon definitions
"pause-circle" => include_str!("../assets/icons/pause-circle.svg"),
"check-circle" => include_str!("../assets/icons/check-circle.svg"),
```

## Component Registration

### Forms Module

**File**: `gui/src/components/forms/mod.rs`

```rust
pub mod prompt_form;
pub mod workflow_form;
pub mod user_input_form; // NEW

pub use user_input_form::UserInputForm;
```

### Component Exports

**File**: `gui/src/components/mod.rs`

```rust
pub use forms::{UserInputForm, ...};
```

## Testing Requirements

### Test Files

1. **`gui/tests/components/user_input_form_tests.rs`**
   - Component renders correctly
   - Input handling
   - Submit button behavior
   - Error display

### Test Coverage

- Form displays when task waiting for input
- Input validation
- Submit triggers API call
- Visual indicators show correctly

## Logging Requirements

```rust
info!("Displaying input form for task {}", task.id);
debug!("User input form submitted with value: {}", value);
trace!("Input form validation: {}", valid);
error!("Failed to submit input: {}", error);
```

## Accessibility

- Input fields have proper labels
- Submit buttons have clear labels
- Error messages are announced to screen readers
- Focus management on form open/close
- Keyboard navigation support

