//! shell.rs から抽出した純粋ロジック関数。
//!
//! egui に依存しないユーティリティ関数群。テスト容易性のために分離。

use std::path::Path;

/// FNV-1a ハッシュのオフセットベース値。
const FNV1A_OFFSET_BASIS: u64 = 0xcbf29ce484222325;

/// FNV-1a ハッシュのプライム値。
const FNV1A_PRIME: u64 = 0x100000001b3;

/// FNV-1a ハッシュで文字列をu64に変換する。
pub fn hash_str(s: &str) -> u64 {
    let mut h: u64 = FNV1A_OFFSET_BASIS;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(FNV1A_PRIME);
    }
    h
}

/// ワークスペースルートからの相対フルパスを返す（ツールチップ用）。
/// 例: /workspace/specs/auth/spec.md → "specs/auth/spec.md"
pub fn relative_full_path(path: &Path, ws_root: Option<&Path>) -> String {
    let rel = match ws_root {
        Some(root) => path.strip_prefix(root).unwrap_or(path),
        None => path,
    };
    rel.to_string_lossy().to_string()
}

/// タブを前方（左）にナビゲートしたときの新インデックスを返す。
/// ラップアラウンド対応: インデックス0から左に移動すると最後のタブになる。
pub fn prev_tab_index(current: usize, count: usize) -> usize {
    if count == 0 {
        return 0;
    }
    if current == 0 {
        count - 1
    } else {
        current - 1
    }
}

/// タブを後方（右）にナビゲートしたときの新インデックスを返す。
/// ラップアラウンド対応: 最後のタブから右に移動すると最初のタブになる。
pub fn next_tab_index(current: usize, count: usize) -> usize {
    if count == 0 {
        return 0;
    }
    (current + 1) % count
}
