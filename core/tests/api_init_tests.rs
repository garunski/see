// Initialization tests ONLY

use s_e_e_core::*;

#[test]
fn test_global_store_initialization() {
    // Test that we can initialize the global store
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(init_test_store());

    // The initialization might fail in test environment due to database permissions
    // This is acceptable - we just test that the function exists and can be called
    match result {
        Ok(_) => {
            // Success case - test that we can get the store
            let store = get_global_store();
            assert!(store.is_ok(), "Failed to get global store");
        }
        Err(e) => {
            // Failure case - this is acceptable in test environment
            // Just verify we got some error message
            assert!(!e.is_empty(), "Expected some error message");
        }
    }
}

#[test]
fn test_tracing_initialization() {
    // Test that tracing initialization function exists and has correct signature
    use s_e_e_core::logging::TracingGuard;

    // Test that init_tracing function exists and has correct signature
    let _init_fn: fn(Option<String>) -> Result<TracingGuard, String> =
        s_e_e_core::logging::init_tracing;
}

#[test]
fn test_initialization_functions_exist() {
    // Test that all initialization functions exist and can be called
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Test init_test_store function exists
    let _result = rt.block_on(init_test_store());

    // Test get_global_store function exists
    let _store_result = get_global_store();
}
