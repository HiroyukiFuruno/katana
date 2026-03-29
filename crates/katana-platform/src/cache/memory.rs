use crate::cache::{read_guard, write_guard, CacheFacade};
use std::sync::RwLock;

// WHY: An in-memory only CacheFacade for tests.
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
            if let Some(entry) = map.get_mut(pos) {
                entry.1 = value;
            }
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
            if let Some(entry) = data.get_mut(pos) {
                entry.1 = value;
            }
        } else {
            data.push((key.to_string(), value));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
