use katana_core::workspace::TreeEntry;
use std::path::{Path, PathBuf};

/// Recursively and in parallel scans a directory, returning a tree containing only visible files.
#[allow(clippy::too_many_arguments)]
pub(crate) fn scan_directory(
    dir: &Path,
    ignored_directories: &[String],
    max_depth: usize,
    current_depth: usize,
    visible_extensions: &[String],
    extensionless_excludes: &[String],
    cancel_token: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    in_memory_dirs: &std::collections::HashSet<PathBuf>,
) -> std::io::Result<Vec<TreeEntry>> {
    if current_depth >= max_depth || cancel_token.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok(Vec::new());
    }

    use rayon::prelude::*;

    let iter = std::fs::read_dir(dir)?;
    let child_entries: Vec<_> = iter.filter_map(Result::ok).collect();

    let mut entries: Vec<TreeEntry> = child_entries
        .into_par_iter()
        .filter_map(|entry| {
            if cancel_token.load(std::sync::atomic::Ordering::Relaxed) {
                return None;
            }

            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str())?;

            if ignored_directories
                .iter()
                .any(|ignored| ignored == file_name)
            {
                return None;
            }

            if path.is_dir() {
                let children = scan_directory(
                    &path,
                    ignored_directories,
                    max_depth,
                    current_depth + 1,
                    visible_extensions,
                    extensionless_excludes,
                    cancel_token,
                    in_memory_dirs,
                )
                .unwrap_or_default();
                // Show directory if it has visible files OR it is an explicitly created empty directory.
                if has_any_visible(&children, visible_extensions)
                    || in_memory_dirs.contains(&path)
                {
                    Some(TreeEntry::Directory { path, children })
                } else {
                    None
                }
            } else {
                let is_visible = match path.extension().and_then(|e| e.to_str()) {
                    Some(ext) => visible_extensions
                        .iter()
                        .any(|v| v.eq_ignore_ascii_case(ext)),
                    None => {
                        let no_ext_enabled = visible_extensions.iter().any(|v| v.is_empty());
                        if no_ext_enabled {
                            let is_excluded =
                                extensionless_excludes.iter().any(|excl| excl == file_name);
                            !is_excluded
                        } else {
                            false
                        }
                    }
                };
                if is_visible {
                    Some(TreeEntry::File { path })
                } else {
                    None
                }
            }
        })
        .collect();

    // Sort: directories first, then files, both alphabetically.
    entries.sort_by(|a, b| match (a, b) {
        (TreeEntry::Directory { .. }, TreeEntry::File { .. }) => std::cmp::Ordering::Less,
        (TreeEntry::File { .. }, TreeEntry::Directory { .. }) => std::cmp::Ordering::Greater,
        (a, b) => a.path().cmp(b.path()),
    });
    Ok(entries)
}

/// Recursively checks if there is at least one visible file in the tree.
pub(crate) fn has_any_visible(entries: &[TreeEntry], visible_extensions: &[String]) -> bool {
    entries.iter().any(|e| match e {
        TreeEntry::File { path } => match path.extension().and_then(|ext| ext.to_str()) {
            Some(ext) => visible_extensions
                .iter()
                .any(|v| v.eq_ignore_ascii_case(ext)),
            None => visible_extensions.iter().any(|v| v.is_empty()),
        },
        TreeEntry::Directory { children, .. } => has_any_visible(children, visible_extensions),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_has_any_visible_with_empty_extension() {
        let file_with_no_ext = TreeEntry::File {
            path: PathBuf::from("no_extension_file"),
        };
        let file_with_ext = TreeEntry::File {
            path: PathBuf::from("file.md"),
        };

        // Without empty string in visible_extensions
        let visible_exts_without_empty = vec!["md".to_string()];
        assert!(!has_any_visible(
            std::slice::from_ref(&file_with_no_ext),
            &visible_exts_without_empty
        ));
        assert!(has_any_visible(
            std::slice::from_ref(&file_with_ext),
            &visible_exts_without_empty
        ));

        // With empty string in visible_extensions
        let visible_exts_with_empty = vec!["md".to_string(), "".to_string()];
        assert!(has_any_visible(
            std::slice::from_ref(&file_with_no_ext),
            &visible_exts_with_empty
        ));

        // Inside a directory
        let dir = TreeEntry::Directory {
            path: PathBuf::from("dir"),
            children: vec![file_with_no_ext],
        };
        assert!(!has_any_visible(
            std::slice::from_ref(&dir),
            &visible_exts_without_empty
        ));
        assert!(has_any_visible(&[dir], &visible_exts_with_empty));
    }

    #[test]
    fn test_scan_directory_empty_extension() {
        use std::fs;
        use std::sync::atomic::AtomicBool;
        use std::sync::Arc;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("no_ext_file");
        fs::write(&file_path, "test").unwrap();

        let cancel_token = Arc::new(AtomicBool::new(false));
        let in_memory_dirs = std::collections::HashSet::new();

        // 1. empty string in visible_extensions
        let tree_with_empty = scan_directory(
            dir.path(),
            &[],
            10,
            0,
            &["".to_string()],
            &[],
            &cancel_token,
            &in_memory_dirs,
        )
        .unwrap();

        assert_eq!(tree_with_empty.len(), 1);
        if let TreeEntry::File { path } = &tree_with_empty[0] {
            assert_eq!(path.file_name().unwrap(), "no_ext_file");
        } else {
            panic!("Expected file entry");
        }

        // 2. empty string not in visible_extensions
        let tree_without_empty = scan_directory(
            dir.path(),
            &[],
            10,
            0,
            &["md".to_string()],
            &[],
            &cancel_token,
            &in_memory_dirs,
        )
        .unwrap();
        assert!(tree_without_empty.is_empty());
    }
}
