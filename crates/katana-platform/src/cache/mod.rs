/* WHY: General-purpose caching facade for Katana.

Provides both an in-memory ephemeral cache and a persistent on-disk cache. */

mod default;
mod memory;

pub use default::DefaultCacheService;
pub use memory::InMemoryCacheService;

use serde::{Deserialize, Serialize};
use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

// WHY: A Facade for managing both ephemeral (in-memory) and durable (persistent) caches.
pub trait CacheFacade: Send + Sync {
    // WHY: Retrieves a value from the in-memory cache.
    fn get_memory(&self, key: &str) -> Option<String>;
    // WHY: Stores a value in the in-memory cache. Note: this does not persist across application restarts.
    fn set_memory(&self, key: &str, value: String);

    // WHY: Retrieves a value from the persistent cache.
    fn get_persistent(&self, key: &str) -> Option<String>;
    // WHY: Stores a value in the persistent cache, syncing to disk.
    #[allow(clippy::missing_errors_doc)]
    fn set_persistent(&self, key: &str, value: String) -> anyhow::Result<()>;
}

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct PersistentData {
    pub(crate) entries: Vec<(String, String)>,
}

pub(crate) fn read_guard<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read().unwrap_or_else(PoisonError::into_inner)
}

pub(crate) fn write_guard<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    lock.write().unwrap_or_else(PoisonError::into_inner)
}
