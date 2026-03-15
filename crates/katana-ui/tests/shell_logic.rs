use katana_ui::shell_logic::{hash_str, next_tab_index, prev_tab_index, relative_full_path};
use std::path::Path;

// ── hash_str テスト ──

#[test]
fn 空文字列のハッシュが一貫している() {
    let h1 = hash_str("");
    let h2 = hash_str("");
    assert_eq!(h1, h2, "同一入力は同じハッシュを返すべき");
}

#[test]
fn ascii文字列のハッシュが決定的() {
    let h = hash_str("hello");
    assert_ne!(h, 0, "ハッシュ値はゼロでないべき");
    assert_eq!(h, hash_str("hello"), "同一入力の一貫性チェック");
}

#[test]
fn 異なる文字列は異なるハッシュを返す() {
    assert_ne!(hash_str("hello"), hash_str("world"));
    assert_ne!(hash_str("a"), hash_str("b"));
}

#[test]
fn 日本語文字列のハッシュが一貫している() {
    let h1 = hash_str("こんにちは");
    let h2 = hash_str("こんにちは");
    assert_eq!(h1, h2);
    assert_ne!(h1, hash_str("さようなら"));
}

#[test]
fn 同一入力は常に同じハッシュ値を返す() {
    let input = "test-consistency-check";
    let results: Vec<u64> = (0..100).map(|_| hash_str(input)).collect();
    assert!(results.iter().all(|&h| h == results[0]));
}

// ── relative_full_path テスト ──

#[test]
fn ルートありの場合相対パスを返す() {
    let path = Path::new("/workspace/specs/auth/spec.md");
    let root = Path::new("/workspace");
    assert_eq!(relative_full_path(path, Some(root)), "specs/auth/spec.md");
}

#[test]
fn ルートなしの場合フルパスを返す() {
    let path = Path::new("/home/user/doc.md");
    assert_eq!(relative_full_path(path, None), "/home/user/doc.md");
}

#[test]
fn ルート外パスの場合フルパスを返す() {
    let path = Path::new("/other/project/file.md");
    let root = Path::new("/workspace");
    assert_eq!(
        relative_full_path(path, Some(root)),
        "/other/project/file.md"
    );
}

#[test]
fn ルートとパスが同一の場合空文字列を返す() {
    let path = Path::new("/workspace");
    let root = Path::new("/workspace");
    assert_eq!(relative_full_path(path, Some(root)), "");
}

// ── prev_tab_index テスト ──

#[test]
fn prev_tab_index_中間から前に移動() {
    assert_eq!(prev_tab_index(2, 5), 1);
}

#[test]
fn prev_tab_index_先頭からラップアラウンド() {
    assert_eq!(prev_tab_index(0, 5), 4);
}

#[test]
fn prev_tab_index_タブ1つの場合() {
    assert_eq!(prev_tab_index(0, 1), 0);
}

#[test]
fn prev_tab_index_空リストの場合() {
    assert_eq!(prev_tab_index(0, 0), 0);
}

// ── next_tab_index テスト ──

#[test]
fn next_tab_index_中間から次に移動() {
    assert_eq!(next_tab_index(2, 5), 3);
}

#[test]
fn next_tab_index_末尾からラップアラウンド() {
    assert_eq!(next_tab_index(4, 5), 0);
}

#[test]
fn next_tab_index_タブ1つの場合() {
    assert_eq!(next_tab_index(0, 1), 0);
}

#[test]
fn next_tab_index_空リストの場合() {
    assert_eq!(next_tab_index(0, 0), 0);
}
