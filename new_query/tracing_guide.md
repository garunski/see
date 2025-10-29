# Dioxus Query - Tracing Configuration Guide

## 📦 Dependencies

Add these to your `Cargo.toml`:

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
tracing-appender = "0.2"
```

---

## 🎯 Log Levels & What They Show

### TRACE (Most Verbose)
- Cache key creation
- Individual cache lookups
- Query state transitions
- Every retry attempt
- Callback execution

**Use when:** Debugging complex caching issues or race conditions

### DEBUG
- Cache hits/misses
- Data freshness checks
- State updates
- Query invalidation
- Mutation lifecycle

**Use when:** Understanding query behavior and cache performance

### INFO (Default)
- Query initialization
- Fetch operations (start/success)
- Mutation triggers
- User actions (button clicks, ID changes)
- Cache statistics

**Use when:** General application monitoring

### WARN
- Failed fetch attempts (before final failure)
- Deserialization issues
- Invalid inputs

**Use when:** Catching potential problems

### ERROR
- Final fetch failures
- Mutation errors
- Critical issues

**Use when:** Production error tracking

---

## 🛠️ Configuration Modes

### Development Mode (Console Only)
```rust
use tracing_setup::init_tracing_dev;

fn main() {
    init_tracing_dev(); // TRACE level, pretty console output
    dioxus::launch(App);
}
```

**Output:**
```
2024-10-29T10:30:45.123Z TRACE query: Creating new QueryKey key="user:1"
2024-10-29T10:30:45.125Z DEBUG query: Cache miss key="user:1"
2024-10-29T10:30:45.126Z INFO  query: Starting fetch operation key="user:1"
```

### Production Mode (File Only)
```rust
use tracing_setup::init_tracing_prod;

fn main() {
    init_tracing_prod(); // INFO level, JSON logs to ./logs/prod.log
    dioxus::launch(App);
}
```

**Output** (in `./logs/prod.log`):
```json
{
  "timestamp": "2024-10-29T10:30:45.123Z",
  "level": "INFO",
  "target": "query",
  "fields": {
    "message": "Starting fetch operation",
    "key": "user:1"
  },
  "span": {
    "name": "use_query",
    "key": "user:1"
  }
}
```

### Balanced Mode (Console + File)
```rust
use tracing_setup::init_tracing;

fn main() {
    init_tracing(); // INFO console, DEBUG file
    dioxus::launch(App);
}
```

### Custom Level
```rust
use tracing::Level;
use tracing_setup::init_tracing_with_level;

fn main() {
    init_tracing_with_level(Level::DEBUG);
    dioxus::launch(App);
}
```

---

## 🔍 Environment Variable Control

Set the `RUST_LOG` environment variable to override defaults:

```bash
# Show only errors
RUST_LOG=error cargo run

# Show everything from query module
RUST_LOG=query=trace cargo run

# Mixed levels
RUST_LOG=info,query=debug,tokio=warn cargo run

# Show specific spans
RUST_LOG=query::use_query=trace cargo run
```

---

## 📊 Example Log Output

### Successful Query Fetch
```
INFO  query: Initializing query key="user:1" stale_time=30000ms retry=2 enabled=true
DEBUG query: Cache miss key="user:1"
INFO  query: Starting fetch operation key="user:1"
DEBUG query: Fetch attempt key="user:1" attempt=1 max_attempts=3
INFO  app: Fetching user data url="https://jsonplaceholder.typicode.com/users/1"
INFO  query: Fetch successful key="user:1" attempt=1
TRACE query: Updated cache with fresh data key="user:1"
DEBUG query: Query state updated: success key="user:1"
```

### Failed Query with Retry
```
INFO  query: Starting fetch operation key="user:999"
DEBUG query: Fetch attempt key="user:999" attempt=1 max_attempts=3
ERROR app: HTTP request failed url="https://..." error="404 Not Found"
WARN  query: Fetch failed - will retry key="user:999" attempt=1 max_attempts=3 error="404 Not Found"
DEBUG query: Waiting before retry key="user:999" delay_ms=1000
DEBUG query: Fetch attempt key="user:999" attempt=2 max_attempts=3
ERROR app: HTTP request failed url="https://..." error="404 Not Found"
WARN  query: Fetch failed - will retry key="user:999" attempt=2 max_attempts=3 error="404 Not Found"
DEBUG query: Fetch attempt key="user:999" attempt=3 max_attempts=3
ERROR query: Fetch failed after all retry attempts key="user:999" attempt=3 error="404 Not Found"
```

### Mutation with Callbacks
```
INFO  query: Initializing mutation invalidate_keys_count=1 has_optimistic=true
INFO  query: Mutation triggered
DEBUG query: Mutation state: is_loading = true
INFO  query: Applying optimistic update key="user:1"
DEBUG query: Query data set in cache key="user:1"
INFO  app: Create mutation called name="New User" email="new@example.com"
INFO  query: Mutation successful
DEBUG query: Mutation state updated: success
DEBUG query: Invalidating queries after successful mutation count=1
INFO  query: Invalidating query key="user:1"
TRACE query: Executing on_success callback
INFO  app: on_success: User created successfully user_id=999 user_name="New User"
```

### Cache Invalidation
```
INFO  query: Invalidating queries by prefix prefix="user"
DEBUG query: Queries invalidated by prefix prefix="user" removed_count=3
```

---

## 🎨 Customizing Log Output

### Add Custom Fields to Your Fetchers
```rust
let (user_query, _) = use_query(
    user_key.clone(),
    move || {
        info!(
            user_id = user_id(),
            source = "api",
            "Fetching user from API"
        );
        async move {
            // ... fetch logic
        }
    },
    options,
);
```

### Instrument Your Own Functions
```rust
use tracing::instrument;

#[instrument(skip(data), fields(size = data.len()))]
async fn process_data(data: Vec<u8>) -> Result<String, String> {
    debug!("Processing data");
    // ... your logic
    info!("Data processed successfully");
    Ok(result)
}
```

---

## 📁 Log File Management

Logs are automatically rotated daily in `./logs/` directory:

```
./logs/
├── dioxus_query.log.2024-10-29
├── dioxus_query.log.2024-10-30
└── dioxus_query.log (current)
```

### Custom Log Directory
```rust
use tracing_appender::rolling;

let file_appender = rolling::daily("/var/log/myapp", "queries.log");
```

---

## 🐛 Debugging Tips

### 1. Track a Specific Query
```bash
RUST_LOG=query::use_query=trace cargo run 2>&1 | grep "user:1"
```

### 2. Monitor Cache Behavior
Click the "Log Cache Stats" button in the UI or add periodic logging:
```rust
use_future(|| async {
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
        let (size, keys) = get_cache_stats();
        info!(cache_size = size, "Periodic cache check");
    }
});
```

### 3. Filter JSON Logs
```bash
cat logs/dioxus_query.log | jq 'select(.level == "ERROR")'
cat logs/dioxus_query.log | jq 'select(.fields.key == "user:1")'
```

---

## 🚀 Performance Impact

- **TRACE**: ~5-10% overhead (dev only)
- **DEBUG**: ~2-3% overhead
- **INFO**: ~0.5-1% overhead
- **File logging**: Async, minimal impact (<0.5%)

**Recommendation:** Use INFO in production, DEBUG for staging, TRACE only for debugging specific issues.

---

## 📝 Best Practices

1. **Always initialize tracing before launching Dioxus**
   ```rust
   init_tracing();
   dioxus::launch(App);
   ```

2. **Use structured fields instead of string formatting**
   ```rust
   // Good
   info!(user_id = id, "User fetched");
   
   // Avoid
   info!("User fetched: {}", id);
   ```

3. **Keep log messages actionable**
   - Include relevant context (keys, IDs, counts)
   - Make errors searchable
   - Add timing information for slow operations

4. **Monitor log file size in production**
   - Daily rotation prevents unbounded growth
   - Consider log aggregation services (e.g., Loki, ELK)

5. **Use appropriate log levels**
   - Don't log sensitive data at INFO/DEBUG in production
   - Use WARN for things that need attention
   - Reserve ERROR for actual failures