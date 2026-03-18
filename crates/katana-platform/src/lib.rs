#![deny(warnings)]

pub mod filesystem;
pub mod settings;
pub mod theme;

pub use filesystem::FilesystemService;
pub use settings::{
    AppSettings, InMemoryRepository, JsonFileRepository, SettingsRepository, SettingsService,
    MAX_FONT_SIZE, MIN_FONT_SIZE,
};
pub use theme::{ThemeColors, ThemeMode, ThemePreset};
