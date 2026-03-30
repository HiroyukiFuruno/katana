#![allow(clippy::unwrap_used)]
use katana_platform::os_fonts::OsFontScanner;
use std::fs;
use std::path::Path;

#[test]
fn scan_directory_ignores_missing_dir() {
    let mut fonts = vec![];
    let path = Path::new("/path/that/does/not/exist");
    OsFontScanner::scan_directory(path, &mut fonts);
    assert!(fonts.is_empty());
}

#[test]
fn process_entry_skips_invalid_files() {
    let tmp = tempfile::TempDir::new().unwrap();
    let mut fonts = vec![];

    let dir_path = tmp.path().join("dir.ttf");
    fs::create_dir(&dir_path).unwrap();
    OsFontScanner::process_entry(&dir_path, &mut fonts);
    assert!(fonts.is_empty());

    let no_ext_path = tmp.path().join("no_extension");
    fs::write(&no_ext_path, "").unwrap();
    OsFontScanner::process_entry(&no_ext_path, &mut fonts);
    assert!(fonts.is_empty());

    let invalid_ext_path = tmp.path().join("invalid.txt");
    fs::write(&invalid_ext_path, "").unwrap();
    OsFontScanner::process_entry(&invalid_ext_path, &mut fonts);
    assert!(fonts.is_empty());

    let valid_path = tmp.path().join("MyFont.ttf");
    fs::write(&valid_path, "").unwrap();
    OsFontScanner::process_entry(&valid_path, &mut fonts);
    assert_eq!(fonts.len(), 1);
    assert_eq!(fonts[0].0, "MyFont");
}

#[test]
fn process_entry_accepts_ttc_and_otf() {
    let tmp = tempfile::TempDir::new().unwrap();
    let mut fonts = vec![];

    let ttc_path = tmp.path().join("CollFont.ttc");
    fs::write(&ttc_path, "").unwrap();
    OsFontScanner::process_entry(&ttc_path, &mut fonts);
    assert_eq!(fonts.len(), 1);
    assert_eq!(fonts[0].0, "CollFont");

    let otf_path = tmp.path().join("OpenFont.otf");
    fs::write(&otf_path, "").unwrap();
    OsFontScanner::process_entry(&otf_path, &mut fonts);
    assert_eq!(fonts.len(), 2);
    assert_eq!(fonts[1].0, "OpenFont");
}