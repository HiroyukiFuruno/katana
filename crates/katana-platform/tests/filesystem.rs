use katana_platform::FilesystemService;
use std::fs;
use tempfile::TempDir;

fn setup_workspace() -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("README.md"), "# Workspace").unwrap();
    fs::write(dir.path().join("spec.md"), "## Spec").unwrap();
    fs::create_dir(dir.path().join("docs")).unwrap();
    fs::write(dir.path().join("docs").join("arch.md"), "## Architecture").unwrap();
    dir
}

#[test]
fn open_valid_workspace_returns_workspace() {
    let tmp = setup_workspace();
    let svc = FilesystemService::new();
    let ws = svc.open_workspace(tmp.path()).unwrap();
    assert_eq!(ws.root, tmp.path());
    assert!(!ws.tree.is_empty());
}

#[test]
fn open_invalid_workspace_returns_error() {
    let svc = FilesystemService::new();
    let result = svc.open_workspace("/nonexistent/path/that/does/not/exist");
    assert!(result.is_err());
}

#[test]
fn load_document_reads_content() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("test.md");
    fs::write(&path, "# Hello World").unwrap();
    let svc = FilesystemService::new();
    let doc = svc.load_document(&path).unwrap();
    assert_eq!(doc.buffer, "# Hello World");
    assert!(!doc.is_dirty);
}

#[test]
fn save_document_writes_buffer_and_marks_clean() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("doc.md");
    fs::write(&path, "original").unwrap();
    let svc = FilesystemService::new();
    let mut doc = svc.load_document(&path).unwrap();
    doc.update_buffer("# Updated Content");
    assert!(doc.is_dirty);
    svc.save_document(&mut doc).unwrap();
    assert!(!doc.is_dirty);
    assert_eq!(fs::read_to_string(&path).unwrap(), "# Updated Content");
}

#[test]
fn edit_without_explicit_save_does_not_change_disk() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("doc.md");
    fs::write(&path, "original").unwrap();
    let svc = FilesystemService::new();
    let mut doc = svc.load_document(&path).unwrap();
    doc.update_buffer("edited but not saved");
    // Disk unchanged.
    assert_eq!(fs::read_to_string(&path).unwrap(), "original");
}

#[test]
fn load_nonexistent_document_returns_error() {
    let svc = FilesystemService::new();
    let result = svc.load_document("/this/does/not/exist.md");
    assert!(result.is_err());
}
