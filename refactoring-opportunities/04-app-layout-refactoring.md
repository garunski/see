# App Layout Refactoring

## Current State
- **File**: `gui/src/layout/app.rs`
- **Lines**: 160 lines
- **Priority**: 📱 MEDIUM - Will simplify main application structure

## Problems

### 1. Mixed Responsibilities
The `app.rs` file handles too many concerns:
- Main app initialization
- Settings loading logic
- Error boundary setup
- Suspense boundary management
- Window configuration
- State provider setup

### 2. Complex Settings Loading
```rust
// Complex settings loading logic mixed with UI
let settings_loader = use_resource(|| async move {
    match see_core::get_global_store() {
        Ok(store) => match store.load_settings().await {
            Ok(Some(loaded)) => Ok(loaded),
            Ok(None) => Ok(AppSettings::default()),
            Err(e) => Err(format!("Failed to load settings: {}", e)),
        },
        Err(e) => {
            eprintln!("Failed to get global store for settings: {}", e);
            Ok(AppSettings::default())
        }
    }
});

use_effect(move || {
    if let Some(Ok(settings)) = settings_loader.read().as_ref() {
        state_provider
            .settings
            .write()
            .apply_loaded_settings(settings.clone());
        // ... more complex logic
    }
});
```

### 3. Error Boundary Complexity
Error boundary setup is verbose and could be extracted:

```rust
ErrorBoundary {
    handle_error: |error: ErrorContext| rsx! {
        div { class: "flex items-center justify-center min-h-screen",
            div { class: "text-center p-8",
                h1 { class: "text-2xl font-bold text-red-600 dark:text-red-400 mb-4", "Application Error" }
                p { class: "text-zinc-600 dark:text-zinc-400 mb-4", "An error occurred while initializing the application." }
                pre { class: "text-sm text-zinc-500 dark:text-zinc-500 bg-zinc-100 dark:bg-zinc-800 p-4 rounded", "{error:#?}" }
            }
        }
    },
    // ... more complex error handling
}
```

### 4. Suspense Boundary Duplication
Similar loading patterns could be standardized.

## Refactoring Plan

### Create Focused Layout Components
```
layout/
├── app.rs                    // Main app (30 lines)
├── app_content.rs            // Content wrapper
├── settings_loader.rs        // Settings loading logic
├── error_boundary.rs         // Error handling
├── loading_screen.rs         // Loading states
└── window_config.rs          // Window configuration
```

### 1. `app.rs` - Main App (Simplified)
```rust
#[component]
pub fn App() -> Element {
    let window = use_window();
    use_effect(move || {
        window.set_always_on_top(false);
        window.set_focus();
    });

    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }
        div {
            class: "min-h-screen bg-white dark:bg-zinc-900 text-zinc-950 dark:text-white",
            AppErrorBoundary {
                AppSuspenseBoundary {
                    AppContent {}
                }
            }
        }
    }
}
```

### 2. `app_content.rs` - Content Wrapper
```rust
#[component]
pub fn AppContent() -> Element {
    let state_provider = use_hook(AppStateProvider::new);
    use_context_provider(|| state_provider.clone());

    let settings_loader = use_settings_loader();
    use_settings_effect(settings_loader, state_provider.clone());

    rsx! {
        Router::<Route> {}
    }
}
```

### 3. `settings_loader.rs` - Settings Loading Logic
```rust
pub fn use_settings_loader() -> Resource<Result<AppSettings, String>> {
    use_resource(|| async move {
        match see_core::get_global_store() {
            Ok(store) => match store.load_settings().await {
                Ok(Some(loaded)) => Ok(loaded),
                Ok(None) => Ok(AppSettings::default()),
                Err(e) => Err(format!("Failed to load settings: {}", e)),
            },
            Err(e) => {
                eprintln!("Failed to get global store for settings: {}", e);
                Ok(AppSettings::default())
            }
        }
    })
}

pub fn use_settings_effect(
    settings_loader: Resource<Result<AppSettings, String>>,
    state_provider: AppStateProvider,
) {
    use_effect(move || {
        if let Some(Ok(settings)) = settings_loader.read().as_ref() {
            state_provider
                .settings
                .write()
                .apply_loaded_settings(settings.clone());

            // Load default workflows
            let default_workflows = see_core::WorkflowDefinition::get_default_workflows();
            let mut settings_guard = state_provider.settings.write();
            for default_workflow in default_workflows {
                let exists = settings_guard
                    .settings
                    .workflows
                    .iter()
                    .any(|w| w.id == default_workflow.id);
                if !exists {
                    settings_guard.add_workflow(default_workflow);
                }
            }

            // Save settings
            let settings_to_save = settings_guard.settings.clone();
            spawn(async move {
                if let Ok(store) = see_core::get_global_store() {
                    let _ = store.save_settings(&settings_to_save).await;
                }
            });
        }
    });
}
```

### 4. `error_boundary.rs` - Error Handling
```rust
#[component]
pub fn AppErrorBoundary(children: Element) -> Element {
    rsx! {
        ErrorBoundary {
            handle_error: |error: ErrorContext| rsx! {
                ErrorDisplay {
                    error: Signal::new(Some(format!("{:#?}", error))),
                    title: Some("Application Error".to_string()),
                }
            },
            {children}
        }
    }
}

#[component]
pub fn ErrorDisplay(
    error: Signal<Option<String>>,
    title: Option<String>,
) -> Element {
    rsx! {
        div { class: "flex items-center justify-center min-h-screen",
            div { class: "text-center p-8",
                h1 { class: "text-2xl font-bold text-red-600 dark:text-red-400 mb-4",
                    {title.unwrap_or_else(|| "Error".to_string())}
                }
                p { class: "text-zinc-600 dark:text-zinc-400 mb-4",
                    "An error occurred while running the application."
                }
                if let Some(error_msg) = error() {
                    pre { class: "text-sm text-zinc-500 dark:text-zinc-500 bg-zinc-100 dark:bg-zinc-800 p-4 rounded",
                        "{error_msg}"
                    }
                }
            }
        }
    }
}
```

### 5. `loading_screen.rs` - Loading States
```rust
#[component]
pub fn AppSuspenseBoundary(children: Element) -> Element {
    rsx! {
        SuspenseBoundary {
            fallback: move |_| rsx! {
                AppLoadingScreen {}
            },
            {children}
        }
    }
}

#[component]
pub fn AppLoadingScreen() -> Element {
    rsx! {
        div { class: "flex items-center justify-center min-h-screen",
            div { class: "text-center",
                div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4" }
                p { class: "text-zinc-600 dark:text-zinc-400", "Loading application..." }
            }
        }
    }
}
```

### 6. `window_config.rs` - Window Configuration
```rust
pub fn use_window_config() {
    let window = use_window();
    use_effect(move || {
        window.set_always_on_top(false);
        window.set_focus();
    });
}
```

## Implementation Examples

### Before (Current app.rs)
```rust
#[component]
pub fn App() -> Element {
    let window = use_window();
    use_effect(move || {
        window.set_always_on_top(false);
        window.set_focus();
    });

    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }
        div {
            class: "min-h-screen bg-white dark:bg-zinc-900 text-zinc-950 dark:text-white",
            ErrorBoundary {
                handle_error: |error: ErrorContext| rsx! {
                    // ... complex error handling
                },
                SuspenseBoundary {
                    fallback: move |_| rsx! {
                        // ... complex loading screen
                    },
                    AppContent {}
                }
            }
        }
    }
}

#[component]
fn AppContent() -> Element {
    // ... 100+ lines of complex settings loading logic
}
```

### After (Refactored)
```rust
#[component]
pub fn App() -> Element {
    use_window_config();

    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }
        div {
            class: "min-h-screen bg-white dark:bg-zinc-900 text-zinc-950 dark:text-white",
            AppErrorBoundary {
                AppSuspenseBoundary {
                    AppContent {}
                }
            }
        }
    }
}

#[component]
pub fn AppContent() -> Element {
    let state_provider = use_hook(AppStateProvider::new);
    use_context_provider(|| state_provider.clone());

    let settings_loader = use_settings_loader();
    use_settings_effect(settings_loader, state_provider.clone());

    rsx! {
        Router::<Route> {}
    }
}
```

## Benefits

1. **Single Responsibility**: Each component has one clear purpose
2. **Testability**: Smaller components are easier to test
3. **Reusability**: Error boundaries and loading screens can be reused
4. **Maintainability**: Settings loading logic is isolated
5. **Readability**: Main app structure is clear and simple

## Migration Strategy

1. **Create new files** - Implement focused components
2. **Extract settings logic** - Move to dedicated module
3. **Extract error handling** - Create reusable error components
4. **Update main app** - Use extracted components
5. **Test thoroughly** - Ensure all functionality works

## Success Metrics

- Main app component < 50 lines
- Settings loading logic isolated and testable
- Reusable error and loading components
- Clear separation of concerns
- Improved maintainability
