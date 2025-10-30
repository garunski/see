



use s_e_e_persistence::{AppSettings, Theme};

#[test]
fn test_app_settings_default() {
    let settings = AppSettings::default();

    assert_eq!(settings.theme, Theme::System);
    assert!(settings.auto_save);
    assert!(settings.notifications);
    assert!(settings.default_workflow.is_none());
}

#[test]
fn test_app_settings_validation() {
    let settings = AppSettings::default();


    let result = settings.validate();
    assert!(result.is_ok());
}

#[test]
fn test_app_settings_set_theme() {
    let mut settings = AppSettings::default();

    settings.set_theme(Theme::Dark);
    assert_eq!(settings.theme, Theme::Dark);

    settings.set_theme(Theme::Light);
    assert_eq!(settings.theme, Theme::Light);

    settings.set_theme(Theme::System);
    assert_eq!(settings.theme, Theme::System);
}

#[test]
fn test_app_settings_set_auto_save() {
    let mut settings = AppSettings::default();

    settings.set_auto_save(false);
    assert!(!settings.auto_save);

    settings.set_auto_save(true);
    assert!(settings.auto_save);
}

#[test]
fn test_app_settings_set_notifications() {
    let mut settings = AppSettings::default();

    settings.set_notifications(false);
    assert!(!settings.notifications);

    settings.set_notifications(true);
    assert!(settings.notifications);
}

#[test]
fn test_app_settings_set_default_workflow() {
    let mut settings = AppSettings::default();

    settings.set_default_workflow(Some("workflow-1".to_string()));
    assert_eq!(settings.default_workflow, Some("workflow-1".to_string()));

    settings.set_default_workflow(None);
    assert!(settings.default_workflow.is_none());
}

#[test]
fn test_app_settings_serialization() {
    let settings = AppSettings {
        theme: Theme::Dark,
        auto_save: false,
        notifications: true,
        default_workflow: Some("workflow-1".to_string()),
    };


    let json = serde_json::to_string(&settings).unwrap();
    assert!(json.contains("dark"));
    assert!(json.contains("auto_save"));
    assert!(json.contains("notifications"));
    assert!(json.contains("workflow-1"));


    let deserialized: AppSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.theme, settings.theme);
    assert_eq!(deserialized.auto_save, settings.auto_save);
    assert_eq!(deserialized.notifications, settings.notifications);
    assert_eq!(deserialized.default_workflow, settings.default_workflow);
}
