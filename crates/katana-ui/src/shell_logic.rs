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
}
