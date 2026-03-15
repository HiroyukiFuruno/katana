//! PlantUML subprocess renderer.
//!
//! Runs `java -jar plantuml.jar -pipe -tsvg`,
//! passes PlantUML source to stdin and reads SVG from stdout.
//!
//! MVP constraints:
//! - Only supports raw source containing `@startuml` / `@enduml` delimiters.
//! - JAR search path is: `PLANTUML_JAR` environment variable -> adjacent to binary -> XDG data directory.

use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use super::diagram::{DiagramBlock, DiagramResult};

/// Returns candidate paths to search for the PlantUML JAR.
pub fn jar_candidate_paths() -> Vec<PathBuf> {
    // If the environment variable is set, use only that path (ignore other candidates).
    if let Ok(env_path) = std::env::var("PLANTUML_JAR") {
        return vec![PathBuf::from(env_path)];
    }
    let mut paths = Vec::new();
    // Homebrew (Apple Silicon / Intel)
    for prefix in &["/opt/homebrew", "/usr/local"] {
        paths.push(PathBuf::from(prefix).join("opt/plantuml/libexec/plantuml.jar"));
    }
    // Same directory as the binary
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            paths.push(dir.join("plantuml.jar"));
            paths.push(dir.join("renderers").join("plantuml.jar"));
        }
    }
    // XDG / macOS app data
    if let Some(home) = dirs_sys::home_dir() {
        paths.push(home.join(".local").join("katana").join("plantuml.jar"));
    }
    paths
}

/// Default JAR path where Katana automatically installs.
pub fn default_install_path() -> Option<PathBuf> {
    dirs_sys::home_dir().map(|h| h.join(".local").join("katana").join("plantuml.jar"))
}

/// Returns the path to the available PlantUML JAR on the system. If it doesn't exist, returns `None`.
pub fn find_plantuml_jar() -> Option<PathBuf> {
    jar_candidate_paths().into_iter().find(|p| p.exists())
}

/// Converts PlantUML source to SVG.
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

/// Runs `java -jar plantuml.jar`, passes the source, and returns the SVG.
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
        .map_err(|e| format!("java startup failed: {e}"))?;

    // Write to stdin in a separate scope to drop it and send EOF.
    {
        let stdin = child.stdin.as_mut().ok_or("stdin acquisition failed")?;
        stdin
            .write_all(source.as_bytes())
            .map_err(|e| format!("stdin write failed: {e}"))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("process wait failed: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("PlantUML rendering error: {stderr}"));
    }
    String::from_utf8(output.stdout).map_err(|e| format!("SVG decode error: {e}"))
}

/// Converts SVG text into an HTML fragment for preview embedding.
pub fn svg_to_html_fragment(svg: &str) -> String {
    format!(r#"<div class="katana-diagram plantuml">{svg}</div>"#)
}
