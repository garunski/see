use s_e_e_dioxus_query::cache::{get_typed_value, CacheEntry, TypedCacheEntry};
use std::sync::Arc;
use std::time::Duration;

#[test]
fn test_typed_cache_entry_creation() {
    let value = Arc::new(vec![1, 2, 3]);
    let entry = TypedCacheEntry::new(value.clone());

    assert_eq!(entry.value(), &value);
    assert!(!entry.is_fetching());
    assert_eq!(entry.cache_time(), Some(300_000)); // Default 5 minutes
}

#[test]
fn test_typed_cache_entry_with_custom_cache_time() {
    let value = Arc::new(42);
    let cache_time = Some(60_000); // 1 minute
    let entry = TypedCacheEntry::with_cache_time(value.clone(), cache_time);

    assert_eq!(entry.cache_time(), cache_time);
    assert_eq!(entry.value(), &value);
}

#[test]
fn test_placeholder_entry() {
    let default_value = Arc::new(String::from("placeholder"));
    let entry = TypedCacheEntry::placeholder(default_value.clone());

    assert!(entry.is_fetching());
    assert_eq!(entry.value(), &default_value);
}

#[test]
fn test_touch_updates_last_accessed() {
    let value = Arc::new(100);
    let mut entry = TypedCacheEntry::new(value);

    let first_accessed = entry.last_accessed();
    std::thread::sleep(Duration::from_millis(10));

    entry.touch();
    let second_accessed = entry.last_accessed();

    assert!(second_accessed > first_accessed);
}

#[test]
fn test_set_fetching() {
    let value = Arc::new("test");
    let mut entry = TypedCacheEntry::new(value);

    assert!(!entry.is_fetching());

    entry.set_fetching(true);
    assert!(entry.is_fetching());

    entry.set_fetching(false);
    assert!(!entry.is_fetching());
}

#[test]
fn test_get_typed_value_success() {
    let value = Arc::new(String::from("Hello, World!"));
    let entry = TypedCacheEntry::new(value.clone());
    let entry_trait: &dyn CacheEntry = &entry;

    let retrieved: Option<Arc<String>> = get_typed_value(entry_trait);
    assert!(retrieved.is_some());
    assert_eq!(*retrieved.unwrap(), *value);
}

#[test]
fn test_get_typed_value_wrong_type() {
    let value = Arc::new(42i32);
    let entry = TypedCacheEntry::new(value);
    let entry_trait: &dyn CacheEntry = &entry;

    // Try to get as String, should fail
    let retrieved: Option<Arc<String>> = get_typed_value(entry_trait);
    assert!(retrieved.is_none());
}

#[test]
fn test_type_id_matches() {
    use std::any::TypeId;

    let value = Arc::new(vec![1, 2, 3]);
    let entry = TypedCacheEntry::new(value);

    assert_eq!(entry.type_id(), TypeId::of::<Vec<i32>>());
}

#[test]
fn test_fetched_at() {
    let value = Arc::new(123);
    let entry = TypedCacheEntry::new(value);

    let fetched_at = entry.fetched_at();
    assert!(fetched_at.is_some());

    // Should be very recent
    let elapsed = fetched_at.unwrap().elapsed();
    assert!(elapsed < Duration::from_secs(1));
}
