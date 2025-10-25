//! Database connection tests

use crate::tests::common::setup_test_db;

#[tokio::test]
async fn test_database_pool_creation() {
    let (_temp_dir, pool) = setup_test_db();
    assert_eq!(pool.pool_size(), 0);
    assert_eq!(pool.idle_connections(), 0);
}

#[tokio::test]
async fn test_connection_acquisition() {
    let (_temp_dir, pool) = setup_test_db();
    
    // Acquire connections
    let conn1 = pool.get_connection().unwrap();
    let conn2 = pool.get_connection().unwrap();
    
    // Should fail on third connection (pool size is 5, but we're testing with 2)
    // This test might need adjustment based on actual pool behavior
    drop(conn1);
    drop(conn2);
    
    // Should work again after dropping
    let conn3 = pool.get_connection().unwrap();
    assert!(conn3.is_ok());
}

#[tokio::test]
async fn test_concurrent_connections() {
    let (_temp_dir, pool) = std::sync::Arc::new(setup_test_db().1);
    
    let mut handles = vec![];
    
    for i in 0..10 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            let conn = pool_clone.get_connection().unwrap();
            // Simulate some work
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            i
        });
        handles.push(handle);
    }
    
    let results: Vec<_> = futures::future::join_all(handles).await;
    assert_eq!(results.len(), 10);
    
    // All should succeed
    for result in results {
        assert!(result.is_ok());
    }
}
