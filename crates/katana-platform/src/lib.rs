#![deny(warnings, clippy::all)]
#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::too_many_lines,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented
)]
#![warn(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::missing_errors_doc
)]

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