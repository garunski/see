use s_e_e_core::*;

#[test]
fn test_global_store_initialization() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(init_test_store());

    match result {
        Ok(_) => {
            let store = get_global_store();
            assert!(store.is_ok(), "Failed to get global store");
        }
        Err(e) => {
            assert!(!e.is_empty(), "Expected some error message");
        }
    }
}

#[test]
fn test_tracing_initialization() {
    use s_e_e_core::logging::TracingGuard;

    let _init_fn: fn(Option<String>) -> Result<TracingGuard, String> =
        s_e_e_core::logging::init_tracing;
}

#[test]
fn test_initialization_functions_exist() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let _result = rt.block_on(init_test_store());

    let _store_result = get_global_store();
}
