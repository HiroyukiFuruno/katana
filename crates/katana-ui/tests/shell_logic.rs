use katana_ui::shell_logic::{
    format_tree_tooltip, hash_str, next_tab_index, prev_tab_index, relative_full_path,
};
use std::path::Path;


#[test]
fn empty_string_hash_is_consistent() {
    let h1 = hash_str("");
    let h2 = hash_str("");
    assert_eq!(h1, h2, "Identical inputs should return the same hash");
}

#[test]
fn ascii_string_hash_is_deterministic() {
    let h = hash_str("hello");
    assert_ne!(h, 0, "Hash value should not be zero");
    assert_eq!(
        h,
        hash_str("hello"),
        "Consistency check for identical input"
    );
}

#[test]
fn different_strings_return_different_hashes() {
    assert_ne!(hash_str("hello"), hash_str("world"));
    assert_ne!(hash_str("a"), hash_str("b"));
}

#[test]
fn non_ascii_string_hash_is_consistent() {
    let h1 = hash_str("hello world");
    let h2 = hash_str("hello world");
    assert_eq!(h1, h2);
    assert_ne!(h1, hash_str("goodbye"));
}

#[test]
fn identical_inputs_always_return_same_hash_value() {
    let input = "test-consistency-check";
    let results: Vec<u64> = (0..100).map(|_| hash_str(input)).collect();
    assert!(results.iter().all(|&h| h == results[0]));
}


#[test]
fn returns_relative_path_if_root_is_provided() {
    let path = Path::new("/workspace/specs/auth/spec.md");
    let root = Path::new("/workspace");
    assert_eq!(relative_full_path(path, Some(root)), "specs/auth/spec.md");
}

#[test]
fn returns_full_path_if_no_root_provided() {
    let path = Path::new("/home/user/doc.md");
    assert_eq!(relative_full_path(path, None), "/home/user/doc.md");
}

#[test]
fn returns_full_path_if_path_is_outside_root() {
    let path = Path::new("/other/project/file.md");
    let root = Path::new("/workspace");
    assert_eq!(
        relative_full_path(path, Some(root)),
        "/other/project/file.md"
    );
}

#[test]
fn returns_empty_string_if_path_and_root_are_identical() {
    let path = Path::new("/workspace");
    let root = Path::new("/workspace");
    assert_eq!(relative_full_path(path, Some(root)), "");
}


#[test]
fn prev_tab_index_moves_backward_from_middle() {
    assert_eq!(prev_tab_index(2, 5), 1);
}

#[test]
fn prev_tab_index_wraps_around_from_start() {
    assert_eq!(prev_tab_index(0, 5), 4);
}

#[test]
fn prev_tab_index_with_single_tab() {
    assert_eq!(prev_tab_index(0, 1), 0);
}

#[test]
fn prev_tab_index_with_empty_list() {
    assert_eq!(prev_tab_index(0, 0), 0);
}


#[test]
fn next_tab_index_moves_forward_from_middle() {
    assert_eq!(next_tab_index(2, 5), 3);
}

#[test]
fn next_tab_index_wraps_around_from_end() {
    assert_eq!(next_tab_index(4, 5), 0);
}

#[test]
fn next_tab_index_with_single_tab() {
    assert_eq!(next_tab_index(0, 1), 0);
}

#[test]
fn next_tab_index_with_empty_list() {
    assert_eq!(next_tab_index(0, 0), 0);
}


#[test]
fn test_format_tree_tooltip_metadata_unavailable() {
    let path = Path::new("/non/existent/path/for/test/metadata_unavailable.md");
    let tooltip = format_tree_tooltip("metadata_unavailable.md", path);
    assert!(tooltip.contains("Metadata unavailable"));
}