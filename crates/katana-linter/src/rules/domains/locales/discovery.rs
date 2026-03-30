use std::path::{Path, PathBuf};

pub fn collect_locale_json_files(locale_dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(locale_dir)
        .expect("Locale directory should be readable")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().is_some_and(|ext| ext == "json")
                && path
                    .file_name()
                    .is_some_and(|name| name != "languages.json")
        })
        .collect();
    files.sort();
    files
}

pub fn locale_code_from_path(path: &Path) -> Option<String> {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
}
