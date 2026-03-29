pub mod defaults;
pub mod impls;
pub mod migration;
pub mod repository;
pub mod service;
pub mod types;

// WHY: Public API re-exports to preserve `use crate::settings::*` compatibility.
pub use defaults::default_true;
pub use repository::{InMemoryRepository, JsonFileRepository, SettingsRepository};
pub use service::SettingsService;
pub use types::*;

#[cfg(test)]
mod tests;
