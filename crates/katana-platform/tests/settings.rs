use katana_platform::SettingsService;

#[test]
fn new_settings_service_has_defaults() {
    let svc = SettingsService::new();
    let settings = svc.settings();
    assert!(settings.last_workspace.is_none());
    assert!(settings.ai_provider.is_none());
    assert!(settings.extra.is_empty());
}

#[test]
fn load_from_returns_defaults_in_mvp() {
    let svc = SettingsService::load_from("/nonexistent/path");
    let settings = svc.settings();
    assert!(settings.last_workspace.is_none());
}

#[test]
fn settings_returns_immutable_reference() {
    let svc = SettingsService::new();
    let settings = svc.settings();
    // settings は不変参照なので変更できないことをコンパイル時に保証
    assert_eq!(settings.extra.len(), 0);
}

#[test]
fn settings_mut_allows_modification() {
    let mut svc = SettingsService::new();
    svc.settings_mut().last_workspace = Some("/workspace".to_string());
    svc.settings_mut().ai_provider = Some("openai".to_string());
    svc.settings_mut()
        .extra
        .insert("theme".to_string(), "dark".to_string());

    let settings = svc.settings();
    assert_eq!(settings.last_workspace.as_deref(), Some("/workspace"));
    assert_eq!(settings.ai_provider.as_deref(), Some("openai"));
    assert_eq!(
        settings.extra.get("theme").map(|s| s.as_str()),
        Some("dark")
    );
}

#[test]
fn default_trait_matches_new() {
    let from_new = SettingsService::new();
    let from_default = SettingsService::default();
    // Both should produce equivalent default settings
    assert_eq!(
        from_new.settings().last_workspace,
        from_default.settings().last_workspace
    );
    assert_eq!(
        from_new.settings().ai_provider,
        from_default.settings().ai_provider,
    );
    assert_eq!(
        from_new.settings().extra.len(),
        from_default.settings().extra.len(),
    );
}
