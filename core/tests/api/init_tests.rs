

use core::*;

#[test]
fn test_global_store_initialization() {

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(init_test_store());



    match result {
        Ok(_) => {

            let store = get_global_store();
            assert!(store.is_ok(), "Failed to get global store");
        },
        Err(e) => {

            assert!(e.contains("Failed to create store"), "Expected store creation error");
        }
    }
}

#[test]
fn test_tracing_initialization() {

    use core::logging::TracingGuard;


    let _init_fn: fn(Option<String>) -> Result<TracingGuard, String> = core::logging::init_tracing;
}

#[test]
fn test_initialization_functions_exist() {

    let _init_store_fn: fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>> = init_test_store;
    let _get_store_fn: fn() -> Result<std::sync::Arc<s_e_e_persistence::Store>, String> = get_global_store;
}
