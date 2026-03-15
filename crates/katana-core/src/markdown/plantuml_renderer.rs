//! PlantUML サブプロセスレンダラー。
//!
//! `java -jar plantuml.jar -pipe -tsvg` を起動し、
//! stdin に PlantUML ソースを渡して stdout から SVG を読み取る。
//!
//! MVP 制約:
//! - 入力は `@startuml` / `@enduml` デリミタを含む生ソースのみ対応。
//! - JAR の探索パスは `PLANTUML_JAR` 環境変数 → バイナリ隣 → XDG データディレクトリ。

use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use super::diagram::{DiagramBlock, DiagramResult};

/// PlantUML JAR を探索する候補パスを返す。
pub fn jar_candidate_paths() -> Vec<PathBuf> {
    // 環境変数が設定されている場合はそのパスのみを使用する（他の候補は無視）。
    if let Ok(env_path) = std::env::var("PLANTUML_JAR") {
        return vec![PathBuf::from(env_path)];
    }
    let mut paths = Vec::new();
    // Homebrew (Apple Silicon / Intel)
    for prefix in &["/opt/homebrew", "/usr/local"] {
        paths.push(PathBuf::from(prefix).join("opt/plantuml/libexec/plantuml.jar"));
    }
    // バイナリと同じディレクトリ
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            paths.push(dir.join("plantuml.jar"));
            paths.push(dir.join("renderers").join("plantuml.jar"));
        }
    }
    // XDG / macOS アプリデータ
    if let Some(home) = dirs_sys::home_dir() {
        paths.push(home.join(".local").join("katana").join("plantuml.jar"));
    }
    paths
}

/// Katana が自動インストールするデフォルトの JAR パス。
pub fn default_install_path() -> Option<PathBuf> {
    dirs_sys::home_dir().map(|h| h.join(".local").join("katana").join("plantuml.jar"))
}

/// システムで利用可能な PlantUML JAR パスを返す。存在しなければ `None`。
pub fn find_plantuml_jar() -> Option<PathBuf> {
    jar_candidate_paths().into_iter().find(|p| p.exists())
}

/// PlantUML ソースを SVG に変換する。
pub fn render_plantuml(block: &DiagramBlock) -> DiagramResult {
    let Some(jar) = find_plantuml_jar() else {
        let install_path = default_install_path().unwrap_or_else(|| PathBuf::from("plantuml.jar"));
        return DiagramResult::NotInstalled {
            kind: "PlantUML".to_string(),
            download_url:
                "https://github.com/plantuml/plantuml/releases/latest/download/plantuml.jar"
                    .to_string(),
            install_path,
        };
    };
    match run_plantuml_process(&jar, &block.source) {
        Ok(svg) => DiagramResult::Ok(svg_to_html_fragment(&svg)),
        Err(e) => DiagramResult::Err {
            source: block.source.clone(),
            error: e,
        },
    }
}

/// `java -jar plantuml.jar` を起動してソースを渡し SVG を返す。
pub fn run_plantuml_process(jar: &Path, source: &str) -> Result<String, String> {
    let mut child = Command::new("java")
        .args([
            "-Djava.awt.headless=true",
            "-jar",
            jar.to_str().unwrap_or("plantuml.jar"),
            "-pipe",
            "-tsvg",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("java 起動失敗: {e}"))?;

    // stdin への書き込みは別スコープで drop して EOF を送る。
    {
        let stdin = child.stdin.as_mut().ok_or("stdin 取得失敗")?;
        stdin
            .write_all(source.as_bytes())
            .map_err(|e| format!("stdin 書き込み失敗: {e}"))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("プロセス待機失敗: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("PlantUML レンダリングエラー: {stderr}"));
    }
    String::from_utf8(output.stdout).map_err(|e| format!("SVG デコードエラー: {e}"))
}

/// SVG テキストをプレビュー埋め込み用の HTML フラグメントに変換する。
pub fn svg_to_html_fragment(svg: &str) -> String {
    format!(r#"<div class="katana-diagram plantuml">{svg}</div>"#)
}
