use katana_platform::SettingsService;

#[test]
fn new_settings_service_has_defaults() {
    let svc = SettingsService::default();
    let settings = svc.settings();
    assert!(settings.last_workspace.is_none());
    assert_eq!(settings.theme, "dark");
    assert!(settings.extra.is_empty());
}

#[test]
fn settings_returns_immutable_reference() {
    let svc = SettingsService::default();
    let settings = svc.settings();
    // Compile-time guarantee that settings cannot be modified because it is an immutable reference
    assert_eq!(settings.extra.len(), 0);
}

#[test]
fn settings_mut_allows_modification() {
    let mut svc = SettingsService::default();
    svc.settings_mut().last_workspace = Some("/workspace".to_string());
    svc.settings_mut().theme = "light".to_string();
    svc.settings_mut()
        .extra
        .insert("key".to_string(), "value".to_string());

    let settings = svc.settings();
    assert_eq!(settings.last_workspace.as_deref(), Some("/workspace"));
    assert_eq!(settings.theme, "light");
    assert_eq!(settings.extra.get("key").map(|s| s.as_str()), Some("value"));
}

#[test]
fn default_trait_matches_new() {
    let from_default = SettingsService::default();
    let from_new = SettingsService::new(Box::new(katana_platform::InMemoryRepository));
    // Both should produce equivalent default settings
    assert_eq!(
        from_new.settings().last_workspace,
        from_default.settings().last_workspace
    );
    assert_eq!(from_new.settings().theme, from_default.settings().theme,);
    assert_eq!(
        from_new.settings().extra.len(),
        from_default.settings().extra.len(),
    );
}

#[test]
fn json_repository_roundtrip() {
    let tmp = tempfile::TempDir::new().unwrap();
    let path = tmp.path().join("settings.json");

    let repo = katana_platform::JsonFileRepository::new(path.clone());
    let mut svc = SettingsService::new(Box::new(repo));
    svc.settings_mut().theme = "light".to_string();
    svc.settings_mut().language = "ja".to_string();
    svc.save().unwrap();

    // Reload from the same file
    let repo2 = katana_platform::JsonFileRepository::new(path);
    let svc2 = SettingsService::new(Box::new(repo2));
    assert_eq!(svc2.settings().theme, "light");
    assert_eq!(svc2.settings().language, "ja");
}
