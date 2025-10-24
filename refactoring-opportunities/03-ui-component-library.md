# UI Component Library Extraction

## Current State
- **Files**: Multiple page components with repeated UI patterns
- **Priority**: 🎨 MEDIUM - Will improve consistency and maintainability

## Problems

### 1. Repeated Form Patterns
Every edit page has similar form structures:

```rust
// Repeated in PromptEditPage, WorkflowEditPage, etc.
div {
    label { class: "block text-sm font-medium text-zinc-900 dark:text-white mb-2",
        "Field Label"
    }
    input {
        r#type: "text",
        value: "{field_value()}",
        oninput: move |evt| field_value.set(evt.value()),
        placeholder: "Enter value...",
        class: "block w-full rounded-md border-0 py-1.5 text-zinc-900 dark:text-white shadow-sm ring-1 ring-inset ring-zinc-300 dark:ring-zinc-600 placeholder:text-zinc-400 dark:placeholder:text-zinc-500 focus:ring-2 focus:ring-inset focus:ring-blue-600 dark:bg-zinc-700 sm:text-sm sm:leading-6"
    }
    if !validation_error().is_empty() {
        div { class: "mt-2 text-sm text-red-600 dark:text-red-400",
            {validation_error()}
        }
    }
}
```

### 2. Duplicate Table/List Rendering
Similar table patterns across list pages:

```rust
// Repeated in PromptsListPage, WorkflowsListPage, etc.
table { class: "min-w-full divide-y divide-zinc-200 dark:divide-zinc-700",
    thead { class: "bg-zinc-50 dark:bg-zinc-700",
        tr {
            th { class: "px-6 py-3 text-left text-xs font-medium text-zinc-500 dark:text-zinc-300 uppercase tracking-wider", "Column 1" }
            th { class: "px-6 py-3 text-left text-xs font-medium text-zinc-500 dark:text-zinc-300 uppercase tracking-wider", "Column 2" }
            // ... more columns
        }
    }
    tbody { class: "bg-white dark:bg-zinc-800 divide-y divide-zinc-200 dark:divide-zinc-700",
        for item in items().iter() {
            tr { class: "hover:bg-zinc-50 dark:hover:bg-zinc-700",
                // ... row content
            }
        }
    }
}
```

### 3. Inconsistent Loading States
Different loading patterns across pages:

```rust
// Various loading implementations
if loading() {
    div { class: "flex items-center justify-center h-full",
        div { class: "text-center",
            div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4" }
            p { class: "text-zinc-600 dark:text-zinc-400", "Loading..." }
        }
    }
}
```

### 4. Duplicate Error Display
Similar error handling UI across components:

```rust
// Repeated error display patterns
if !error_message().is_empty() {
    div { class: "p-4 bg-red-50 dark:bg-red-900/20 rounded-lg border border-red-200 dark:border-red-700",
        p { class: "text-sm text-red-700 dark:text-red-300", "{error_message()}" }
    }
}
```

## Refactoring Plan

### Create `gui/src/components/common/` Directory
```
components/
├── common/
│   ├── mod.rs
│   ├── forms/
│   │   ├── mod.rs
│   │   ├── text_input.rs       // Reusable text input with validation
│   │   ├── textarea_input.rs   // Reusable textarea
│   │   ├── form_section.rs     // Form section wrapper
│   │   └── form_actions.rs     // Save/Cancel button group
│   ├── data_display/
│   │   ├── mod.rs
│   │   ├── data_table.rs       // Generic table component
│   │   ├── loading_state.rs    // Unified loading component
│   │   ├── empty_state.rs      // Empty state component
│   │   └── pagination.rs       // Pagination component
│   ├── feedback/
│   │   ├── mod.rs
│   │   ├── error_display.rs    // Error display component
│   │   ├── success_toast.rs    // Success notifications
│   │   └── validation_message.rs // Validation feedback
│   └── layout/
│       ├── mod.rs
│       ├── page_header.rs      // Standard page header
│       ├── section_card.rs     // Card wrapper for sections
│       └── action_bar.rs       // Action button bar
```

### 1. Form Components

#### TextInput Component
```rust
#[derive(Props, PartialEq, Clone)]
pub struct TextInputProps {
    pub label: String,
    pub value: Signal<String>,
    pub on_change: EventHandler<String>,
    pub placeholder: Option<String>,
    pub validation_error: Option<Signal<String>>,
    pub required: bool,
    pub disabled: bool,
}

#[component]
pub fn TextInput(props: TextInputProps) -> Element {
    // Standardized text input with validation
}
```

#### TextareaInput Component
```rust
#[derive(Props, PartialEq, Clone)]
pub struct TextareaInputProps {
    pub label: String,
    pub value: Signal<String>,
    pub on_change: EventHandler<String>,
    pub placeholder: Option<String>,
    pub rows: Option<usize>,
    pub validation_error: Option<Signal<String>>,
    pub required: bool,
}

#[component]
pub fn TextareaInput(props: TextareaInputProps) -> Element {
    // Standardized textarea with validation
}
```

#### FormSection Component
```rust
#[derive(Props, PartialEq, Clone)]
pub struct FormSectionProps {
    pub title: String,
    pub description: Option<String>,
    pub children: Element,
}

#[component]
pub fn FormSection(props: FormSectionProps) -> Element {
    // Wrapper for form sections with consistent styling
}
```

### 2. Data Display Components

#### DataTable Component
```rust
#[derive(Props, PartialEq, Clone)]
pub struct DataTableProps<T> {
    pub columns: Vec<Column<T>>,
    pub data: Signal<Vec<T>>,
    pub empty_message: String,
    pub loading: Signal<bool>,
}

#[derive(Props, PartialEq, Clone)]
pub struct Column<T> {
    pub header: String,
    pub render: impl Fn(&T) -> Element,
    pub width: Option<String>,
}

#[component]
pub fn DataTable<T: Clone + PartialEq + 'static>(props: DataTableProps<T>) -> Element {
    // Generic table component
}
```

#### LoadingState Component
```rust
#[derive(Props, PartialEq, Clone)]
pub struct LoadingStateProps {
    pub message: Option<String>,
    pub size: LoadingSize,
}

#[derive(PartialEq, Clone)]
pub enum LoadingSize {
    Small,
    Medium,
    Large,
}

#[component]
pub fn LoadingState(props: LoadingStateProps) -> Element {
    // Standardized loading component
}
```

#### EmptyState Component
```rust
#[derive(Props, PartialEq, Clone)]
pub struct EmptyStateProps {
    pub icon: Option<String>,
    pub title: String,
    pub description: String,
    pub action: Option<EmptyStateAction>,
}

#[derive(Props, PartialEq, Clone)]
pub struct EmptyStateAction {
    pub label: String,
    pub on_click: EventHandler<()>,
}

#[component]
pub fn EmptyState(props: EmptyStateProps) -> Element {
    // Standardized empty state component
}
```

### 3. Feedback Components

#### ErrorDisplay Component
```rust
#[derive(Props, PartialEq, Clone)]
pub struct ErrorDisplayProps {
    pub error: Signal<Option<String>>,
    pub on_retry: Option<EventHandler<()>>,
    pub title: Option<String>,
}

#[component]
pub fn ErrorDisplay(props: ErrorDisplayProps) -> Element {
    // Standardized error display
}
```

#### ValidationMessage Component
```rust
#[derive(Props, PartialEq, Clone)]
pub struct ValidationMessageProps {
    pub error: Signal<String>,
    pub field_name: Option<String>,
}

#[component]
pub fn ValidationMessage(props: ValidationMessageProps) -> Element {
    // Standardized validation message
}
```

### 4. Layout Components

#### PageHeader Component
```rust
#[derive(Props, PartialEq, Clone)]
pub struct PageHeaderProps {
    pub title: String,
    pub description: Option<String>,
    pub actions: Option<Element>,
    pub breadcrumb: Option<Element>,
}

#[component]
pub fn PageHeader(props: PageHeaderProps) -> Element {
    // Standardized page header
}
```

#### SectionCard Component
```rust
#[derive(Props, PartialEq, Clone)]
pub struct SectionCardProps {
    pub title: Option<String>,
    pub children: Element,
    pub actions: Option<Element>,
}

#[component]
pub fn SectionCard(props: SectionCardProps) -> Element {
    // Standardized section card wrapper
}
```

## Implementation Examples

### Before (PromptEditPage)
```rust
div { class: "space-y-6",
    div {
        label { class: "block text-sm font-medium text-zinc-900 dark:text-white mb-2",
            "Prompt ID"
        }
        input {
            r#type: "text",
            value: "{prompt_id()}",
            oninput: move |evt| prompt_id.set(evt.value()),
            placeholder: "e.g., generate-rust-code",
            class: "block w-full rounded-md border-0 py-1.5 text-zinc-900 dark:text-white shadow-sm ring-1 ring-inset ring-zinc-300 dark:ring-zinc-600 placeholder:text-zinc-400 dark:placeholder:text-zinc-500 focus:ring-2 focus:ring-inset focus:ring-blue-600 dark:bg-zinc-700 sm:text-sm sm:leading-6"
        }
        p { class: "mt-1 text-xs text-zinc-500 dark:text-zinc-400",
            "Human-readable identifier used to reference this prompt in workflows"
        }
    }
    // ... more form fields
}
```

### After (PromptEditPage)
```rust
div { class: "space-y-6",
    TextInput {
        label: "Prompt ID".to_string(),
        value: prompt_id,
        on_change: move |value| prompt_id.set(value),
        placeholder: Some("e.g., generate-rust-code".to_string()),
        validation_error: Some(validation_error),
        required: true,
        disabled: false,
    }
    // ... more form fields using components
}
```

## Benefits

1. **Consistency**: Uniform UI across all pages
2. **Maintainability**: Changes to components affect all usage
3. **Developer Experience**: Faster development with reusable components
4. **Accessibility**: Centralized accessibility improvements
5. **Design System**: Foundation for a proper design system

## Migration Strategy

1. **Create component files** - Implement common components
2. **Update one page at a time** - Start with simplest forms
3. **Test thoroughly** - Ensure visual consistency
4. **Refactor remaining pages** - Apply components to all pages
5. **Document usage** - Create component documentation

## Success Metrics

- Reduce form code duplication by 80%
- Standardize UI patterns across all pages
- Improve accessibility scores
- Faster development of new forms
- Consistent user experience
