use dioxus_query_custom::prelude::QueryKey;

#[test]
fn test_query_key_new() {
    let key = QueryKey::new(&["user", "123"]);
    assert_eq!(key.as_str(), "user:123");
}

#[test]
fn test_query_key_from_string() {
    let key = QueryKey::from_string("custom:key:path".to_string());
    assert_eq!(key.as_str(), "custom:key:path");
}

#[test]
fn test_query_key_display() {
    let key = QueryKey::new(&["prompts", "list"]);
    assert_eq!(format!("{}", key), "prompts:list");
}

#[test]
fn test_query_key_equality() {
    let key1 = QueryKey::new(&["test", "1"]);
    let key2 = QueryKey::new(&["test", "1"]);
    let key3 = QueryKey::new(&["test", "2"]);

    assert_eq!(key1, key2);
    assert_ne!(key1, key3);
}

#[test]
fn test_query_key_clone() {
    let key1 = QueryKey::new(&["original"]);
    let key2 = key1.clone();

    assert_eq!(key1, key2);
}

#[test]
fn test_query_key_hash() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    let key = QueryKey::new(&["hash", "test"]);

    map.insert(key.clone(), "value");

    assert_eq!(map.get(&key), Some(&"value"));
}

#[test]
fn test_query_key_empty_parts() {
    let key = QueryKey::new(&[]);
    assert_eq!(key.as_str(), "");
}

#[test]
fn test_query_key_single_part() {
    let key = QueryKey::new(&["single"]);
    assert_eq!(key.as_str(), "single");
}

#[test]
fn test_query_key_many_parts() {
    let key = QueryKey::new(&["a", "b", "c", "d", "e"]);
    assert_eq!(key.as_str(), "a:b:c:d:e");
}
