#![allow(clippy::useless_vec)]
use egui::{
    load::{Bytes, BytesLoadResult, BytesLoader, BytesPoll, LoadError},
    mutex::Mutex,
    Context,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fmt::Write as _,
    path::{Path, PathBuf},
    sync::Arc,
    task::Poll,
};

const HTTP_PROTOCOL: &str = "http://";
const HTTPS_PROTOCOL: &str = "https://";
const CACHE_NAMESPACE_DIR: &str = "KatanA";
const HTTP_IMAGE_CACHE_DIR: &str = "http-image-cache";
const CACHE_BODY_EXTENSION: &str = "bin";
const CACHE_META_EXTENSION: &str = "json";

#[derive(Clone)]
struct CachedFile {
    bytes: Arc<[u8]>,
    mime: Option<String>,
}

impl CachedFile {
    fn from_response(uri: &str, response: ehttp::Response) -> Result<Self, String> {
        if !response.ok {
            match response.text() {
                Some(response_text) => Err(format!(
                    "failed to load {uri:?}: {} {} {response_text}",
                    response.status, response.status_text
                )),
                None => Err(format!(
                    "failed to load {uri:?}: {} {}",
                    response.status, response.status_text
                )),
            }
        } else {
            let mime = response.content_type().map(ToOwned::to_owned);
            let bytes = response.bytes.into();
            Ok(Self { bytes, mime })
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct CacheMetadata {
    mime: Option<String>,
}

type Entry = Poll<Result<CachedFile, String>>;

struct HttpCacheEntry {
    uri: String,
    entry: Entry,
}

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

    fn cache_paths(&self, uri: &str) -> (PathBuf, PathBuf) {
        let key = cache_key(uri);
        let body = self.cache_dir.join(format!("{key}.{CACHE_BODY_EXTENSION}"));
        let meta = self.cache_dir.join(format!("{key}.{CACHE_META_EXTENSION}"));
        (body, meta)
    }

    fn read_from_disk(&self, uri: &str) -> Option<CachedFile> {
        let (body_path, meta_path) = self.cache_paths(uri);
        read_cached_file(&body_path, &meta_path)
    }

    #[cfg(test)]
    fn write_to_disk(&self, uri: &str, file: &CachedFile) -> anyhow::Result<()> {
        let (body_path, meta_path) = self.cache_paths(uri);
        write_cached_file(&body_path, &meta_path, file)
    }

    fn remove_from_disk(&self, uri: &str) {
        let (body_path, meta_path) = self.cache_paths(uri);
        remove_cache_file(&body_path, &meta_path);
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

        let uri = uri.to_owned();
        cache.push(HttpCacheEntry {
            uri: uri.clone(),
            entry: Poll::Pending,
        });
        drop(cache);

        let cache = Arc::clone(&self.cache);
        let cache_dir = self.cache_dir.clone();
        let repaint_ctx = ctx.clone();
        ehttp::fetch(ehttp::Request::get(uri.clone()), move |response| {
            let result = process_fetch_response(&uri, &cache_dir, response);

            let repaint = {
                let mut cache = cache.lock();
                if let Some(entry) = cache.iter_mut().find(|e| e.uri == uri) {
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
        if let Err(err) = std::fs::remove_dir_all(&self.cache_dir) {
            if err.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!(
                    "Failed to clear HTTP image cache directory {}: {err}",
                    self.cache_dir.display()
                );
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

fn entry_to_bytes_result(entry: Entry) -> BytesLoadResult {
    match entry {
        Poll::Ready(Ok(file)) => Ok(BytesPoll::Ready {
            size: None,
            bytes: Bytes::Shared(file.bytes),
            mime: file.mime,
        }),
        Poll::Ready(Err(err)) => Err(LoadError::Loading(err)),
        Poll::Pending => Ok(BytesPoll::Pending { size: None }),
    }
}

/// Extracted from the `ehttp::fetch` callback to make the logic testable.
///
/// Converts a raw HTTP result into a `CachedFile`, persisting successful
/// responses to the disk cache.
fn process_fetch_response(
    uri: &str,
    cache_dir: &Path,
    response: ehttp::Result<ehttp::Response>,
) -> Result<CachedFile, String> {
    match response {
        Ok(response) => CachedFile::from_response(uri, response).inspect(|file| {
            if let Err(err) = write_cached_file_for_uri(cache_dir, uri, file) {
                tracing::warn!("Failed to persist HTTP image cache for {uri}: {err}");
            }
        }),
        Err(err) => {
            tracing::error!("Failed to load {uri:?}: {err}");
            Err(format!("Failed to load {uri:?}"))
        }
    }
}

fn is_http_uri(uri: &str) -> bool {
    uri.starts_with(HTTP_PROTOCOL) || uri.starts_with(HTTPS_PROTOCOL)
}

fn default_http_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(CACHE_NAMESPACE_DIR)
        .join(HTTP_IMAGE_CACHE_DIR)
}

fn cache_key(uri: &str) -> String {
    let digest = Sha256::digest(uri.as_bytes());
    let mut key = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(&mut key, "{byte:02x}");
    }
    key
}

fn write_cached_file_for_uri(cache_dir: &Path, uri: &str, file: &CachedFile) -> anyhow::Result<()> {
    let key = cache_key(uri);
    let body_path = cache_dir.join(format!("{key}.{CACHE_BODY_EXTENSION}"));
    let meta_path = cache_dir.join(format!("{key}.{CACHE_META_EXTENSION}"));
    write_cached_file(&body_path, &meta_path, file)
}

fn write_cached_file(body_path: &Path, meta_path: &Path, file: &CachedFile) -> anyhow::Result<()> {
    if let Some(parent) = body_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(body_path, &file.bytes)?;
    let metadata = CacheMetadata {
        mime: file.mime.clone(),
    };
    let meta_json = serde_json::to_vec(&metadata)?;
    std::fs::write(meta_path, meta_json)?;
    Ok(())
}

fn read_cached_file(body_path: &Path, meta_path: &Path) -> Option<CachedFile> {
    let bytes = std::fs::read(body_path).ok()?;
    let metadata = std::fs::read(meta_path)
        .ok()
        .and_then(|raw| serde_json::from_slice::<CacheMetadata>(&raw).ok())?;
    Some(CachedFile {
        bytes: bytes.into(),
        mime: metadata.mime,
    })
}

fn remove_cache_file(body_path: &Path, meta_path: &Path) {
    for path in vec![body_path, meta_path] {
        if let Err(err) = std::fs::remove_file(path) {
            if err.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!("Failed to remove cache file {}: {err}", path.display());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_file() -> CachedFile {
        CachedFile {
            bytes: Arc::<[u8]>::from(&b"cached-image"[..]),
            mime: Some("image/svg+xml".to_string()),
        }
    }

    #[test]
    fn cached_http_file_roundtrip_persists_bytes_and_mime() {
        let tmp = TempDir::new().expect("tempdir");
        let loader = PersistentHttpLoader::new(tmp.path().to_path_buf());
        let uri = "https://example.com/badge.svg";
        let file = sample_file();

        loader.write_to_disk(uri, &file).expect("write cache");
        let loaded = loader.read_from_disk(uri).expect("read cache");

        assert_eq!(&*loaded.bytes, &*file.bytes);
        assert_eq!(loaded.mime, file.mime);
    }

    #[test]
    fn load_uses_disk_cache_without_network() {
        let tmp = TempDir::new().expect("tempdir");
        let loader = PersistentHttpLoader::new(tmp.path().to_path_buf());
        let uri = "https://example.com/badge.svg";
        let file = sample_file();
        let ctx = Context::default();

        loader.write_to_disk(uri, &file).expect("write cache");
        let result = loader.load(&ctx, uri).expect("bytes result");

        match result {
            BytesPoll::Ready { bytes, mime, .. } => {
                assert_eq!(bytes.as_ref(), &*file.bytes);
                assert_eq!(mime, file.mime);
            }
            BytesPoll::Pending { .. } => panic!("disk cache hit must be ready immediately"),
        }
    }

    #[test]
    fn forget_removes_disk_cache_for_uri() {
        let tmp = TempDir::new().expect("tempdir");
        let loader = PersistentHttpLoader::new(tmp.path().to_path_buf());
        let uri = "https://example.com/badge.svg";
        let file = sample_file();
        let (body_path, meta_path) = loader.cache_paths(uri);

        loader.write_to_disk(uri, &file).expect("write cache");
        loader.forget(uri);

        assert!(!body_path.exists());
        assert!(!meta_path.exists());
    }

    #[test]
    fn cache_persists_across_loader_instances() {
        let tmp = TempDir::new().expect("tempdir");
        let uri = "https://example.com/badge.svg";
        let file = sample_file();
        let first = PersistentHttpLoader::new(tmp.path().to_path_buf());
        let second = PersistentHttpLoader::new(tmp.path().to_path_buf());

        first.write_to_disk(uri, &file).expect("write cache");
        let loaded = second.read_from_disk(uri).expect("read cache");

        assert_eq!(loaded.mime, file.mime);
        assert_eq!(&*loaded.bytes, &*file.bytes);
    }

    #[test]
    fn cached_file_from_response_success() {
        let response = ehttp::Response {
            url: "https://example.com/img.svg".to_string(),
            ok: true,
            status: 200,
            status_text: "OK".to_string(),
            headers: ehttp::Headers::new(&[("content-type", "image/svg+xml")]),
            bytes: b"<svg></svg>".to_vec(),
        };
        let file = CachedFile::from_response("https://example.com/img.svg", response)
            .expect("should succeed for ok response");
        assert_eq!(&*file.bytes, b"<svg></svg>");
        assert_eq!(file.mime.as_deref(), Some("image/svg+xml"));
    }

    #[test]
    fn cached_file_from_response_error_with_text() {
        let response = ehttp::Response {
            url: "https://example.com/img.svg".to_string(),
            ok: false,
            status: 404,
            status_text: "Not Found".to_string(),
            headers: ehttp::Headers::default(),
            bytes: b"page not found".to_vec(),
        };
        let result = CachedFile::from_response("https://example.com/img.svg", response);
        match result {
            Err(err) => {
                assert!(err.contains("404"));
                assert!(err.contains("page not found"));
            }
            Ok(_) => panic!("should fail for non-ok response"),
        }
    }

    #[test]
    fn cached_file_from_response_error_without_text() {
        let response = ehttp::Response {
            url: "https://example.com/img.svg".to_string(),
            ok: false,
            status: 500,
            status_text: "Internal Server Error".to_string(),
            headers: ehttp::Headers::default(),
            // Invalid UTF-8 bytes so text() returns None
            bytes: vec![0xFF, 0xFE],
        };
        let result = CachedFile::from_response("https://example.com/img.svg", response);
        match result {
            Err(err) => assert!(err.contains("500")),
            Ok(_) => panic!("should fail for non-ok response"),
        }
    }

    #[test]
    fn entry_to_bytes_result_ready_ok() {
        let file = sample_file();
        let entry: Entry = Poll::Ready(Ok(file.clone()));
        let result = entry_to_bytes_result(entry).expect("should be ok");
        match result {
            BytesPoll::Ready { bytes, mime, .. } => {
                assert_eq!(bytes.as_ref(), &*file.bytes);
                assert_eq!(mime, file.mime);
            }
            _ => panic!("expected Ready"),
        }
    }

    #[test]
    fn entry_to_bytes_result_ready_err() {
        let entry: Entry = Poll::Ready(Err("load failed".to_string()));
        let result = entry_to_bytes_result(entry);
        assert!(result.is_err());
    }

    #[test]
    fn entry_to_bytes_result_pending() {
        let entry: Entry = Poll::Pending;
        let result = entry_to_bytes_result(entry).expect("Pending is not an error");
        assert!(matches!(result, BytesPoll::Pending { .. }));
    }

    #[test]
    fn load_rejects_non_http_uri() {
        let tmp = TempDir::new().expect("tempdir");
        let loader = PersistentHttpLoader::new(tmp.path().to_path_buf());
        let ctx = Context::default();
        let result = loader.load(&ctx, "file:///tmp/image.svg");
        assert!(result.is_err());
    }

    #[test]
    fn load_returns_cached_entry_from_memory() {
        let tmp = TempDir::new().expect("tempdir");
        let loader = PersistentHttpLoader::new(tmp.path().to_path_buf());
        let ctx = Context::default();
        let uri = "https://example.com/badge.svg";
        let file = sample_file();

        // Pre-populate the in-memory cache
        loader.cache.lock().push(HttpCacheEntry {
            uri: uri.to_owned(),
            entry: Poll::Ready(Ok(file.clone())),
        });

        let result = loader.load(&ctx, uri).expect("should hit memory cache");
        match result {
            BytesPoll::Ready { bytes, mime, .. } => {
                assert_eq!(bytes.as_ref(), &*file.bytes);
                assert_eq!(mime, file.mime);
            }
            _ => panic!("expected Ready from memory cache"),
        }
    }

    #[test]
    fn load_triggers_fetch_for_uncached_uri() {
        let tmp = TempDir::new().expect("tempdir");
        let loader = PersistentHttpLoader::new(tmp.path().to_path_buf());
        let ctx = Context::default();
        // URI not on disk and not in memory — triggers ehttp::fetch → returns Pending
        let result = loader
            .load(&ctx, "https://example.com/new-badge.svg")
            .expect("should return Pending");
        assert!(matches!(result, BytesPoll::Pending { .. }));
    }

    #[test]
    fn byte_size_accounts_for_all_entries() {
        let tmp = TempDir::new().expect("tempdir");
        let loader = PersistentHttpLoader::new(tmp.path().to_path_buf());
        let file = sample_file();

        {
            let mut cache = loader.cache.lock();
            cache.push(HttpCacheEntry {
                uri: "ok".to_owned(),
                entry: Poll::Ready(Ok(file.clone())),
            });
            cache.push(HttpCacheEntry {
                uri: "err".to_owned(),
                entry: Poll::Ready(Err("error msg".to_string())),
            });
            cache.push(HttpCacheEntry {
                uri: "pending".to_owned(),
                entry: Poll::Pending,
            });
        }

        let size = loader.byte_size();
        let expected =
            file.bytes.len() + file.mime.as_ref().map_or(0, String::len) + "error msg".len(); // Pending contributes 0
        assert_eq!(size, expected);
    }

    #[test]
    fn has_pending_detects_pending_entries() {
        let tmp = TempDir::new().expect("tempdir");
        let loader = PersistentHttpLoader::new(tmp.path().to_path_buf());
        assert!(!loader.has_pending());

        loader.cache.lock().push(HttpCacheEntry {
            uri: "pending".to_owned(),
            entry: Poll::Pending,
        });
        assert!(loader.has_pending());
    }

    #[test]
    fn forget_all_clears_cache_and_disk() {
        let tmp = TempDir::new().expect("tempdir");
        let loader = PersistentHttpLoader::new(tmp.path().to_path_buf());
        let file = sample_file();

        loader
            .write_to_disk("https://example.com/a.svg", &file)
            .expect("write");
        loader.cache.lock().push(HttpCacheEntry {
            uri: "a".to_owned(),
            entry: Poll::Ready(Ok(file)),
        });

        loader.forget_all();
        assert!(loader.cache.lock().is_empty());
        // cache_dir itself should be removed
        assert!(!tmp.path().exists());
    }

    #[test]
    fn write_cached_file_for_uri_persists_to_disk() {
        let tmp = TempDir::new().expect("tempdir");
        let file = sample_file();
        let uri = "https://shields.io/badge.svg";

        write_cached_file_for_uri(tmp.path(), uri, &file).expect("write");

        let key = cache_key(uri);
        let body_path = tmp.path().join(format!("{key}.{CACHE_BODY_EXTENSION}"));
        let meta_path = tmp.path().join(format!("{key}.{CACHE_META_EXTENSION}"));
        assert!(body_path.exists());
        assert!(meta_path.exists());

        let loaded = read_cached_file(&body_path, &meta_path).expect("read back");
        assert_eq!(&*loaded.bytes, &*file.bytes);
        assert_eq!(loaded.mime, file.mime);
    }

    #[test]
    fn remove_cache_file_tolerates_missing_files() {
        let tmp = TempDir::new().expect("tempdir");
        let body = tmp.path().join("nonexistent.bin");
        let meta = tmp.path().join("nonexistent.json");
        // Should not panic
        remove_cache_file(&body, &meta);
    }

    // ── process_fetch_response ──

    #[test]
    fn process_fetch_response_success_persists_to_disk() {
        let tmp = TempDir::new().expect("tempdir");
        let uri = "https://example.com/badge.svg";
        let response = Ok(ehttp::Response {
            url: uri.to_string(),
            ok: true,
            status: 200,
            status_text: "OK".to_string(),
            headers: ehttp::Headers::new(&[("content-type", "image/svg+xml")]),
            bytes: b"<svg></svg>".to_vec(),
        });

        let result = process_fetch_response(uri, tmp.path(), response);
        assert!(result.is_ok());
        let file = result.unwrap();
        assert_eq!(&*file.bytes, b"<svg></svg>");

        // Verify persisted to disk
        let key = cache_key(uri);
        let body_path = tmp.path().join(format!("{key}.{CACHE_BODY_EXTENSION}"));
        assert!(body_path.exists());
    }

    #[test]
    fn process_fetch_response_error_response() {
        let tmp = TempDir::new().expect("tempdir");
        let uri = "https://example.com/404.svg";
        let response = Ok(ehttp::Response {
            url: uri.to_string(),
            ok: false,
            status: 404,
            status_text: "Not Found".to_string(),
            headers: ehttp::Headers::default(),
            bytes: b"not found".to_vec(),
        });

        let result = process_fetch_response(uri, tmp.path(), response);
        match result {
            Err(err) => assert!(err.contains("404")),
            Ok(_) => panic!("expected error"),
        }
    }

    #[test]
    fn process_fetch_response_network_error() {
        let tmp = TempDir::new().expect("tempdir");
        let uri = "https://example.com/unreachable.svg";
        let response: ehttp::Result<ehttp::Response> = Err("connection refused".to_string());

        let result = process_fetch_response(uri, tmp.path(), response);
        match result {
            Err(err) => assert!(err.contains(uri)),
            Ok(_) => panic!("expected error"),
        }
    }

    #[test]
    fn load_returns_disk_cached_entry() {
        let tmp = TempDir::new().expect("tempdir");
        let loader = PersistentHttpLoader::new(tmp.path().to_path_buf());
        let ctx = Context::default();
        let uri = "https://example.com/cached-badge.svg";
        let file = sample_file();

        // Pre-populate the disk cache
        loader.write_to_disk(uri, &file).expect("write disk cache");

        let result = loader.load(&ctx, uri).expect("should load from disk");
        match result {
            BytesPoll::Ready { bytes, mime, .. } => {
                assert_eq!(bytes.as_ref(), &*file.bytes);
                assert_eq!(mime, file.mime);
            }
            _ => panic!("expected Ready from disk cache"),
        }
    }

    #[test]
    fn forget_all_on_nonexistent_dir_is_safe() {
        let tmp = TempDir::new().expect("tempdir");
        let cache_dir = tmp.path().join("nonexistent_subdir");
        let loader = PersistentHttpLoader::new(cache_dir);
        // Should not panic — NotFound error is silently ignored
        loader.forget_all();
    }

    // ── IO error handling (tracing::warn branches) ──

    #[test]
    fn remove_cache_file_warns_on_non_not_found_error() {
        let tmp = TempDir::new().expect("tempdir");
        // Create a directory at the path — remove_file on a directory returns IsADirectory
        let dir_as_file = tmp.path().join("is_a_dir.bin");
        std::fs::create_dir(&dir_as_file).expect("mkdir");
        let meta = tmp.path().join("unused.json");
        // Should not panic — the IsADirectory error triggers the tracing::warn branch
        remove_cache_file(&dir_as_file, &meta);
    }

    #[test]
    fn forget_all_warns_on_non_not_found_io_error() {
        let tmp = TempDir::new().expect("tempdir");
        // Create a regular file at cache_dir path — remove_dir_all on a file returns
        // "not a directory" error, triggering the warn branch
        let file_as_dir = tmp.path().join("not_a_dir");
        std::fs::write(&file_as_dir, b"block").expect("write");
        let loader = PersistentHttpLoader::new(file_as_dir);
        // Should not panic — the non-NotFound error triggers tracing::warn
        loader.forget_all();
    }

    #[test]
    fn process_fetch_response_warns_on_write_failure() {
        let tmp = TempDir::new().expect("tempdir");
        // Make cache dir read-only so write_cached_file_for_uri fails
        let cache_dir = tmp.path().join("readonly_cache");
        std::fs::create_dir(&cache_dir).expect("mkdir");
        let mut perms = std::fs::metadata(&cache_dir).expect("meta").permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(&cache_dir, perms).expect("chmod");

        let uri = "https://example.com/write_fail.svg";
        let response = Ok(ehttp::Response {
            url: uri.to_string(),
            ok: true,
            status: 200,
            status_text: "OK".to_string(),
            headers: ehttp::Headers::new(&[("content-type", "image/svg+xml")]),
            bytes: b"<svg></svg>".to_vec(),
        });

        // Should succeed (returns Ok) but internally logs a warning about write failure
        let result = process_fetch_response(uri, &cache_dir, response);
        assert!(
            result.is_ok(),
            "response should parse even if disk write fails"
        );

        // Restore permissions for cleanup
        let mut perms = std::fs::metadata(&cache_dir).expect("meta").permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        perms.set_readonly(false);
        std::fs::set_permissions(&cache_dir, perms).expect("chmod restore");
    }
}
