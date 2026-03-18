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
    sync::OnceLock,
};
use tempfile::NamedTempFile;

use super::color_preset::DiagramColorPreset;
use super::diagram::{DiagramBlock, DiagramResult};

/// Process-wide cache for the resolved `mmdc` binary path (excluding env-var override).
static MMDC_RESOLVED_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Returns the `mmdc` binary path using a fast multi-tier strategy.
///
/// `MERMAID_MMDC` env var is always checked fresh (not cached) so runtime
/// overrides work immediately.  The remaining resolution tiers are cached
/// after first invocation via `OnceLock`.
///
/// Resolution order (stops at first hit):
/// 1. `MERMAID_MMDC` environment variable — explicit override (always fresh).
/// 2. Homebrew paths (`/opt/homebrew/bin`, `/usr/local/bin`) — direct
///    filesystem probe, no subprocess. (cached)
/// 3. Well-known Node version manager paths (nvm, volta, fnm) — direct
///    filesystem probe, no subprocess, sub-millisecond. (cached)
/// 4. Current `PATH` lookup via `which`. (cached)
/// 5. `/bin/zsh -l -c "which mmdc"` — login shell fallback. (cached)
/// 6. Bare `mmdc` — lets the OS try at execution time. (cached)
pub fn resolve_mmdc_binary() -> PathBuf {
    // Always check env var first (not cached — allows runtime override)
    if let Ok(p) = std::env::var("MERMAID_MMDC") {
        return PathBuf::from(p);
    }

    MMDC_RESOLVED_PATH
        .get_or_init(|| {
            probe_well_known_paths()
                .or_else(which_from_current_path)
                .or_else(resolve_via_login_shell)
                .unwrap_or_else(|| PathBuf::from("mmdc"))
        })
        .clone()
}

/// Probes well-known binary directories for `mmdc`.
///
/// Checks Homebrew, nvm, volta, and fnm default install locations without
/// spawning any subprocess. Typical cost: a handful of `stat()` calls (~μs).
fn probe_well_known_paths() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;

    // --- Homebrew ---
    // Apple Silicon: /opt/homebrew/bin, Intel Mac: /usr/local/bin
    for brew_prefix in ["/opt/homebrew/bin/mmdc", "/usr/local/bin/mmdc"] {
        let p = PathBuf::from(brew_prefix);
        if p.is_file() {
            return Some(p);
        }
    }

    // --- nvm ---
    // nvm stores the default alias in ~/.nvm/alias/default (e.g. "v24" or "24")
    let nvm_dir = std::env::var("NVM_DIR").unwrap_or_else(|_| format!("{home}/.nvm"));
    if let Some(path) = probe_nvm_mmdc(&nvm_dir) {
        return Some(path);
    }

    // --- volta ---
    // volta shims live in ~/.volta/bin/
    let volta_bin = PathBuf::from(format!("{home}/.volta/bin/mmdc"));
    if volta_bin.is_file() {
        return Some(volta_bin);
    }

    // --- fnm ---
    // fnm aliases live in ~/.local/share/fnm/aliases/default/bin/
    let fnm_bin = PathBuf::from(format!("{home}/.local/share/fnm/aliases/default/bin/mmdc"));
    if fnm_bin.is_file() {
        return Some(fnm_bin);
    }

    None
}

/// Resolves `mmdc` under the nvm default alias without spawning a shell.
///
/// Reads `~/.nvm/alias/default`, then glob-matches the version directory.
fn probe_nvm_mmdc(nvm_dir: &str) -> Option<PathBuf> {
    let alias_file = PathBuf::from(format!("{nvm_dir}/alias/default"));
    let alias = std::fs::read_to_string(&alias_file).ok()?;
    let alias = alias.trim();
    if alias.is_empty() {
        return None;
    }

    let versions_dir = PathBuf::from(format!("{nvm_dir}/versions/node"));
    // Try exact match first (e.g. alias = "v24.14.0")
    let exact = versions_dir.join(alias).join("bin/mmdc");
    if exact.is_file() {
        return Some(exact);
    }

    // Prefix match (e.g. alias = "v24" or "24")
    let prefix = if alias.starts_with('v') {
        alias.to_string()
    } else {
        format!("v{alias}")
    };
    let entries = std::fs::read_dir(&versions_dir).ok()?;
    // Pick the last (highest) matching version
    let mut best: Option<PathBuf> = None;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(&prefix) {
            let candidate = entry.path().join("bin/mmdc");
            if candidate.is_file() {
                best = Some(candidate);
            }
        }
    }
    best
}

/// Tries `which mmdc` using the current process PATH (no shell spawn).
fn which_from_current_path() -> Option<PathBuf> {
    let output = Command::new("which")
        .arg("mmdc")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Some(PathBuf::from(path));
        }
    }
    None
}

/// Spawns a zsh login shell to resolve `mmdc` from the user's full shell config.
///
/// This is the slowest path (~200-500ms) but handles any custom setup.
fn resolve_via_login_shell() -> Option<PathBuf> {
    let output = Command::new("/bin/zsh")
        .args(["-l", "-c", "which mmdc"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Some(PathBuf::from(path));
        }
    }
    None
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

    let preset = DiagramColorPreset::current();
    let status = Command::new(resolve_mmdc_binary())
        .args([
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

/// Writes Mermaid source to a temporary file.
pub fn create_input_file(source: &str) -> Result<NamedTempFile, String> {
    let mut file = NamedTempFile::with_suffix(".mmd")
        .map_err(|e| format!("Temp file creation failed: {e}"))?;
    file.write_all(source.as_bytes())
        .map_err(|e| format!("Temp file write failed: {e}"))?;
    Ok(file)
}
