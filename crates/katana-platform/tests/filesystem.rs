use katana_core::workspace::TreeEntry;
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
    let ignored = katana_platform::settings::DEFAULT_IGNORED_DIRECTORIES
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let ws = svc
        .open_workspace(
            tmp.path(),
            &ignored,
            10,
            &["md".to_string(), "markdown".to_string(), "mdx".to_string()],
            &[],
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            &std::collections::HashSet::new(),
        )
        .unwrap();
    assert_eq!(ws.root, tmp.path());
    assert!(!ws.tree.is_empty());
}

#[test]
fn open_invalid_workspace_returns_error() {
    let svc = FilesystemService::new();
    let result = svc.open_workspace(
        "/nonexistent/path/that/does/not/exist",
        &[],
        10,
        &["md".to_string()],
        &[],
        std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        &std::collections::HashSet::new(),
    );
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
    assert_eq!(fs::read_to_string(&path).unwrap(), "original");
}

#[test]
fn load_nonexistent_document_returns_error() {
    let svc = FilesystemService::new();
    let result = svc.load_document("/this/does/not/exist.md");
    assert!(result.is_err());
}

#[test]
fn hidden_directories_with_markdown_are_included() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("visible.md"), "# Visible").unwrap();
    fs::write(tmp.path().join(".hidden.md"), "# Hidden").unwrap();
    fs::create_dir(tmp.path().join(".config")).unwrap();
    fs::write(tmp.path().join(".config").join("notes.md"), "# Notes").unwrap();
    fs::create_dir(tmp.path().join(".cache")).unwrap();
    fs::write(tmp.path().join(".cache").join("data.bin"), b"binary").unwrap();

    let svc = FilesystemService::new();
    let ignored = katana_platform::settings::DEFAULT_IGNORED_DIRECTORIES
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let ws = svc
        .open_workspace(
            tmp.path(),
            &ignored,
            10,
            &["md".to_string(), "markdown".to_string(), "mdx".to_string()],
            &[],
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            &std::collections::HashSet::new(),
        )
        .unwrap();

    let all_paths = collect_all_paths(&ws.tree);
    assert!(all_paths.iter().any(|p| p.contains("visible.md")));
    assert!(all_paths.iter().any(|p| p.contains(".hidden.md")));
    assert!(all_paths.iter().any(|p| p.contains(".config")));
    assert!(!all_paths.iter().any(|p| p.contains(".cache")));
}

fn collect_all_paths(tree: &[katana_core::workspace::TreeEntry]) -> Vec<String> {
    let mut paths = Vec::new();
    for entry in tree {
        paths.push(entry.path().to_string_lossy().to_string());
        if let katana_core::workspace::TreeEntry::Directory { children, .. } = entry {
            paths.extend(collect_all_paths(children));
        }
    }
    paths
}

#[test]
fn target_directory_is_excluded_from_workspace() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("docs.md"), "# Docs").unwrap();
    fs::create_dir(tmp.path().join("target")).unwrap();
    fs::write(tmp.path().join("target").join("build.md"), "# Build output").unwrap();

    let svc = FilesystemService::new();
    let ignored = katana_platform::settings::DEFAULT_IGNORED_DIRECTORIES
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let ws = svc
        .open_workspace(
            tmp.path(),
            &ignored,
            10,
            &["md".to_string(), "markdown".to_string(), "mdx".to_string()],
            &[],
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            &std::collections::HashSet::new(),
        )
        .unwrap();

    let paths: Vec<_> = ws
        .tree
        .iter()
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();
    assert!(!paths.iter().any(|p| p.contains("target")));
}

#[test]
fn node_modules_directory_is_excluded_from_workspace() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("index.md"), "# Index").unwrap();
    fs::create_dir(tmp.path().join("node_modules")).unwrap();
    fs::write(tmp.path().join("node_modules").join("pkg.md"), "# Package").unwrap();

    let svc = FilesystemService::new();
    let ignored = katana_platform::settings::DEFAULT_IGNORED_DIRECTORIES
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let ws = svc
        .open_workspace(
            tmp.path(),
            &ignored,
            10,
            &["md".to_string(), "markdown".to_string(), "mdx".to_string()],
            &[],
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            &std::collections::HashSet::new(),
        )
        .unwrap();

    let paths: Vec<_> = ws
        .tree
        .iter()
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();
    assert!(!paths.iter().any(|p| p.contains("node_modules")));
}

#[test]
fn directories_without_markdown_are_excluded() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("root.md"), "# Root").unwrap();
    fs::create_dir(tmp.path().join("assets")).unwrap();
    fs::write(tmp.path().join("assets").join("image.png"), b"PNG data").unwrap();
    fs::write(tmp.path().join("assets").join("style.css"), "body{}").unwrap();

    let svc = FilesystemService::new();
    let ignored = katana_platform::settings::DEFAULT_IGNORED_DIRECTORIES
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let ws = svc
        .open_workspace(
            tmp.path(),
            &ignored,
            10,
            &["md".to_string(), "markdown".to_string(), "mdx".to_string()],
            &[],
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            &std::collections::HashSet::new(),
        )
        .unwrap();

    let paths: Vec<_> = ws
        .tree
        .iter()
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();
    assert!(!paths.iter().any(|p| p.contains("assets")));
}

#[test]
fn non_markdown_files_at_root_are_excluded() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("README.md"), "# Read").unwrap();
    fs::write(tmp.path().join("config.toml"), "[config]").unwrap();
    fs::write(tmp.path().join("script.sh"), "#!/bin/bash").unwrap();

    let svc = FilesystemService::new();
    let ignored = katana_platform::settings::DEFAULT_IGNORED_DIRECTORIES
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let ws = svc
        .open_workspace(
            tmp.path(),
            &ignored,
            10,
            &["md".to_string(), "markdown".to_string(), "mdx".to_string()],
            &[],
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            &std::collections::HashSet::new(),
        )
        .unwrap();

    let paths: Vec<_> = ws
        .tree
        .iter()
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();
    assert!(!paths.iter().any(|p| p.contains("config.toml")));
    assert!(!paths.iter().any(|p| p.contains("script.sh")));
    assert!(paths.iter().any(|p| p.contains("README.md")));
}

#[test]
fn nested_subdirectory_with_markdown_is_included() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("root.md"), "# Root").unwrap();
    fs::create_dir_all(tmp.path().join("docs").join("deep")).unwrap();
    fs::write(
        tmp.path().join("docs").join("deep").join("spec.md"),
        "# Spec",
    )
    .unwrap();

    let svc = FilesystemService::new();
    let ignored = katana_platform::settings::DEFAULT_IGNORED_DIRECTORIES
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let ws = svc
        .open_workspace(
            tmp.path(),
            &ignored,
            10,
            &["md".to_string(), "markdown".to_string(), "mdx".to_string()],
            &[],
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            &std::collections::HashSet::new(),
        )
        .unwrap();

    fn find_dir(tree: &[katana_core::workspace::TreeEntry], name: &str) -> bool {
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

#[test]
fn filesystem_service_default_works() {
    let svc: FilesystemService = Default::default();
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("note.md"), "# Note").unwrap();
    let ignored = katana_platform::settings::DEFAULT_IGNORED_DIRECTORIES
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let ws = svc
        .open_workspace(
            tmp.path(),
            &ignored,
            10,
            &["md".to_string(), "markdown".to_string(), "mdx".to_string()],
            &[],
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            &std::collections::HashSet::new(),
        )
        .unwrap();
    assert!(!ws.tree.is_empty());
}

#[test]
fn test_scan_directory_respects_max_depth() {
    let svc = FilesystemService::new();
    let tmp = tempfile::TempDir::new().unwrap();
    let dir1 = tmp.path().join("dir1");
    fs::create_dir(&dir1).unwrap();
    let dir2 = dir1.join("dir2");
    fs::create_dir(&dir2).unwrap();
    fs::write(dir1.join("file1.md"), "# File 1").unwrap();
    fs::write(dir2.join("file2.md"), "# File 2").unwrap();

    let ignored = vec![];
    let cancel_token = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

    let ws = svc
        .open_workspace(
            tmp.path(),
            &ignored,
            2,
            &["md".to_string(), "markdown".to_string(), "mdx".to_string()],
            &[],
            cancel_token.clone(),
            &std::collections::HashSet::new(),
        )
        .unwrap();

    fn find_dir(entries: &[TreeEntry], name: &str) -> bool {
        entries.iter().any(|e| match e {
            TreeEntry::Directory { path, children } => {
                path.file_name().and_then(|n| n.to_str()) == Some(name) || find_dir(children, name)
            }
            _ => false,
        })
    }
    assert!(find_dir(&ws.tree, "dir1"));
    assert!(!find_dir(&ws.tree, "dir2"));
}