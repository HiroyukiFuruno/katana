use katana_core::workspace::*;
use std::path::PathBuf;

#[test]
fn workspace_name_is_root_basename() {
    let ws = Workspace::new("/home/user/myproject", vec![]);
    assert_eq!(ws.name(), Some("myproject"));
}

#[test]
fn tree_entry_identifies_markdown_files() {
    let md = TreeEntry::File {
        path: PathBuf::from("docs/spec.md"),
    };
    let rs = TreeEntry::File {
        path: PathBuf::from("src/main.rs"),
    };
    assert!(md.is_markdown());
    assert!(!rs.is_markdown());
}

#[test]
fn directory_entry_is_not_markdown() {
    let dir = TreeEntry::Directory {
        path: PathBuf::from("docs"),
        children: vec![],
    };
    assert!(!dir.is_markdown());
    assert!(!dir.is_file());
}
