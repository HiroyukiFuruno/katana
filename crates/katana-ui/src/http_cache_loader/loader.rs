use std::{path::PathBuf, sync::Arc, task::Poll};

use egui::{
    load::{BytesLoadResult, BytesLoader, BytesPoll, LoadError},
    mutex::Mutex,
    Context,
};

use super::{
    disk::{
        cache_key, default_http_cache_dir, read_cached_file, remove_cache_file,
        CACHE_BODY_EXTENSION, CACHE_META_EXTENSION,
    },
    fetch::{entry_to_bytes_result, is_http_uri, process_fetch_response},
    types::{CachedFile, HttpCacheEntry},
};

pub struct PersistentHttpLoader {
    cache: Arc<Mutex<Vec<HttpCacheEntry>>>,
    cache_dir: PathBuf,
}

impl PersistentHttpLoader {
    pub const ID: &'static str = egui::generate_loader_id!(PersistentHttpLoader);

    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache: Arc::new(Mutex::new(Vec::new())),
            cache_dir,
        }
    }

    #[cfg(test)]
    pub(crate) fn cache_paths(&self, uri: &str) -> (PathBuf, PathBuf) {
        let key = cache_key(uri);
        let body = self.cache_dir.join(format!("{key}.{CACHE_BODY_EXTENSION}"));
        let meta = self.cache_dir.join(format!("{key}.{CACHE_META_EXTENSION}"));
        (body, meta)
    }

    pub(crate) fn read_from_disk(&self, uri: &str) -> Option<CachedFile> {
        let key = cache_key(uri);
        let body_path = self.cache_dir.join(format!("{key}.{CACHE_BODY_EXTENSION}"));
        let meta_path = self.cache_dir.join(format!("{key}.{CACHE_META_EXTENSION}"));
        read_cached_file(&body_path, &meta_path)
    }

    #[cfg(test)]
    pub(crate) fn write_to_disk(&self, uri: &str, file: &CachedFile) -> anyhow::Result<()> {
        use super::disk::write_cached_file;
        let key = cache_key(uri);
        let body_path = self.cache_dir.join(format!("{key}.{CACHE_BODY_EXTENSION}"));
        let meta_path = self.cache_dir.join(format!("{key}.{CACHE_META_EXTENSION}"));
        write_cached_file(&body_path, &meta_path, file)
    }

    pub(crate) fn remove_from_disk(&self, uri: &str) {
        let key = cache_key(uri);
        let body_path = self.cache_dir.join(format!("{key}.{CACHE_BODY_EXTENSION}"));
        let meta_path = self.cache_dir.join(format!("{key}.{CACHE_META_EXTENSION}"));
        remove_cache_file(&body_path, &meta_path);
    }

    #[cfg(test)]
    pub(crate) fn get_cache_mutex(&self) -> &Arc<Mutex<Vec<HttpCacheEntry>>> {
        &self.cache
    }
}

impl Default for PersistentHttpLoader {
    fn default() -> Self {
        Self::new(default_http_cache_dir())
    }
}

impl BytesLoader for PersistentHttpLoader {
    fn id(&self) -> &str {
        Self::ID
    }

    fn load(&self, ctx: &Context, uri: &str) -> BytesLoadResult {
        if !is_http_uri(uri) {
            return Err(LoadError::NotSupported);
        }

        let mut cache = self.cache.lock();
        if let Some(entry) = cache.iter().find(|e| e.uri == uri).map(|e| e.entry.clone()) {
            return entry_to_bytes_result(entry);
        }

        if let Some(file) = self.read_from_disk(uri) {
            let entry = Poll::Ready(Ok(file.clone()));
            cache.push(HttpCacheEntry {
                uri: uri.to_owned(),
                entry: entry.clone(),
            });
            return entry_to_bytes_result(entry);
        }

        let uri_clone = uri.to_owned();
        cache.push(HttpCacheEntry {
            uri: uri_clone.clone(),
            entry: Poll::Pending,
        });
        drop(cache);

        let cache = Arc::clone(&self.cache);
        let cache_dir = self.cache_dir.clone();
        let repaint_ctx = ctx.clone();

        ehttp::fetch(ehttp::Request::get(uri_clone.clone()), move |response| {
            let result = process_fetch_response(&uri_clone, &cache_dir, response);

            let repaint = {
                let mut cache = cache.lock();
                if let Some(entry) = cache.iter_mut().find(|e| e.uri == uri_clone) {
                    entry.entry = Poll::Ready(result);
                    true
                } else {
                    false
                }
            };

            if repaint {
                repaint_ctx.request_repaint();
            }
        });

        Ok(BytesPoll::Pending { size: None })
    }

    fn forget(&self, uri: &str) {
        self.cache.lock().retain(|e| e.uri != uri);
        self.remove_from_disk(uri);
    }

    fn forget_all(&self) {
        self.cache.lock().clear();

        let Ok(entries) = std::fs::read_dir(&self.cache_dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Err(err) = std::fs::remove_dir_all(&path) {
                    tracing::warn!("Failed to clear cache subdir {}: {err}", path.display());
                }
            } else if let Err(err) = std::fs::remove_file(&path) {
                tracing::warn!("Failed to delete cache file {}: {err}", path.display());
            }
        }
    }

    fn byte_size(&self) -> usize {
        self.cache
            .lock()
            .iter()
            .map(|e| match &e.entry {
                Poll::Ready(Ok(file)) => {
                    file.bytes.len() + file.mime.as_ref().map_or(0, String::len)
                }
                Poll::Ready(Err(err)) => err.len(),
                Poll::Pending => 0,
            })
            .sum()
    }

    fn has_pending(&self) -> bool {
        self.cache.lock().iter().any(|e| e.entry.is_pending())
    }
}