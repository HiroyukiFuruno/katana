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

        let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
            return;
        };

        let ext_lower = ext.to_lowercase();
        if ext_lower != "ttf" && ext_lower != "ttc" && ext_lower != "otf" {
            return; // 対象外の拡張子はスキップ
        }

        let Some(name) = path.file_stem().and_then(|s| s.to_str()) else {
            return;
        };

        let path_str = path.to_string_lossy().to_string();
        fonts.push((name.to_string(), path_str));
    }
}
