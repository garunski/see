# Service Layer Standardization

## Current State
- **Files**: `gui/src/services/` directory
- **Priority**: 🔧 MEDIUM - Will improve API consistency and error handling

## Problems

### 1. Inconsistent Error Handling
Different services handle errors differently:

```rust
// In prompt service
match PromptService::fetch_prompts().await {
    Ok(loaded_prompts) => {
        state_provider.prompts.write().load_prompts(loaded_prompts);
    }
    Err(e) => {
        tracing::error!("Failed to load prompts: {}", e);
    }
}

// In history service
match HistoryService::fetch_history().await {
    Ok(history) => {
        state_provider.history.write().load_history(history);
    }
    Err(e) => {
        error_signal.set(format!("Failed to load history: {}", e));
    }
}
```

### 2. Duplicate Async Patterns
Similar async patterns repeated across services:

```rust
// Repeated pattern in multiple services
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
```

### 3. Mixed Concerns
Services mix business logic with state management:

```rust
// Service handling both API calls and state updates
pub async fn fetch_prompts() -> Result<Vec<Prompt>, String> {
    // API call logic
    // State update logic mixed in
}
```

### 4. Inconsistent API Patterns
Different services have different method signatures and return types.

## Refactoring Plan

### Create `gui/src/services/common/` Directory
```
services/
├── common/
│   ├── mod.rs
│   ├── api_client.rs          // Centralized API client
│   ├── error_types.rs         // Standardized error types
│   ├── async_helpers.rs       // Common async patterns
│   └── service_traits.rs      // Service trait definitions
├── database.rs
├── history.rs
├── prompt.rs
└── workflow.rs
```

### 1. `error_types.rs` - Standardized Error Types
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceError {
    Network(String),
    Database(String),
    Validation(String),
    NotFound(String),
    Unauthorized(String),
    Internal(String),
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::Network(msg) => write!(f, "Network error: {}", msg),
            ServiceError::Database(msg) => write!(f, "Database error: {}", msg),
            ServiceError::Validation(msg) => write!(f, "Validation error: {}", msg),
            ServiceError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ServiceError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            ServiceError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for ServiceError {}

pub type ServiceResult<T> = Result<T, ServiceError>;
```

### 2. `api_client.rs` - Centralized API Client
```rust
pub struct ApiClient {
    base_url: String,
    timeout: Duration,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            timeout: Duration::from_secs(30),
        }
    }

    pub async fn get<T>(&self, endpoint: &str) -> ServiceResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        // Centralized GET request logic
    }

    pub async fn post<T, U>(&self, endpoint: &str, data: &T) -> ServiceResult<U>
    where
        T: serde::Serialize,
        U: serde::de::DeserializeOwned,
    {
        // Centralized POST request logic
    }

    pub async fn put<T, U>(&self, endpoint: &str, data: &T) -> ServiceResult<U>
    where
        T: serde::Serialize,
        U: serde::de::DeserializeOwned,
    {
        // Centralized PUT request logic
    }

    pub async fn delete(&self, endpoint: &str) -> ServiceResult<()> {
        // Centralized DELETE request logic
    }
}
```

### 3. `async_helpers.rs` - Common Async Patterns
```rust
pub async fn with_retry<F, Fut, T>(
    operation: F,
    max_retries: usize,
    delay: Duration,
) -> ServiceResult<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = ServiceResult<T>>,
{
    let mut last_error = None;
    
    for attempt in 0..max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries - 1 {
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
    
    Err(last_error.unwrap_or_else(|| ServiceError::Internal("Max retries exceeded".to_string())))
}

pub fn spawn_service_call<F, Fut, T>(
    operation: F,
    on_success: impl Fn(T) + 'static,
    on_error: impl Fn(ServiceError) + 'static,
) where
    F: Fn() -> Fut + 'static,
    Fut: Future<Output = ServiceResult<T>> + 'static,
    T: 'static,
{
    spawn(async move {
        match operation().await {
            Ok(result) => on_success(result),
            Err(error) => on_error(error),
        }
    });
}
```

### 4. `service_traits.rs` - Service Trait Definitions
```rust
pub trait Service<T> {
    async fn fetch_all() -> ServiceResult<Vec<T>>;
    async fn fetch_by_id(id: String) -> ServiceResult<Option<T>>;
    async fn create(item: T) -> ServiceResult<T>;
    async fn update(id: String, item: T) -> ServiceResult<T>;
    async fn delete(id: String) -> ServiceResult<()>;
}

pub trait CrudService<T> {
    async fn list() -> ServiceResult<Vec<T>>;
    async fn get(id: String) -> ServiceResult<Option<T>>;
    async fn create(item: T) -> ServiceResult<T>;
    async fn update(id: String, item: T) -> ServiceResult<T>;
    async fn delete(id: String) -> ServiceResult<()>;
}
```

### 5. Refactored Service Examples

#### Before (PromptService)
```rust
pub struct PromptService;

impl PromptService {
    pub async fn fetch_prompts() -> Result<Vec<Prompt>, String> {
        match see_core::get_global_store() {
            Ok(store) => match store.get_prompts().await {
                Ok(prompts) => Ok(prompts),
                Err(e) => Err(format!("Failed to fetch prompts: {}", e)),
            },
            Err(e) => Err(format!("Database not available: {}", e)),
        }
    }

    pub async fn create_prompt(prompt: Prompt) -> Result<(), String> {
        match see_core::get_global_store() {
            Ok(store) => match store.create_prompt(&prompt).await {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Failed to create prompt: {}", e)),
            },
            Err(e) => Err(format!("Database not available: {}", e)),
        }
    }
    // ... more methods with inconsistent error handling
}
```

#### After (PromptService)
```rust
pub struct PromptService {
    client: ApiClient,
}

impl PromptService {
    pub fn new() -> Self {
        Self {
            client: ApiClient::new(),
        }
    }

    pub async fn fetch_prompts() -> ServiceResult<Vec<Prompt>> {
        with_retry(
            || async {
                match see_core::get_global_store() {
                    Ok(store) => store.get_prompts().await
                        .map_err(|e| ServiceError::Database(e.to_string())),
                    Err(e) => Err(ServiceError::Database(e.to_string())),
                }
            },
            3,
            Duration::from_secs(1),
        ).await
    }

    pub async fn create_prompt(prompt: Prompt) -> ServiceResult<()> {
        match see_core::get_global_store() {
            Ok(store) => store.create_prompt(&prompt).await
                .map_err(|e| ServiceError::Database(e.to_string())),
            Err(e) => Err(ServiceError::Database(e.to_string())),
        }
    }
}

impl CrudService<Prompt> for PromptService {
    async fn list() -> ServiceResult<Vec<Prompt>> {
        Self::fetch_prompts().await
    }

    async fn get(id: String) -> ServiceResult<Option<Prompt>> {
        // Implementation
    }

    async fn create(item: Prompt) -> ServiceResult<Prompt> {
        Self::create_prompt(item.clone()).await?;
        Ok(item)
    }

    async fn update(id: String, item: Prompt) -> ServiceResult<Prompt> {
        // Implementation
    }

    async fn delete(id: String) -> ServiceResult<()> {
        // Implementation
    }
}
```

### 6. Service Integration Helpers
```rust
pub fn use_service_call<T, F, Fut>(
    operation: F,
    auto_execute: bool,
) -> (Signal<Option<T>>, Signal<bool>, Signal<Option<ServiceError>>, impl Fn())
where
    F: Fn() -> Fut + 'static,
    Fut: Future<Output = ServiceResult<T>> + 'static,
    T: Clone + 'static,
{
    let data = use_signal(|| None::<T>);
    let loading = use_signal(|| false);
    let error = use_signal(|| None::<ServiceError>);

    let execute = {
        let data = data;
        let loading = loading;
        let error = error;
        move || {
            loading.set(true);
            error.set(None);
            
            spawn_service_call(
                operation,
                move |result| {
                    data.set(Some(result));
                    loading.set(false);
                },
                move |err| {
                    error.set(Some(err));
                    loading.set(false);
                },
            );
        }
    };

    if auto_execute {
        use_effect(move || {
            execute();
        });
    }

    (data, loading, error, execute)
}
```

## Implementation Examples

### Before (Using Services)
```rust
// In component
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
```

### After (Using Services)
```rust
// In component
let (prompts, loading, error, refresh) = use_service_call(
    || PromptService::fetch_prompts(),
    true,
);

use_effect(move || {
    if let Some(prompts_data) = prompts() {
        state_provider.prompts.write().load_prompts(prompts_data);
    }
});
```

## Benefits

1. **Consistency**: Standardized error handling across all services
2. **Reliability**: Retry logic and better error recovery
3. **Maintainability**: Centralized API client and patterns
4. **Testability**: Services can be easily mocked and tested
5. **Developer Experience**: Consistent API patterns

## Migration Strategy

1. **Create common modules** - Implement error types and helpers
2. **Refactor one service at a time** - Start with simplest service
3. **Update components** - Use new service patterns
4. **Test thoroughly** - Ensure error handling works correctly
5. **Document patterns** - Create usage documentation

## Success Metrics

- Standardized error handling across all services
- Reduced code duplication by 70%
- Improved error recovery and user experience
- Consistent API patterns
- Better test coverage for services
