use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

/// Recursively collects all `.rs` files starting from the root directory.
/// Excludes paths containing `tests`.
pub fn collect_rs_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let walker = WalkBuilder::new(root)
        .standard_filters(true)
        .require_git(false)
        .build();

    for entry in walker.flatten() {
        let path = entry.path();
        if path.is_file()
            && path.extension().is_some_and(|ext| ext == "rs")
            && !path.components().any(|c| c.as_os_str() == "tests")
        {
            files.push(path.to_path_buf());
        }
    }

    files.sort();
    files
}

/// Gets the static workspace root directory of the application.
///
/// # Errors
/// Returns an error string if the root cannot be resolved from the manifest directory.
pub fn workspace_root() -> Result<&'static Path, String> {
    use std::sync::OnceLock;
    static ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();
    let root = ROOT.get_or_init(|| {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|it| it.parent())
            .map(|it| it.to_path_buf())
    });

    match root {
        Some(path) => Ok(path.as_path()),
        None => Err("Workspace root not found".to_string()),
    }
}
