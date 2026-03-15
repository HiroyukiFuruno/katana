//! Mermaid CLI (`mmdc`) subprocess renderer.
//!
//! Calls the system-installed `mmdc`,
//! converts Mermaid source to SVG and returns it.
//!
//! MVP constraints:
//! - Only works if `mmdc` is on the system PATH.
//! - Alternative binary path can be specified via the `MERMAID_MMDC` environment variable.
//! - Input is raw Mermaid source (excluding code fence markers).

use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};
use tempfile::NamedTempFile;

use super::diagram::{DiagramBlock, DiagramResult};

/// Resolves the `mmdc` binary path to use.
///
/// 1. If the `MERMAID_MMDC` environment variable is set, use it.
/// 2. Run `which mmdc` via login shell to search including paths like nvm.
///    Since GUI apps don't inherit the shell's PATH, use `sh -l -c` for a login shell.
/// 3. If neither is found, return `mmdc` as a fallback.
pub fn resolve_mmdc_binary() -> PathBuf {
    if let Ok(env_path) = std::env::var("MERMAID_MMDC") {
        return PathBuf::from(env_path);
    }

    // Resolve actual path via login shell (for nvm, volta, etc.).
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

/// Checks if `mmdc` is available.
pub fn is_mmdc_available() -> bool {
    Command::new(resolve_mmdc_binary())
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Converts Mermaid source to PNG.
///
/// Rendering as PNG with mmdc (Puppeteer/Chrome based)
/// bypasses resvg's lack of support for `<foreignObject>`.
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

/// Executes mmdc via a temporary file and returns PNG bytes.
///
/// PNG output ensures mmdc (Puppeteer) correctly renders all SVG elements.
/// Bypasses text loss caused by `<foreignObject>` which resvg doesn't support.
pub fn run_mmdc_process(source: &str) -> Result<Vec<u8>, String> {
    let input_file = create_input_file(source)?;
    // mmdc determines the format by the output file's extension.
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
        .map_err(|e| format!("mmdc startup failed: {e}"))?;

    if !status.success() {
        return Err("mmdc returned a non-zero exit code".to_string());
    }
    std::fs::read(&output_path).map_err(|e| format!("PNG read failed: {e}"))
}

/// Writes Mermaid source to a temporary file.
pub fn create_input_file(source: &str) -> Result<NamedTempFile, String> {
    let mut file = NamedTempFile::with_suffix(".mmd")
        .map_err(|e| format!("Temp file creation failed: {e}"))?;
    file.write_all(source.as_bytes())
        .map_err(|e| format!("Temp file write failed: {e}"))?;
    Ok(file)
}
