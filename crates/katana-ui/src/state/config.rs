use katana_core::plugin::PluginRegistry;
use katana_platform::{CacheFacade, SettingsService};
use std::sync::Arc;

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum SettingsSection {
    #[default]
    Appearance,
    Behavior,
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum SettingsTab {
    #[default]
    Theme,
    Font,
    Layout,
    Workspace,
    Updates,
    Behavior,
}

impl SettingsTab {
    pub const fn section(&self) -> SettingsSection {
        match self {
            Self::Theme | Self::Font | Self::Layout => SettingsSection::Appearance,
            Self::Workspace | Self::Updates | Self::Behavior => SettingsSection::Behavior,
        }
    }
}

impl SettingsSection {
    pub const fn tabs(&self) -> &[SettingsTab] {
        match self {
            Self::Appearance => &[SettingsTab::Theme, SettingsTab::Font, SettingsTab::Layout],
            Self::Behavior => &[
                SettingsTab::Workspace,
                SettingsTab::Updates,
                SettingsTab::Behavior,
            ],
        }
    }
}

pub struct ConfigState {
    pub plugin_registry: PluginRegistry,
    pub settings: SettingsService,
    pub cache: Arc<dyn CacheFacade>,
    pub active_settings_tab: SettingsTab,
    pub active_settings_section: SettingsSection,
    pub settings_tree_force_open: Option<bool>,
}

impl ConfigState {
    pub fn new(
        plugin_registry: PluginRegistry,
        settings: SettingsService,
        cache: Arc<dyn CacheFacade>,
    ) -> Self {
        Self {
            plugin_registry,
            settings,
            cache,
            active_settings_tab: SettingsTab::default(),
            active_settings_section: SettingsSection::default(),
            settings_tree_force_open: None,
        }
    }
}