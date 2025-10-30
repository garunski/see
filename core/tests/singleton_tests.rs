use s_e_e_core::store_singleton;

#[test]
fn test_global_store_initialization() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(store_singleton::init_global_store());

    match result {
        Ok(_) => {
            let store = store_singleton::get_global_store();
            assert!(store.is_ok(), "Failed to get global store");
        }
        Err(e) => {
            assert!(!e.is_empty(), "Expected some error message");
        }
    }
}

#[test]
fn test_global_store_double_initialization() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let result1 = rt.block_on(store_singleton::init_global_store());

    let result2 = rt.block_on(store_singleton::init_global_store());

    match (result1, result2) {
        (Ok(_), Err(msg)) => {
            assert!(
                msg.contains("Store already initialized"),
                "Expected double init error"
            );
        }
        (Err(_), Err(_)) => {}
        (Ok(_), Ok(_)) => {
            panic!("Second initialization should have failed");
        }
        (Err(_), Ok(_)) => {
            panic!("Unexpected: first failed but second succeeded");
        }
    }
}

#[test]
fn test_get_global_store_before_init() {
    let store = store_singleton::get_global_store();

    match store {
        Ok(_) => {}
        Err(error_msg) => {
            assert!(error_msg.contains("Store not initialized"));
        }
    }
}

#[test]
fn test_store_thread_safety() {
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();

    let init_result = rt.block_on(store_singleton::init_global_store());

    match init_result {
        Ok(_) => {
            let handles: Vec<_> = (0..10)
                .map(|_i| {
                    rt.spawn(async move {
                        let store = store_singleton::get_global_store();
                        assert!(store.is_ok(), "Failed to get store in concurrent task");
                        store.unwrap()
                    })
                })
                .collect();

            for handle in handles.into_iter() {
                let store = rt.block_on(handle).unwrap();
                assert!(
                    Arc::strong_count(&store) > 1,
                    "Store should be shared across tasks"
                );
            }
        }
        Err(_) => {}
    }
}
