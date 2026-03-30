/* WHY: Application settings service.

`SettingsService` handles reading and writing settings via the repository,
and manages OS integration (theme, language) on first launch. */

use super::defaults::select_initial_preset;
use super::repository::{InMemoryRepository, SettingsRepository};
use super::types::{AppSettings, SettingsLoadOrigin};

// WHY: Platform settings service.
pub struct SettingsService {
    settings: AppSettings,
    repository: Box<dyn SettingsRepository>,
    // WHY: `true` when the settings were first loaded without an existing settings file.
    is_first_launch: bool,
}

impl SettingsService {
    // WHY: Create a new service backed by the given repository, loading initial settings.
    pub fn new(repository: Box<dyn SettingsRepository>) -> Self {
        let is_first_launch = repository.load_origin() == SettingsLoadOrigin::FirstLaunch;
        let settings = repository.load();
        Self {
            settings,
            repository,
            is_first_launch,
        }
    }

    pub fn settings(&self) -> &AppSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut AppSettings {
        &mut self.settings
    }

    // WHY: Persist current settings via the repository.
    #[allow(clippy::missing_errors_doc)]
    pub fn save(&self) -> anyhow::Result<()> {
        self.repository.save(&self.settings)
    }

    /* WHY: Applies the OS-default theme preset on first launch only.
    If this is not a first launch (settings file already existed), this is a no-op
    to respect the user's saved theme preference. */
    pub fn apply_os_default_theme(&mut self) {
        if !self.is_first_launch {
            return; // WHY: Existing users keep their saved preset unchanged.
        }
        let preset = select_initial_preset();
        self.settings.theme.preset = preset;
        self.settings.theme.theme = preset.colors().mode.to_theme_string();
    }

    // WHY: Applies the OS-default language on first launch.
    pub fn apply_os_default_language(&mut self, detected_lang: Option<String>) {
        if !self.is_first_launch {
            return;
        }
        if let Some(lang) = detected_lang {
            self.settings.language = lang;
        }
    }
}

impl Default for SettingsService {
    fn default() -> Self {
        Self::new(Box::new(InMemoryRepository))
    }
}