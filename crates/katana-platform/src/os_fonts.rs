use std::fs;
use std::path::Path;
use std::sync::OnceLock;

/// Service for discovering installed macOS fonts.
pub struct OsFontScanner;

static SYSTEM_FONTS: OnceLock<Vec<(String, String)>> = OnceLock::new();

impl OsFontScanner {
    /// Returns a cached list of system fonts to avoid filesystem IO on every frame.
    pub fn cached_fonts() -> &'static [(String, String)] {
        SYSTEM_FONTS.get_or_init(Self::scan_fonts).as_slice()
    }

    /// Scans standard macOS font directories for TTF, TTC, and OTF files.
    ///
    /// Returns a list of `(font_name, file_path)`.
    pub fn scan_fonts() -> Vec<(String, String)> {
        let mut fonts = Vec::new();

        // ユーザ定義パスや環境変数などがない場合はデフォルトを設定（ここではOS依存）
        let home_dir = std::env::var("HOME").unwrap_or_default();
        let user_fonts = format!("{home_dir}/Library/Fonts");

        let mut all_dirs = vec![
            "/System/Library/Fonts".to_string(),
            "/System/Library/Fonts/Supplemental".to_string(),
            "/Library/Fonts".to_string(),
        ];
        if !home_dir.is_empty() {
            all_dirs.push(user_fonts);
        }

        for dir in all_dirs {
            Self::scan_directory(Path::new(&dir), &mut fonts);
        }

        fonts.sort_by(|a, b| a.0.cmp(&b.0));
        fonts.dedup_by(|a, b| a.0 == b.0);
        fonts
    }

    /// Recursively unnested scan to adhere to maximum nest rules (2 levels focus).
    fn scan_directory(dir: &Path, fonts: &mut Vec<(String, String)>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return; // ディレクトリが存在しない、アクセス権限がない場合はスキップ
        };

        for entry in entries.flatten() {
            Self::process_entry(&entry.path(), fonts);
        }
    }

    /// Extracted per-entry logic to ensure max 30 lines and 2 block nest rule.
    fn process_entry(path: &Path, fonts: &mut Vec<(String, String)>) {
        if !path.is_file() {
            return;
        }

        let ext = path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        if ext != "ttf" && ext != "ttc" && ext != "otf" {
            return; // 対象外の拡張子はスキップ
        }

        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let path_str = path.to_string_lossy().to_string();
        fonts.push((name, path_str));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_scan_directory_ignores_missing_dir() {
        let mut fonts = vec![];
        let path = Path::new("/path/that/does/not/exist");
        OsFontScanner::scan_directory(path, &mut fonts);
        assert!(fonts.is_empty());
    }

    #[test]
    fn test_process_entry_skips_invalid_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mut fonts = vec![];

        // 1. Directory is skipped
        let dir_path = tmp.path().join("dir.ttf");
        fs::create_dir(&dir_path).unwrap();
        OsFontScanner::process_entry(&dir_path, &mut fonts);
        assert!(fonts.is_empty());

        // 2. File without extension is skipped
        let no_ext_path = tmp.path().join("no_extension");
        fs::write(&no_ext_path, "").unwrap();
        OsFontScanner::process_entry(&no_ext_path, &mut fonts);
        assert!(fonts.is_empty());

        // 3. File with invalid extension is skipped
        let invalid_ext_path = tmp.path().join("invalid.txt");
        fs::write(&invalid_ext_path, "").unwrap();
        OsFontScanner::process_entry(&invalid_ext_path, &mut fonts);
        assert!(fonts.is_empty());

        // 4. File with valid extension is added
        let valid_path = tmp.path().join("MyFont.ttf");
        fs::write(&valid_path, "").unwrap();
        OsFontScanner::process_entry(&valid_path, &mut fonts);
        assert_eq!(fonts.len(), 1);
        assert_eq!(fonts[0].0, "MyFont");
    }
}
