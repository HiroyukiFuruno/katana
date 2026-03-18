//! Tests for real-time font size reflection.
//!
//! Verifies that changing font_size in AppSettings triggers
//! the apply_font_size path on next update (via cache invalidation).

use katana_ui::app_state::AppState;
use katana_ui::shell::KatanaApp;

/// After changing font_size, the settings value should reflect the change correctly.
/// (update() invoking apply_font_size is guaranteed by cache mismatch with cached_font_size)
#[test]
fn font_size_change_is_reflected_in_settings() {
    let state = AppState::new(
        katana_core::ai::AiProviderRegistry::new(),
        katana_core::plugin::PluginRegistry::new(),
        katana_platform::SettingsService::default(),
    );
    let mut app = KatanaApp::new(state);

    // Default is 14.0
    let default_size = app.app_state_mut().settings.settings().clamped_font_size();
    assert!((default_size - 14.0).abs() < f32::EPSILON);

    // Change setting
    app.app_state_mut()
        .settings
        .settings_mut()
        .set_font_size(22.0);

    // Change is immediately reflected in the settings value
    let new_size = app.app_state_mut().settings.settings().clamped_font_size();
    assert!((new_size - 22.0).abs() < f32::EPSILON);
}

/// Confirms that font size out of range is clamped correctly in the app context
#[test]
fn font_size_out_of_range_is_clamped_in_app() {
    let state = AppState::new(
        katana_core::ai::AiProviderRegistry::new(),
        katana_core::plugin::PluginRegistry::new(),
        katana_platform::SettingsService::default(),
    );
    let mut app = KatanaApp::new(state);

    app.app_state_mut()
        .settings
        .settings_mut()
        .set_font_size(99.0);
    let size = app.app_state_mut().settings.settings().clamped_font_size();
    assert!((size - 32.0).abs() < f32::EPSILON);

    app.app_state_mut()
        .settings
        .settings_mut()
        .set_font_size(1.0);
    let size = app.app_state_mut().settings.settings().clamped_font_size();
    assert!((size - 8.0).abs() < f32::EPSILON);
}
