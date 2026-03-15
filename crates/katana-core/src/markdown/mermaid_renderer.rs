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
///
/// 1. `MERMAID_MMDC` 環境変数が設定されていればそれを使う。
/// 2. ログインシェル経由で `which mmdc` を実行し、nvm 等のパスも含めて探す。
///    GUI アプリはシェルの PATH を引き継がないため、`sh -l -c` でログインシェルを使う。
/// 3. どちらも見つからなければフォールバックとして `mmdc` を返す。
fn resolve_mmdc_binary() -> PathBuf {
    if let Ok(env_path) = std::env::var("MERMAID_MMDC") {
        return PathBuf::from(env_path);
    }

    // ログインシェル経由で実パスを解決する（nvm, volta 等に対応）。
    if let Ok(output) = Command::new("sh")
        .args(["-l", "-c", "which mmdc"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return PathBuf::from(path);
            }
        }
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

/// Mermaid ソースを PNG に変換する。
///
/// mmdc (Puppeteer/Chrome ベース) で PNG をレンダリングすることで
/// resvg の `<foreignObject>` 非対応を回避する。
pub fn render_mermaid(block: &DiagramBlock) -> DiagramResult {
    if !is_mmdc_available() {
        return DiagramResult::CommandNotFound {
            tool_name: "mmdc (Mermaid CLI)".to_string(),
            install_hint: "`npm install -g @mermaid-js/mermaid-cli`".to_string(),
            source: block.source.clone(),
        };
    }
    match run_mmdc_process(&block.source) {
        Ok(png_bytes) => DiagramResult::OkPng(png_bytes),
        Err(e) => DiagramResult::Err {
            source: block.source.clone(),
            error: e,
        },
    }
}

/// 一時ファイルを介して mmdc を実行し PNG バイト列を返す。
///
/// PNG 出力により mmdc (Puppeteer) がすべての SVG 要素を正しくレンダリングする。
/// resvg が非対応の `<foreignObject>` によるテキスト消失を回避できる。
fn run_mmdc_process(source: &str) -> Result<Vec<u8>, String> {
    let input_file = create_input_file(source)?;
    // mmdc は出力ファイルの拡張子で形式を判断する。
    let output_path = input_file.path().with_extension("png");

    let status = Command::new(resolve_mmdc_binary())
        .args([
            "-i",
            input_file.path().to_str().unwrap_or(""),
            "-o",
            output_path.to_str().unwrap_or(""),
            "--backgroundColor",
            "white",
            "--theme",
            "default",
            "--quiet",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .status()
        .map_err(|e| format!("mmdc 起動失敗: {e}"))?;

    if !status.success() {
        return Err("mmdc がゼロ以外の終了コードを返しました".to_string());
    }
    std::fs::read(&output_path).map_err(|e| format!("PNG 読み込み失敗: {e}"))
}

/// Mermaid ソースを一時ファイルに書き出す。
fn create_input_file(source: &str) -> Result<NamedTempFile, String> {
    let mut file =
        NamedTempFile::with_suffix(".mmd").map_err(|e| format!("一時ファイル作成失敗: {e}"))?;
    file.write_all(source.as_bytes())
        .map_err(|e| format!("一時ファイル書き込み失敗: {e}"))?;
    Ok(file)
}
