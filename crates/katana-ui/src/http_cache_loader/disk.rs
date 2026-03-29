use std::{
    fmt::Write as _,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

use super::types::{CacheMetadata, CachedFile};

pub(crate) const CACHE_NAMESPACE_DIR: &str = "KatanA";
pub(crate) const HTTP_IMAGE_CACHE_DIR: &str = "http-image-cache";
pub(crate) const CACHE_BODY_EXTENSION: &str = "bin";
pub(crate) const CACHE_META_EXTENSION: &str = "json";

pub(crate) fn default_http_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(CACHE_NAMESPACE_DIR)
        .join(HTTP_IMAGE_CACHE_DIR)
}

pub(crate) fn cache_key(uri: &str) -> String {
    let digest = Sha256::digest(uri.as_bytes());
    let mut key = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(&mut key, "{byte:02x}");
    }
    key
}

pub(crate) fn write_cached_file_for_uri(
    cache_dir: &Path,
    uri: &str,
    file: &CachedFile,
) -> anyhow::Result<()> {
    let key = cache_key(uri);
    let body_path = cache_dir.join(format!("{key}.{CACHE_BODY_EXTENSION}"));
    let meta_path = cache_dir.join(format!("{key}.{CACHE_META_EXTENSION}"));
    write_cached_file(&body_path, &meta_path, file)
}

pub(crate) fn write_cached_file(
    body_path: &Path,
    meta_path: &Path,
    file: &CachedFile,
) -> anyhow::Result<()> {
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

pub(crate) fn read_cached_file(body_path: &Path, meta_path: &Path) -> Option<CachedFile> {
    let bytes = std::fs::read(body_path).ok()?;
    let metadata = std::fs::read(meta_path)
        .ok()
        .and_then(|raw| serde_json::from_slice::<CacheMetadata>(&raw).ok())?;
    Some(CachedFile {
        bytes: bytes.into(),
        mime: metadata.mime,
    })
}

pub(crate) fn remove_cache_file(body_path: &Path, meta_path: &Path) {
    for path in [body_path, meta_path] {
        if let Err(err) = std::fs::remove_file(path) {
            if err.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!("Failed to remove cache file {}: {err}", path.display());
            }
        }
    }
}
