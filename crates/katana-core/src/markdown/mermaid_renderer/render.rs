use std::{io::Write, process::Stdio};
use tempfile::NamedTempFile;

use crate::markdown::color_preset::DiagramColorPreset;
use crate::markdown::diagram::{DiagramBlock, DiagramResult};

use super::resolve::build_mmdc_command;

pub fn is_mmdc_available() -> bool {
    build_mmdc_command()
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/* WHY: Rendering as PNG with mmdc (Puppeteer/Chrome based) bypasses resvg's lack of support for <foreignObject>. */
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

/* WHY: PNG output ensures mmdc (Puppeteer) correctly renders all SVG elements. Bypasses text loss caused by <foreignObject> which resvg doesn't support. */
pub fn run_mmdc_process(source: &str) -> Result<Vec<u8>, String> {
    let input_file = create_input_file(source)?;
    // WHY: mmdc determines the format by the output file's extension.
    let output_path = input_file.path().with_extension("png");

    let preset = DiagramColorPreset::current();
    let status = build_mmdc_command()
        .args(vec![
            "-i",
            input_file.path().to_str().unwrap_or(""),
            "-o",
            output_path.to_str().unwrap_or(""),
            "--backgroundColor",
            preset.background,
            "--theme",
            preset.mermaid_theme,
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

pub fn create_input_file(source: &str) -> Result<NamedTempFile, String> {
    let mut file = NamedTempFile::with_suffix(".mmd")
        .map_err(|e| format!("Temp file creation failed: {e}"))?;
    file.write_all(source.as_bytes())
        .map_err(|e| format!("Temp file write failed: {e}"))?;
    Ok(file)
}
