//! Mermaid CLI (`mmdc`) サブプロセスレンダラー。
//!
//! システムにインストールされた `mmdc` を呼び出し、
//! Mermaid ソースを SVG に変換して返す。
//!
//! MVP 制約:
//! - `mmdc` がシステム PATH 上にある場合のみ動作する。
//! - `MERMAID_MMDC` 環境変数で代替バイナリパスを指定可能。
//! - 入力は生の Mermaid ソース（コードフェンスのマーカーを除く）。

use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};
use tempfile::NamedTempFile;

use super::diagram::{DiagramBlock, DiagramResult};

/// 使用する `mmdc` バイナリパスを解決する。
fn resolve_mmdc_binary() -> PathBuf {
    if let Ok(env_path) = std::env::var("MERMAID_MMDC") {
        return PathBuf::from(env_path);
    }
    PathBuf::from("mmdc")
}

/// `mmdc` が利用可能かどうかを確認する。
pub fn is_mmdc_available() -> bool {
    Command::new(resolve_mmdc_binary())
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Mermaid ソースを SVG に変換する。
pub fn render_mermaid(block: &DiagramBlock) -> DiagramResult {
    if !is_mmdc_available() {
        return DiagramResult::Err {
            source: block.source.clone(),
            error: "mmdc (Mermaid CLI) が見つかりません。`npm install -g @mermaid-js/mermaid-cli` でインストールしてください。".to_string(),
        };
    }
    match run_mmdc_process(&block.source) {
        Ok(svg) => DiagramResult::Ok(svg_to_html_fragment(&svg)),
        Err(e) => DiagramResult::Err {
            source: block.source.clone(),
            error: e,
        },
    }
}

/// 一時ファイルを介して mmdc を実行し SVG を返す。
///
/// mmdc は stdin パイプを直接サポートしないため、
/// 入力を一時ファイルに書き出し、出力先も一時ファイルとして指定する。
fn run_mmdc_process(source: &str) -> Result<String, String> {
    let input_file = create_input_file(source)?;
    let output_path = input_file.path().with_extension("svg");

    let status = Command::new(resolve_mmdc_binary())
        .args([
            "-i",
            input_file.path().to_str().unwrap_or(""),
            "-o",
            output_path.to_str().unwrap_or(""),
            "--quiet",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .status()
        .map_err(|e| format!("mmdc 起動失敗: {e}"))?;

    if !status.success() {
        return Err("mmdc がゼロ以外の終了コードを返しました".to_string());
    }
    std::fs::read_to_string(&output_path).map_err(|e| format!("SVG 読み込み失敗: {e}"))
}

/// Mermaid ソースを一時ファイルに書き出す。
fn create_input_file(source: &str) -> Result<NamedTempFile, String> {
    let mut file =
        NamedTempFile::with_suffix(".mmd").map_err(|e| format!("一時ファイル作成失敗: {e}"))?;
    file.write_all(source.as_bytes())
        .map_err(|e| format!("一時ファイル書き込み失敗: {e}"))?;
    Ok(file)
}

/// SVG テキストをプレビュー埋め込み用の HTML フラグメントに変換する。
fn svg_to_html_fragment(svg: &str) -> String {
    format!(r#"<div class="katana-diagram mermaid">{svg}</div>"#)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::diagram::{DiagramBlock, DiagramKind};

    #[test]
    fn mmdc未検出時はエラー結果を返す() {
        // 存在しないバイナリを指定してフォールバックを検証する。
        std::env::set_var("MERMAID_MMDC", "/nonexistent/mmdc");
        let block = DiagramBlock {
            kind: DiagramKind::Mermaid,
            source: "graph TD; A-->B".to_string(),
        };
        let result = render_mermaid(&block);
        assert!(matches!(result, DiagramResult::Err { .. }));
        // テスト後に環境変数を戻す。
        std::env::remove_var("MERMAID_MMDC");
    }

    // mmdc がシステムで利用可能な場合のみ実行する結合テスト。
    #[test]
    fn mmdcが利用可能なら正しくsvgを返す() {
        // MERMAID_MMDC が nonexistent になっている場合はスキップ。
        if std::env::var("MERMAID_MMDC").as_deref() == Ok("/nonexistent/mmdc") {
            return;
        }
        if !is_mmdc_available() {
            return; // mmdc が未インストールならスキップ。
        }
        let block = DiagramBlock {
            kind: DiagramKind::Mermaid,
            source: "graph TD; A-->B".to_string(),
        };
        let result = render_mermaid(&block);
        assert!(matches!(result, DiagramResult::Ok(_)));
    }
}
