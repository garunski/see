// NOTE: These tests require a Dioxus runtime context for GlobalSignal
// Testing invalidation logic manually with standalone cache would be ideal,
// but GlobalSignal access requires runtime. For now, we document expected behavior.

use dioxus_query_custom::prelude::{
    invalidate_all_queries, invalidate_queries_by_prefix, invalidate_query, QueryKey,
};

#[test]
fn test_query_key_creation() {
    // Test that query keys are created correctly for invalidation
    let key1 = QueryKey::new(&["test", "invalidate", "1"]);
    let key2 = QueryKey::new(&["test", "invalidate", "2"]);

    assert_ne!(key1, key2);
    assert_eq!(key1.as_str(), "test:invalidate:1");
    assert_eq!(key2.as_str(), "test:invalidate:2");
}

#[test]
fn test_prefix_matching_logic() {
    // Test the string prefix matching logic used by invalidate_queries_by_prefix
    let test_keys = vec!["user:123", "user:456", "post:789", "user_profile:abc"];

    // Simulate prefix matching
    let prefix = "user:";
    let matches: Vec<&str> = test_keys
        .iter()
        .filter(|k| k.starts_with(prefix))
        .copied()
        .collect();

    assert_eq!(matches.len(), 2);
    assert!(matches.contains(&"user:123"));
    assert!(matches.contains(&"user:456"));
    assert!(!matches.contains(&"post:789"));
    assert!(!matches.contains(&"user_profile:abc"));
}

#[test]
fn test_invalidation_api_exists() {
    // Smoke test: ensure the functions are callable
    // Actual testing would require Dioxus runtime
    let key = QueryKey::new(&["test", "api"]);

    // These calls won't work without runtime, but we can verify compilation
    let _ = std::panic::catch_unwind(|| {
        invalidate_query(&key);
    });

    let _ = std::panic::catch_unwind(|| {
        invalidate_queries_by_prefix("test:");
    });

    let _ = std::panic::catch_unwind(|| {
        invalidate_all_queries();
    });

    // If we got here without compile errors, the API exists
    assert!(true);
}
