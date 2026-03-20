#![deny(warnings)]

pub mod cache;
pub mod filesystem;
pub mod os_fonts;
pub mod os_theme;
pub mod settings;
pub mod theme;

pub use cache::{CacheFacade, DefaultCacheService, InMemoryCacheService};
pub use filesystem::FilesystemService;
pub use settings::{
    AppSettings, InMemoryRepository, JsonFileRepository, PaneOrder, SettingsRepository,
    SettingsService, SplitDirection, MAX_FONT_SIZE, MIN_FONT_SIZE,
};
pub use theme::{ThemeColors, ThemeMode, ThemePreset};
