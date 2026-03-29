use katana_core::ai::AiProviderRegistry;
use katana_core::plugin::PluginRegistry;
use katana_ui::app_state::*;

#[test]
fn settings_tab_default_is_theme() {
    assert_eq!(SettingsTab::default(), SettingsTab::Theme);
}

#[test]
fn settings_tab_variants_are_distinct() {
    assert_ne!(SettingsTab::Theme, SettingsTab::Font);
    assert_ne!(SettingsTab::Font, SettingsTab::Layout);
    assert_ne!(SettingsTab::Theme, SettingsTab::Layout);
}

#[test]
fn app_state_show_settings_defaults_to_false() {
    let state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    assert!(!state.layout.show_settings);
}

#[test]
fn app_state_active_settings_tab_defaults_to_theme() {
    let state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    assert_eq!(state.config.active_settings_tab, SettingsTab::Theme);
}

#[test]
fn show_settings_can_be_toggled() {
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );

    assert!(!state.layout.show_settings);
    state.layout.show_settings = true;
    assert!(state.layout.show_settings);
    state.layout.show_settings = false;
    assert!(!state.layout.show_settings);
}

#[test]
fn active_settings_tab_can_be_changed() {
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );

    assert_eq!(state.config.active_settings_tab, SettingsTab::Theme);
    state.config.active_settings_tab = SettingsTab::Font;
    assert_eq!(state.config.active_settings_tab, SettingsTab::Font);
    state.config.active_settings_tab = SettingsTab::Layout;
    assert_eq!(state.config.active_settings_tab, SettingsTab::Layout);
}

#[test]
fn settings_tab_clone_and_copy_work() {
    let tab = SettingsTab::Font;
    let cloned = tab;
    assert_eq!(tab, cloned);
}

#[test]
fn settings_tab_debug_format() {
    let tab = SettingsTab::Layout;
    let debug = format!("{tab:?}");
    assert!(debug.contains("Layout"));
}
