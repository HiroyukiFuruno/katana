//! General-purpose caching facade for Katana.
//!
//! Provides both an in-memory ephemeral cache and a persistent on-disk cache.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// A Facade for managing both ephemeral (in-memory) and durable (persistent) caches.
pub trait CacheFacade: Send + Sync {
    /// Retrieves a value from the in-memory cache.
    fn get_memory(&self, key: &str) -> Option<String>;
    /// Stores a value in the in-memory cache. Note: this does not persist across application restarts.
    fn set_memory(&self, key: &str, value: String);

    /// Retrieves a value from the persistent cache.
    fn get_persistent(&self, key: &str) -> Option<String>;
    /// Stores a value in the persistent cache, syncing to disk.
    fn set_persistent(&self, key: &str, value: String) -> anyhow::Result<()>;
}

#[derive(Serialize, Deserialize, Default)]
struct PersistentData {
    entries: Vec<(String, String)>,
}

/// The default implementation of the `CacheFacade` using a JSON file for persistence.
pub struct DefaultCacheService {
    memory: RwLock<Vec<(String, String)>>,
    persistent_path: PathBuf,
    persistent: RwLock<PersistentData>,
}

impl DefaultCacheService {
    /// Creates a new `DefaultCacheService` with the specified persistent path.
    pub fn new(persistent_path: PathBuf) -> Self {
        let persistent = Self::load_persistent(&persistent_path).unwrap_or_default();
        Self {
            memory: RwLock::new(Vec::new()),
            persistent_path,
            persistent: RwLock::new(persistent),
        }
    }

    /// Creates a new `DefaultCacheService` with the standard OS cache directory.
    pub fn with_default_path() -> Self {
        let base = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
        Self::new(base.join("KatanA").join("cache.json"))
    }

    fn load_persistent(path: &PathBuf) -> Option<PersistentData> {
        let json_str = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&json_str).ok()
    }

    fn save_persistent(&self) -> anyhow::Result<()> {
        if let Some(parent) = self
            .persistent_path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)?;
        }
        let data = read_guard(&self.persistent);
        let json = serde_json::to_string_pretty(&*data)?;
        std::fs::write(&self.persistent_path, json)?;
        Ok(())
    }

    /// Clears all subdirectories in the Katana cache directory (e.g., http-image-cache, plantuml, tmp)
    /// while preserving files in the root like `cache.json`.
    pub fn clear_all_directories() {
        let base = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("KatanA");
        Self::clear_all_directories_in(&base);
    }

    pub fn clear_all_directories_in(base: &std::path::Path) {
        if let Ok(entries) = std::fs::read_dir(base) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        let path = entry.path();
                        // Best effort clear contents to prevent macOS directory not empty errors
                        if let Ok(sub_entries) = std::fs::read_dir(&path) {
                            for sub_entry in sub_entries.flatten() {
                                let _ = std::fs::remove_file(sub_entry.path());
                            }
                        }
                        let _ = std::fs::remove_dir_all(&path);
                    }
                }
            }
        }
    }
}

fn read_guard<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read().unwrap_or_else(PoisonError::into_inner)
}

fn write_guard<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    lock.write().unwrap_or_else(PoisonError::into_inner)
}

impl Default for DefaultCacheService {
    fn default() -> Self {
        Self::with_default_path()
    }
}

impl CacheFacade for DefaultCacheService {
    fn get_memory(&self, key: &str) -> Option<String> {
        let map = read_guard(&self.memory);
        map.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
    }

    fn set_memory(&self, key: &str, value: String) {
        let mut map = write_guard(&self.memory);
        if let Some(pos) = map.iter().position(|(k, _)| k == key) {
            map[pos].1 = value;
        } else {
            map.push((key.to_string(), value));
        }
    }

    fn get_persistent(&self, key: &str) -> Option<String> {
        let data = read_guard(&self.persistent);
        data.entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }

    fn set_persistent(&self, key: &str, value: String) -> anyhow::Result<()> {
        {
            let mut data = write_guard(&self.persistent);
            if let Some(pos) = data.entries.iter().position(|(k, _)| k == key) {
                data.entries[pos].1 = value;
            } else {
                data.entries.push((key.to_string(), value));
            }
        }
        self.save_persistent()
    }
}

/// An in-memory only CacheFacade for tests.
#[derive(Default)]
pub struct InMemoryCacheService {
    memory: RwLock<Vec<(String, String)>>,
    persistent: RwLock<Vec<(String, String)>>,
}

impl CacheFacade for InMemoryCacheService {
    fn get_memory(&self, key: &str) -> Option<String> {
        let map = read_guard(&self.memory);
        map.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
    }

    fn set_memory(&self, key: &str, value: String) {
        let mut map = write_guard(&self.memory);
        if let Some(pos) = map.iter().position(|(k, _)| k == key) {
            map[pos].1 = value;
        } else {
            map.push((key.to_string(), value));
        }
    }

    fn get_persistent(&self, key: &str) -> Option<String> {
        let data = read_guard(&self.persistent);
        data.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
    }

    fn set_persistent(&self, key: &str, value: String) -> anyhow::Result<()> {
        let mut data = write_guard(&self.persistent);
        if let Some(pos) = data.iter().position(|(k, _)| k == key) {
            data[pos].1 = value;
        } else {
            data.push((key.to_string(), value));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use tempfile::TempDir;

    #[test]
    fn test_memory_cache() {
        let cache = DefaultCacheService::new(PathBuf::from("dummy.json"));
        assert_eq!(cache.get_memory("test"), None);
        cache.set_memory("test", "data".to_string());
        assert_eq!(cache.get_memory("test"), Some("data".to_string()));

        // test update
        cache.set_memory("test", "data2".to_string());
        assert_eq!(cache.get_memory("test"), Some("data2".to_string()));
    }

    #[test]
    fn test_persistent_cache() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("cache.json");
        let cache = DefaultCacheService::new(path.clone());

        assert_eq!(cache.get_persistent("key"), None);
        cache.set_persistent("key", "val".to_string()).unwrap();
        assert_eq!(cache.get_persistent("key"), Some("val".to_string()));

        // test update
        cache.set_persistent("key", "val2".to_string()).unwrap();
        assert_eq!(cache.get_persistent("key"), Some("val2".to_string()));

        // re-load
        let cache2 = DefaultCacheService::new(path);
        assert_eq!(cache2.get_persistent("key"), Some("val2".to_string()));
    }

    #[test]
    fn test_default_cache_initialization() {
        let cache = DefaultCacheService::default();
        let _cache_clone = DefaultCacheService::with_default_path();

        // We just verify it doesn't crash, because the default path varies by OS
        assert_eq!(cache.get_persistent("non-existent"), None);
    }

    #[test]
    fn test_clear_all_directories() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();

        // Create an empty dir
        std::fs::create_dir(base.join("empty_dir")).unwrap();

        // Create a dir with files
        let full_dir = base.join("full_dir");
        std::fs::create_dir(&full_dir).unwrap();
        std::fs::write(full_dir.join("file.txt"), b"test").unwrap();

        // Create a root file that should be ignored
        let root_file = base.join("cache.json");
        std::fs::write(&root_file, b"test").unwrap();

        DefaultCacheService::clear_all_directories_in(base);

        assert!(!base.join("empty_dir").exists());
        assert!(!full_dir.exists());
        assert!(root_file.exists());

        // Cover dirs::cache_dir() invocation mapping
        DefaultCacheService::clear_all_directories();
    }

    #[test]
    fn test_in_memory_cache_service() {
        let cache = InMemoryCacheService::default();

        assert_eq!(cache.get_memory("test"), None);
        cache.set_memory("test", "val1".to_string());
        assert_eq!(cache.get_memory("test"), Some("val1".to_string()));
        cache.set_memory("test", "val2".to_string());
        assert_eq!(cache.get_memory("test"), Some("val2".to_string()));

        assert_eq!(cache.get_persistent("pkey"), None);
        cache.set_persistent("pkey", "pval1".to_string()).unwrap();
        assert_eq!(cache.get_persistent("pkey"), Some("pval1".to_string()));
        cache.set_persistent("pkey", "pval2".to_string()).unwrap();
        assert_eq!(cache.get_persistent("pkey"), Some("pval2".to_string()));
    }

    #[test]
    fn test_cache_recovers_from_poisoned_memory_lock() {
        let cache = DefaultCacheService::new(PathBuf::from("dummy.json"));
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = cache
                .memory
                .write()
                .expect("poison test must acquire write lock");
            panic!("poison memory lock");
        }));

        cache.set_memory("test", "recovered".to_string());
        assert_eq!(cache.get_memory("test"), Some("recovered".to_string()));
    }
}
