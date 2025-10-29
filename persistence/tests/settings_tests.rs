//! Tests for settings store operations
//!
//! Tests load_settings, save_settings following Single Responsibility Principle.

use s_e_e_persistence::{AppSettings, Store, Theme};

async fn create_test_store() -> Store {
    Store::new(":memory:").await.unwrap()
}

#[tokio::test]
async fn test_load_settings_default() {
    let store = create_test_store().await;

    let settings = store.load_settings().await.unwrap();

    // Should return None when no settings are saved
    assert!(settings.is_none());
}

#[tokio::test]
async fn test_save_settings() {
    let store = create_test_store().await;

    let settings = AppSettings {
        theme: Theme::Dark,
        auto_save: false,
        notifications: true,
        default_workflow: Some("workflow-1".to_string()),
    };

    let result = store.save_settings(&settings).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_load_settings_after_save() {
    let store = create_test_store().await;

    let original_settings = AppSettings {
        theme: Theme::Light,
        auto_save: false,
        notifications: false,
        default_workflow: Some("workflow-2".to_string()),
    };

    // Save settings
    store.save_settings(&original_settings).await.unwrap();

    // Load settings
    let loaded_settings = store.load_settings().await.unwrap().unwrap();

    assert_eq!(loaded_settings.theme, Theme::Light);
    assert!(!loaded_settings.auto_save);
    assert!(!loaded_settings.notifications);
    assert_eq!(
        loaded_settings.default_workflow,
        Some("workflow-2".to_string())
    );
}

#[tokio::test]
async fn test_save_settings_update() {
    let store = create_test_store().await;

    // Save initial settings
    let initial_settings = AppSettings {
        theme: Theme::System,
        auto_save: true,
        notifications: true,
        default_workflow: None,
    };

    store.save_settings(&initial_settings).await.unwrap();

    // Update settings
    let updated_settings = AppSettings {
        theme: Theme::Dark,
        auto_save: false,
        notifications: false,
        default_workflow: Some("workflow-3".to_string()),
    };

    store.save_settings(&updated_settings).await.unwrap();

    // Verify update
    let loaded_settings = store.load_settings().await.unwrap().unwrap();
    assert_eq!(loaded_settings.theme, Theme::Dark);
    assert!(!loaded_settings.auto_save);
    assert!(!loaded_settings.notifications);
    assert_eq!(
        loaded_settings.default_workflow,
        Some("workflow-3".to_string())
    );
}

#[tokio::test]
async fn test_settings_serialization() {
    let store = create_test_store().await;

    let settings = AppSettings {
        theme: Theme::Dark,
        auto_save: false,
        notifications: true,
        default_workflow: Some("workflow-1".to_string()),
    };

    // Save settings
    store.save_settings(&settings).await.unwrap();

    // Load and verify
    let loaded_settings = store.load_settings().await.unwrap().unwrap();
    assert_eq!(loaded_settings.theme, settings.theme);
    assert_eq!(loaded_settings.auto_save, settings.auto_save);
    assert_eq!(loaded_settings.notifications, settings.notifications);
    assert_eq!(loaded_settings.default_workflow, settings.default_workflow);
}

#[tokio::test]
async fn test_settings_all_themes() {
    let store = create_test_store().await;

    let themes = vec![Theme::Light, Theme::Dark, Theme::System];

    for theme in themes {
        let settings = AppSettings {
            theme: theme.clone(),
            auto_save: true,
            notifications: true,
            default_workflow: None,
        };

        // Save settings
        store.save_settings(&settings).await.unwrap();

        // Load and verify
        let loaded_settings = store.load_settings().await.unwrap().unwrap();
        assert_eq!(loaded_settings.theme, theme);
    }
}
