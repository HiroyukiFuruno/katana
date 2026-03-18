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

use super::color_preset::DiagramColorPreset;
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

/// Injects theme skinparams into PlantUML source based on the active color preset.
///
/// Inserts background + color defaults right after `@startuml`
/// so that SVG renders blend naturally with the host UI theme.
fn inject_theme(source: &str, preset: &DiagramColorPreset) -> String {
    let skinparams = format!(
        "\
skinparam backgroundColor {bg}
skinparam defaultFontColor {text}
skinparam classBorderColor {stroke}
skinparam classFontColor {text}
skinparam classBackgroundColor {fill}
skinparam arrowColor {arrow}
skinparam noteBorderColor {stroke}
skinparam noteBackgroundColor {note_bg}
skinparam noteFontColor {note_text}
skinparam sequenceLifeLineBorderColor {stroke}
skinparam sequenceParticipantBackgroundColor {fill}
skinparam sequenceParticipantBorderColor {stroke}
skinparam sequenceParticipantFontColor {text}
skinparam sequenceArrowColor {arrow}
",
        bg = preset.background,
        text = preset.text,
        stroke = preset.stroke,
        fill = preset.plantuml_class_bg,
        arrow = preset.arrow,
        note_bg = preset.plantuml_note_bg,
        note_text = preset.plantuml_note_text,
    );
    if let Some(pos) = source.find("@startuml") {
        let insert_at = source[pos..]
            .find('\n')
            .map(|n| pos + n + 1)
            .unwrap_or(source.len());
        format!(
            "{}{}{}",
            &source[..insert_at],
            skinparams,
            &source[insert_at..]
        )
    } else {
        // If no @startuml delimiter, wrap the source.
        format!("@startuml\n{skinparams}{source}\n@enduml")
    }
}

/// Runs `java -jar plantuml.jar`, passes the source, and returns the SVG.
pub fn run_plantuml_process(jar: &Path, source: &str) -> Result<String, String> {
    let preset = DiagramColorPreset::current();
    let themed_source = inject_theme(source, preset);
    let mut args = vec![
        "-Djava.awt.headless=true".to_string(),
        "-jar".to_string(),
        jar.to_str().unwrap_or("plantuml.jar").to_string(),
        "-pipe".to_string(),
        "-tsvg".to_string(),
    ];
    if DiagramColorPreset::is_dark_mode() {
        args.push("-darkmode".to_string());
    }
    let mut child = Command::new("java")
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("java startup failed: {e}"))?;

    // Write to stdin in a separate scope to drop it and send EOF.
    {
        let stdin = child.stdin.as_mut().ok_or("stdin acquisition failed")?;
        stdin
            .write_all(themed_source.as_bytes())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_theme_inserts_after_startuml() {
        let source = "@startuml\nA -> B\n@enduml";
        let result = inject_theme(source, &DiagramColorPreset::DARK);
        assert!(result.starts_with("@startuml\n"));
        assert!(result.contains("skinparam backgroundColor transparent"));
        assert!(result.contains("skinparam defaultFontColor #E0E0E0"));
        assert!(result.contains("A -> B"));
        assert!(result.ends_with("@enduml"));
    }

    #[test]
    fn inject_theme_wraps_when_no_startuml() {
        let source = "A -> B";
        let result = inject_theme(source, &DiagramColorPreset::DARK);
        assert!(result.starts_with("@startuml\n"));
        assert!(result.contains("skinparam backgroundColor transparent"));
        assert!(result.contains("A -> B"));
        assert!(result.ends_with("@enduml"));
    }

    #[test]
    fn inject_theme_startuml_without_newline() {
        let source = "@startuml";
        let result = inject_theme(source, &DiagramColorPreset::DARK);
        assert!(result.starts_with("@startuml"));
        assert!(result.contains("skinparam backgroundColor transparent"));
    }

    #[test]
    fn inject_theme_with_light_preset() {
        let source = "@startuml\nA -> B\n@enduml";
        let result = inject_theme(source, &DiagramColorPreset::LIGHT);
        assert!(result.contains("skinparam defaultFontColor #333333"));
        assert!(result.contains("skinparam classBackgroundColor #FEFECE"));
    }
}
