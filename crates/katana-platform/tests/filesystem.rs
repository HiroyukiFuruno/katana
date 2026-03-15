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

// L64-70: hidden files/dirs are excluded from workspace tree
#[test]
fn hidden_files_are_excluded_from_workspace() {
    let tmp = TempDir::new().unwrap();
    // Regular md file
    fs::write(tmp.path().join("visible.md"), "# Visible").unwrap();
    // Hidden file (starts with '.')
    fs::write(tmp.path().join(".hidden.md"), "# Hidden").unwrap();
    // Hidden directory
    fs::create_dir(tmp.path().join(".git")).unwrap();
    fs::write(tmp.path().join(".git").join("README.md"), "# Not shown").unwrap();

    let svc = FilesystemService::new();
    let ws = svc.open_workspace(tmp.path()).unwrap();

    let paths: Vec<_> = ws
        .tree
        .iter()
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();
    assert!(paths.iter().any(|p| p.contains("visible.md")));
    assert!(!paths.iter().any(|p| p.contains(".hidden.md")));
    assert!(!paths.iter().any(|p| p.contains(".git")));
}

// L66: "target" directory is excluded
#[test]
fn target_directory_is_excluded_from_workspace() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("docs.md"), "# Docs").unwrap();
    // Create target/ dir with a .md file inside
    fs::create_dir(tmp.path().join("target")).unwrap();
    fs::write(tmp.path().join("target").join("build.md"), "# Build output").unwrap();

    let svc = FilesystemService::new();
    let ws = svc.open_workspace(tmp.path()).unwrap();

    let paths: Vec<_> = ws
        .tree
        .iter()
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();
    assert!(!paths.iter().any(|p| p.contains("target")));
}

// L67: "node_modules" directory is excluded
#[test]
fn node_modules_directory_is_excluded_from_workspace() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("index.md"), "# Index").unwrap();
    fs::create_dir(tmp.path().join("node_modules")).unwrap();
    fs::write(tmp.path().join("node_modules").join("pkg.md"), "# Package").unwrap();

    let svc = FilesystemService::new();
    let ws = svc.open_workspace(tmp.path()).unwrap();

    let paths: Vec<_> = ws
        .tree
        .iter()
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();
    assert!(!paths.iter().any(|p| p.contains("node_modules")));
}

// L75-79: directories without any markdown are excluded
#[test]
fn directories_without_markdown_are_excluded() {
    let tmp = TempDir::new().unwrap();
    // A md file at root so workspace is valid
    fs::write(tmp.path().join("root.md"), "# Root").unwrap();
    // A subdirectory with only non-md files
    fs::create_dir(tmp.path().join("assets")).unwrap();
    fs::write(tmp.path().join("assets").join("image.png"), b"PNG data").unwrap();
    fs::write(tmp.path().join("assets").join("style.css"), "body{}").unwrap();

    let svc = FilesystemService::new();
    let ws = svc.open_workspace(tmp.path()).unwrap();

    let paths: Vec<_> = ws
        .tree
        .iter()
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();
    // 'assets' dir has no md files so should be excluded
    assert!(!paths.iter().any(|p| p.contains("assets")));
}

// L80-88: non-markdown files at root level are excluded
#[test]
fn non_markdown_files_at_root_are_excluded() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("README.md"), "# Read").unwrap();
    fs::write(tmp.path().join("config.toml"), "[config]").unwrap();
    fs::write(tmp.path().join("script.sh"), "#!/bin/bash").unwrap();

    let svc = FilesystemService::new();
    let ws = svc.open_workspace(tmp.path()).unwrap();

    let paths: Vec<_> = ws
        .tree
        .iter()
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();
    assert!(!paths.iter().any(|p| p.contains("config.toml")));
    assert!(!paths.iter().any(|p| p.contains("script.sh")));
    assert!(paths.iter().any(|p| p.contains("README.md")));
}

// L106: Recursion for has_any_markdown (detecting md in nested subdirectories)
#[test]
fn nested_subdirectory_with_markdown_is_included() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("root.md"), "# Root").unwrap();
    // .md file in a subdirectory 2 levels deep
    fs::create_dir_all(tmp.path().join("docs").join("deep")).unwrap();
    fs::write(
        tmp.path().join("docs").join("deep").join("spec.md"),
        "# Spec",
    )
    .unwrap();

    let svc = FilesystemService::new();
    let ws = svc.open_workspace(tmp.path()).unwrap();

    // "docs" directory is included (because it contains a .md file inside)
    fn find_dir<'a>(tree: &'a [katana_core::workspace::TreeEntry], name: &str) -> bool {
        tree.iter().any(|e| match e {
            katana_core::workspace::TreeEntry::Directory { path, children } => {
                path.file_name().and_then(|n| n.to_str()) == Some(name) || find_dir(children, name)
            }
            _ => false,
        })
    }
    assert!(find_dir(&ws.tree, "docs"));
    assert!(find_dir(&ws.tree, "deep"));
}

// L109-112: FilesystemService::default()
#[test]
fn filesystem_service_default_works() {
    let svc: FilesystemService = Default::default();
    // Default::default() and new() are the same
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("note.md"), "# Note").unwrap();
    let ws = svc.open_workspace(tmp.path()).unwrap();
    assert!(!ws.tree.is_empty());
}
