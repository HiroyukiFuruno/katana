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

pub const SPLASH_VISIBLE_DURATION: f32 = 1.5;
pub const SPLASH_FADE_DURATION: f32 = 0.5;

/// Calculates the splash screen opacity based on elapsed time.
/// Returns a value between 0.0 and 1.0.
pub fn calculate_splash_opacity(elapsed_secs: f32) -> f32 {
    if elapsed_secs <= SPLASH_VISIBLE_DURATION {
        1.0
    } else if elapsed_secs <= SPLASH_VISIBLE_DURATION + SPLASH_FADE_DURATION {
        1.0 - ((elapsed_secs - SPLASH_VISIBLE_DURATION) / SPLASH_FADE_DURATION)
    } else {
        0.0
    }
}

/// Calculates the splash screen progress bar ratio based on elapsed time.
/// Returns a value between 0.0 and 1.0.
pub fn calculate_splash_progress(elapsed_secs: f32) -> f32 {
    (elapsed_secs / SPLASH_VISIBLE_DURATION).clamp(0.0, 1.0)
}

/// Helper function to format the full tooltip for a file tree entry.
/// This currently embodies the buggy behavior where `Path` is swallowed.
pub fn format_tree_tooltip(name: &str, path: &std::path::Path) -> String {
    // std::fs::canonicalize resolves /Users to /System/Volumes/Data/Users on macOS, producing confusing paths.
    // Instead we just display the actual path without canonicalizing.
    let path_str = path.display().to_string();
    format!(
        "{name}\n{}: {path_str}\n{}",
        crate::i18n::get().workspace.path_label,
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

/// Recursively collects up to 100 matching files from the workspace tree.
pub fn collect_matches(
    entries: &[katana_core::workspace::TreeEntry],
    query: &str,
    include_regexes: &[regex::Regex],
    exclude_regexes: &[regex::Regex],
    ws_root: &std::path::Path,
    results: &mut Vec<std::path::PathBuf>,
) {
    if results.len() >= 100 {
        return;
    }
    for entry in entries {
        match entry {
            katana_core::workspace::TreeEntry::File { path } => {
                let rel = relative_full_path(path, Some(ws_root));

                // 1. Exclude check (priority)
                let mut is_excluded = false;
                for re in exclude_regexes {
                    if re.is_match(&rel) {
                        is_excluded = true;
                        break;
                    }
                }
                if is_excluded {
                    continue;
                }

                // 2. Query check
                let mut matches_query = true;
                if !query.is_empty() {
                    matches_query = rel.to_lowercase().contains(query);
                }

                // 3. Include check
                let mut matches_include = true;
                if !include_regexes.is_empty() {
                    matches_include = false;
                    for re in include_regexes {
                        if re.is_match(&rel) {
                            matches_include = true;
                            break;
                        }
                    }
                }

                if matches_query && matches_include {
                    results.push(path.clone());
                    if results.len() >= 100 {
                        return;
                    }
                }
            }
            katana_core::workspace::TreeEntry::Directory { children, .. } => {
                collect_matches(
                    children,
                    query,
                    include_regexes,
                    exclude_regexes,
                    ws_root,
                    results,
                );
            }
        }
    }
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
        let expected_path = file_path.display().to_string();
        assert!(
            tooltip.contains(&expected_path),
            "Tooltip must contain the path.\nGot tooltip:\n{}",
            tooltip
        );
    }

    #[test]
    fn collect_matches_filters_correctly() {
        use katana_core::workspace::TreeEntry;
        use regex::Regex;
        let ws_root = Path::new("/root");

        let entries = vec![
            TreeEntry::File {
                path: ws_root.join("src/main.rs"),
            },
            TreeEntry::File {
                path: ws_root.join("tests/integration.rs"),
            },
            TreeEntry::Directory {
                path: ws_root.join("docs"),
                children: vec![TreeEntry::File {
                    path: ws_root.join("docs/readme.md"),
                }],
            },
        ];

        // 1. Empty query matches all files up to 100
        let mut results = Vec::new();
        collect_matches(&entries, "", &[], &[], ws_root, &mut results);
        assert_eq!(results.len(), 3);

        // 2. Query search
        let mut results = Vec::new();
        collect_matches(&entries, "main", &[], &[], ws_root, &mut results);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], ws_root.join("src/main.rs"));

        // 3. Exclude regex
        let mut results = Vec::new();
        let exclude = vec![Regex::new(r"^tests/").unwrap()];
        collect_matches(&entries, "", &[], &exclude, ws_root, &mut results);
        assert_eq!(results.len(), 2);
        assert!(!results.contains(&ws_root.join("tests/integration.rs")));

        // 4. Include regex
        let mut results = Vec::new();
        let include = vec![Regex::new(r"\.md$").unwrap()];
        collect_matches(&entries, "", &include, &[], ws_root, &mut results);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], ws_root.join("docs/readme.md"));

        // 5. Case insensitive query
        let mut results = Vec::new();
        collect_matches(&entries, "readme", &[], &[], ws_root, &mut results);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn collect_matches_limits_to_100() {
        use katana_core::workspace::TreeEntry;
        let ws_root = std::path::Path::new("/root");

        let mut entries = Vec::new();
        for i in 0..105 {
            entries.push(TreeEntry::File {
                path: ws_root.join(format!("file_{}.txt", i)),
            });
        }

        // Test early return at start of function
        let mut results = vec![ws_root.join("dummy.txt"); 100];
        collect_matches(&entries, "", &[], &[], ws_root, &mut results);
        assert_eq!(results.len(), 100); // Should not add any more

        // Test early return inside loop
        let mut results = Vec::new();
        collect_matches(&entries, "", &[], &[], ws_root, &mut results);
        assert_eq!(results.len(), 100); // Should stop at 100
    }

    #[test]
    fn test_calculate_splash_opacity() {
        // Must stay 1.0 up to 1.5 seconds
        assert_eq!(calculate_splash_opacity(0.0), 1.0);
        assert_eq!(calculate_splash_opacity(1.0), 1.0);
        assert_eq!(calculate_splash_opacity(1.5), 1.0);

        // Fades out between 1.5 and 2.0 seconds
        assert_eq!(calculate_splash_opacity(1.75), 0.5);

        // Clamped at 0.0 after 2.0 seconds
        assert_eq!(calculate_splash_opacity(2.0), 0.0);
        assert_eq!(calculate_splash_opacity(2.5), 0.0);
    }

    #[test]
    fn test_calculate_splash_progress() {
        assert_eq!(calculate_splash_progress(0.0), 0.0);
        assert_eq!(calculate_splash_progress(0.75), 0.5);
        assert_eq!(calculate_splash_progress(1.5), 1.0);
        assert_eq!(calculate_splash_progress(2.0), 1.0);
    }
}
