
use katana_ui::app_state::AppState;
use katana_ui::shell::KatanaApp;

#[test]
fn font_size_change_is_reflected_in_settings() {
    let state = AppState::new(
        katana_core::ai::AiProviderRegistry::new(),
        katana_core::plugin::PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    let mut app = KatanaApp::new(state);

    let default_size = app
        .app_state_mut()
        .config
        .settings
        .settings()
        .clamped_font_size();
    assert!((default_size - 14.0).abs() < f32::EPSILON);

    app.app_state_mut()
        .config
        .settings
        .settings_mut()
        .set_font_size(22.0);

    let new_size = app
        .app_state_mut()
        .config
        .settings
        .settings()
        .clamped_font_size();
    assert!((new_size - 22.0).abs() < f32::EPSILON);
}

#[test]
fn font_size_out_of_range_is_clamped_in_app() {
    let state = AppState::new(
        katana_core::ai::AiProviderRegistry::new(),
        katana_core::plugin::PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    let mut app = KatanaApp::new(state);

    app.app_state_mut()
        .config
        .settings
        .settings_mut()
        .set_font_size(99.0);
    let size = app
        .app_state_mut()
        .config
        .settings
        .settings()
        .clamped_font_size();
    assert!((size - 32.0).abs() < f32::EPSILON);

    app.app_state_mut()
        .config
        .settings
        .settings_mut()
        .set_font_size(1.0);
    let size = app
        .app_state_mut()
        .config
        .settings
        .settings()
        .clamped_font_size();
    assert!((size - 8.0).abs() < f32::EPSILON);
}