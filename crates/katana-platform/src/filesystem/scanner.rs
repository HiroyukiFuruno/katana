use katana_core::workspace::TreeEntry;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy)]
pub(crate) struct ScanContext<'a> {
    pub ignored_directories: &'a [String],
    pub max_depth: usize,
    pub visible_extensions: &'a [String],
    pub extensionless_excludes: &'a [String],
    pub cancel_token: &'a std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub in_memory_dirs: &'a std::collections::HashSet<PathBuf>,
}

fn process_file(path: PathBuf, file_name: &str, ctx: ScanContext<'_>) -> Option<TreeEntry> {
    let is_visible = match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => ctx
            .visible_extensions
            .iter()
            .any(|v| v.eq_ignore_ascii_case(ext)),
        None => {
            let no_ext_enabled = ctx.visible_extensions.iter().any(|v| v.is_empty());
            no_ext_enabled
                && !ctx
                    .extensionless_excludes
                    .iter()
                    .any(|excl| excl == file_name)
        }
    };
    if is_visible {
        Some(TreeEntry::File { path })
    } else {
        None
    }
}

fn process_dir(path: PathBuf, current_depth: usize, ctx: ScanContext<'_>) -> Option<TreeEntry> {
    let children = scan_directory_internal(&path, ctx, current_depth + 1).unwrap_or_default();
    if has_any_visible(&children, ctx.visible_extensions) || ctx.in_memory_dirs.contains(&path) {
        Some(TreeEntry::Directory { path, children })
    } else {
        None
    }
}

fn process_entry(
    entry: &std::fs::DirEntry,
    current_depth: usize,
    ctx: ScanContext<'_>,
) -> Option<TreeEntry> {
    if ctx.cancel_token.load(std::sync::atomic::Ordering::Relaxed) {
        return None;
    }
    let path = entry.path();
    let file_name_os = entry.file_name();
    let file_name = file_name_os.to_str()?;
    if ctx
        .ignored_directories
        .iter()
        .any(|ignored| ignored == file_name)
    {
        return None;
    }
    if path.is_dir() {
        process_dir(path, current_depth, ctx)
    } else {
        process_file(path, file_name, ctx)
    }
}

fn scan_directory_internal(
    dir: &Path,
    ctx: ScanContext<'_>,
    current_depth: usize,
) -> std::io::Result<Vec<TreeEntry>> {
    if current_depth >= ctx.max_depth || ctx.cancel_token.load(std::sync::atomic::Ordering::Relaxed)
    {
        return Ok(Vec::new());
    }
    use rayon::prelude::*;
    let iter = std::fs::read_dir(dir)?;
    let child_entries: Vec<_> = iter.filter_map(Result::ok).collect();
    let mut entries: Vec<TreeEntry> = child_entries
        .into_par_iter()
        .filter_map(|entry| process_entry(&entry, current_depth, ctx))
        .collect();
    entries.sort_by(|a, b| match (a, b) {
        (TreeEntry::Directory { .. }, TreeEntry::File { .. }) => std::cmp::Ordering::Less,
        (TreeEntry::File { .. }, TreeEntry::Directory { .. }) => std::cmp::Ordering::Greater,
        (a, b) => a.path().cmp(b.path()),
    });
    Ok(entries)
}

// WHY: Recursively and in parallel scans a directory, returning a tree containing only visible files.
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
    let ctx = ScanContext {
        ignored_directories,
        max_depth,
        visible_extensions,
        extensionless_excludes,
        cancel_token,
        in_memory_dirs,
    };
    scan_directory_internal(dir, ctx, current_depth)
}

// WHY: Recursively checks if there is at least one visible file in the tree.
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
