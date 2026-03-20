//! Pure logic functions extracted from shell.rs.
//!
//! Utility functions that do not depend on egui. Separated for testability.

use std::path::Path;

/// Offset basis value for FNV-1a hash.
const FNV1A_OFFSET_BASIS: u64 = 0xcbf29ce484222325;

/// Prime value for FNV-1a hash.
const FNV1A_PRIME: u64 = 0x100000001b3;

/// Converts a string to u64 using FNV-1a hash.
pub fn hash_str(s: &str) -> u64 {
    let mut h: u64 = FNV1A_OFFSET_BASIS;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(FNV1A_PRIME);
    }
    h
}

/// Returns the relative full path from the workspace root (for tooltips).
/// Example: /workspace/specs/auth/spec.md → "specs/auth/spec.md"
pub fn relative_full_path(path: &Path, ws_root: Option<&Path>) -> String {
    let rel = match ws_root {
        Some(root) => path.strip_prefix(root).unwrap_or(path),
        None => path,
    };
    rel.to_string_lossy().to_string()
}

/// Returns the new index when navigating tabs forward (left).
/// Wraparound support: moving left from index 0 goes to the last tab.
pub fn prev_tab_index(current: usize, count: usize) -> usize {
    if count == 0 {
        return 0;
    }
    if current == 0 {
        count - 1
    } else {
        current - 1
    }
}

/// Returns the new index when navigating tabs backward (right).
/// Wraparound support: moving right from the last tab goes to the first tab.
pub fn next_tab_index(current: usize, count: usize) -> usize {
    if count == 0 {
        return 0;
    }
    (current + 1) % count
}

/// Formats the metadata tooltip string (Size and Modified time).
pub fn format_metadata_tooltip(
    size: u64,
    sys_time: Option<std::time::SystemTime>,
    template: &str,
) -> String {
    let mod_time_str = sys_time
        .map(|st| {
            let dt: chrono::DateTime<chrono::Local> = st.into();
            dt.format("%Y-%m-%d %H:%M:%S").to_string()
        })
        .unwrap_or_else(|| "Unknown".to_string());

    #[allow(clippy::useless_vec)]
    crate::i18n::tf(
        template,
        &vec![
            ("size", size.to_string().as_str()),
            ("mod_time", mod_time_str.as_str()),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn hash_str_deterministic() {
        assert_eq!(hash_str("hello"), hash_str("hello"));
        assert_ne!(hash_str("hello"), hash_str("world"));
    }

    #[test]
    fn relative_full_path_with_workspace_root() {
        let path = Path::new("/workspace/specs/auth/spec.md");
        let root = Path::new("/workspace");
        let result = relative_full_path(path, Some(root));
        assert_eq!(result, "specs/auth/spec.md");
    }

    #[test]
    fn relative_full_path_without_workspace_root() {
        let path = Path::new("/workspace/specs/auth/spec.md");
        let result = relative_full_path(path, None);
        assert_eq!(result, "/workspace/specs/auth/spec.md");
    }

    #[test]
    fn prev_tab_index_zero_count_returns_zero() {
        assert_eq!(prev_tab_index(0, 0), 0);
    }

    #[test]
    fn prev_tab_index_wraps_from_first_to_last() {
        assert_eq!(prev_tab_index(0, 3), 2);
    }

    #[test]
    fn prev_tab_index_normal_decrement() {
        assert_eq!(prev_tab_index(2, 3), 1);
    }

    #[test]
    fn next_tab_index_zero_count_returns_zero() {
        assert_eq!(next_tab_index(0, 0), 0);
    }

    #[test]
    fn next_tab_index_wraps_from_last_to_first() {
        assert_eq!(next_tab_index(2, 3), 0);
    }

    #[test]
    fn next_tab_index_normal_increment() {
        assert_eq!(next_tab_index(0, 3), 1);
    }

    #[test]
    fn format_metadata_tooltip_populates_template() {
        // TDD RED phase test
        let size = 1024;
        let sys_time = Some(std::time::UNIX_EPOCH);
        let template = "Size: {size}\nMod: {mod_time}";

        let result = format_metadata_tooltip(size, sys_time, template);

        // It should replace \{size\} with 1024
        assert!(result.contains("1024"));
        // It should contain 'Size: 1024\nMod: '
        assert!(result.starts_with("Size: 1024"));
        // Since we are formatting UNIX_EPOCH in local time, exact string match of the year (1970 or 1969 depending on timezone) is enough to check replacement without being flaky.
        assert!(result.contains("1970") || result.contains("1969"));
    }

    #[test]
    fn test_format_tree_tooltip_contains_absolute_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");
        std::fs::write(&file_path, "test").unwrap();

        // Use buggy format_tree_tooltip behavior internally replicating what we had
        let tooltip = format_tree_tooltip("test_file.txt", &file_path);
        let abs_path = std::fs::canonicalize(&file_path).unwrap();
        assert!(
            tooltip.contains(&abs_path.display().to_string()),
            "Tooltip must contain the absolute path.\nGot tooltip:\n{}",
            tooltip
        );
    }
}

/// Helper function to format the full tooltip for a file tree entry.
/// This currently embodies the buggy behavior where `Path` is swallowed.
pub fn format_tree_tooltip(name: &str, path: &std::path::Path) -> String {
    let absolute_path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let path_str = absolute_path.display().to_string();
    format!(
        "{name}\n{}: {path_str}\n{}",
        crate::i18n::tf("Path", &[]),
        if let Ok(metadata) = std::fs::metadata(path) {
            format_metadata_tooltip(
                metadata.len(),
                metadata.modified().ok(),
                &crate::i18n::get().workspace.metadata_tooltip,
            )
        } else {
            "Metadata unavailable".to_string()
        }
    )
}
