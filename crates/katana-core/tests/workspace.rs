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

// L20: TreeEntry::path() for Directory
// L24-26: TreeEntry::name()
#[test]
fn tree_entry_path_and_name_for_directory() {
    let path = PathBuf::from("docs/chapter1");
    let entry = TreeEntry::Directory {
        path: path.clone(),
        children: vec![],
    };
    assert_eq!(entry.path(), path.as_path());
    assert_eq!(entry.name(), Some("chapter1"));
}

// L20: TreeEntry::path() for File
#[test]
fn tree_entry_path_for_file() {
    let path = PathBuf::from("docs/spec.md");
    let entry = TreeEntry::File { path: path.clone() };
    assert_eq!(entry.path(), path.as_path());
}

// L63: WorkspaceError::unreadable_root format
#[test]
fn workspace_error_unreadable_root_message() {
    let err = WorkspaceError::unreadable_root(
        PathBuf::from("/some/path"),
        std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
    );
    let msg = err.to_string();
    // Message should contain the path
    assert!(msg.contains("/some/path"));
}

// WorkspaceError::NoWorkspace
#[test]
fn workspace_error_no_workspace_message() {
    let err = WorkspaceError::NoWorkspace;
    let msg = err.to_string();
    assert!(msg.contains("workspace") || msg.contains("Workspace"));
}

// Workspace name returns None for "/" path (filesystem root has no basename)
#[test]
fn workspace_name_none_for_filesystem_root() {
    let ws = Workspace::new("/", vec![]);
    assert_eq!(ws.name(), None);
}

// is_markdown: file without extension is false (false path of unwrap_or(false))
#[test]
fn is_markdown_returns_false_for_no_extension() {
    let entry = TreeEntry::File {
        path: PathBuf::from("Makefile"),
    };
    assert!(!entry.is_markdown());
}
