#![cfg(test)]
#[cfg(test)]
mod tests {
    use crate::http_cache_loader::disk::{
        cache_key, read_cached_file, remove_cache_file, write_cached_file_for_uri,
        CACHE_BODY_EXTENSION, CACHE_META_EXTENSION,
    };
    use crate::http_cache_loader::fetch::{entry_to_bytes_result, process_fetch_response};
    use crate::http_cache_loader::loader::PersistentHttpLoader;
    use crate::http_cache_loader::types::{CachedFile, HttpCacheEntry};
    use egui::load::{BytesLoader, BytesPoll};
    use egui::Context;
    use std::sync::Arc;
    use std::task::Poll;
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

        assert!(!std::path::Path::new(&body_path).exists());
        assert!(!std::path::Path::new(&meta_path).exists());
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
                assert!(err.to_string().contains("404"));
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
        let entry: crate::http_cache_loader::types::Entry = Poll::Ready(Ok(file.clone()));
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
        let entry: crate::http_cache_loader::types::Entry =
            Poll::Ready(Err("load failed".to_string()));
        let result = entry_to_bytes_result(entry);
        assert!(result.is_err());
    }

    #[test]
    fn entry_to_bytes_result_pending() {
        let entry: crate::http_cache_loader::types::Entry = Poll::Pending;
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
        loader.get_cache_mutex().lock().push(HttpCacheEntry {
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
            let mut cache = loader.get_cache_mutex().lock();
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

        loader.get_cache_mutex().lock().push(HttpCacheEntry {
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
        loader.get_cache_mutex().lock().push(HttpCacheEntry {
            uri: "a".to_owned(),
            entry: Poll::Ready(Ok(file)),
        });

        loader.forget_all();
        assert!(loader.get_cache_mutex().lock().is_empty());
        // cache_dir itself should be KEPT, but its contents should be removed
        assert!(tmp.path().exists());
        assert_eq!(std::fs::read_dir(tmp.path()).unwrap().count(), 0);
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

    #[test]
    fn forget_all_handles_subdirs_and_clears_safely() {
        let tmp = TempDir::new().expect("tempdir");
        let loader = PersistentHttpLoader::new(tmp.path().to_path_buf());

        // Create a dummy subdirectory inside the cache directory
        let subdir = tmp.path().join("dummy_subdir");
        std::fs::create_dir(&subdir).expect("mkdir");
        let file_in_subdir = subdir.join("nested.bin");
        std::fs::write(&file_in_subdir, b"nested").expect("write nested");

        // Execute forget_all
        loader.forget_all();

        // Ensure the subdir was deleted
        assert!(!subdir.exists(), "subdirectory should have been deleted");

        // Ensure the root cache_dir still exists
        assert!(tmp.path().exists(), "root cache_dir should NOT be deleted");
    }

    #[test]
    fn forget_all_warns_on_failed_subdir_deletion() {
        let tmp = TempDir::new().expect("tempdir");
        let loader = PersistentHttpLoader::new(tmp.path().to_path_buf());

        let subdir = tmp.path().join("protected_subdir");
        std::fs::create_dir(&subdir).expect("mkdir");

        // Make the PARENT directory read-only so remove_dir_all will fail when trying to delete it
        let mut perms = std::fs::metadata(tmp.path()).expect("meta").permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(tmp.path(), perms).expect("chmod");

        // Calling forget_all should hit the Err branch for remove_dir_all
        loader.forget_all();

        // Restore permissions so the TempDir can be cleaned up
        let mut perms = std::fs::metadata(tmp.path()).expect("meta").permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        perms.set_readonly(false);
        std::fs::set_permissions(tmp.path(), perms).expect("chmod restore");
    }
}
