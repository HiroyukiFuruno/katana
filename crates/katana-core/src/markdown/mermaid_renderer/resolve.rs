use std::{
    path::PathBuf,
    process::{Command, Stdio},
    sync::OnceLock,
};

static MMDC_RESOLVED_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn resolve_mmdc_binary() -> PathBuf {
    // WHY: Always check env var first (not cached — allows runtime override)
    #[allow(clippy::single_match)]
    match std::env::var("MERMAID_MMDC") {
        Ok(p) => return PathBuf::from(p),
        Err(_) => {}
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

fn probe_well_known_paths() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;

    // WHY: --- Homebrew ---
    #[allow(clippy::useless_vec)]
    for brew_prefix in vec!["/opt/homebrew/bin/mmdc", "/usr/local/bin/mmdc"] {
        let p = PathBuf::from(brew_prefix);
        if p.is_file() {
            return Some(p);
        }
    }

    // WHY: --- nvm ---
    let nvm_dir = std::env::var("NVM_DIR").unwrap_or_else(|_| format!("{home}/.nvm"));
    if let Some(path) = probe_nvm_mmdc(&nvm_dir) {
        return Some(path);
    }

    // WHY: --- volta ---
    let volta_bin = PathBuf::from(format!("{home}/.volta/bin/mmdc"));
    if volta_bin.is_file() {
        return Some(volta_bin);
    }

    // WHY: --- fnm ---
    let fnm_bin = PathBuf::from(format!("{home}/.local/share/fnm/aliases/default/bin/mmdc"));
    if fnm_bin.is_file() {
        return Some(fnm_bin);
    }

    None
}

fn probe_nvm_mmdc(nvm_dir: &str) -> Option<PathBuf> {
    let alias_file = PathBuf::from(format!("{nvm_dir}/alias/default"));
    let alias = std::fs::read_to_string(&alias_file).ok()?;
    let alias = alias.trim();
    if alias.is_empty() {
        return None;
    }

    let versions_dir = PathBuf::from(format!("{nvm_dir}/versions/node"));
    let exact = versions_dir.join(alias).join("bin/mmdc");
    if exact.is_file() {
        return Some(exact);
    }

    find_mmdc_by_prefix(&versions_dir, alias)
}

fn find_mmdc_by_prefix(versions_dir: &std::path::Path, alias: &str) -> Option<PathBuf> {
    let prefix = if alias.starts_with('v') {
        alias.to_string()
    } else {
        format!("v{alias}")
    };
    let entries = std::fs::read_dir(versions_dir).ok()?;
    let mut best: Option<PathBuf> = None;
    for entry in entries.flatten() {
        if entry.file_name().to_string_lossy().starts_with(&prefix) {
            let candidate = entry.path().join("bin/mmdc");
            if candidate.is_file() {
                best = Some(candidate);
            }
        }
    }
    best
}

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

fn resolve_via_login_shell() -> Option<PathBuf> {
    let output = Command::new("/bin/zsh")
        .args(vec!["-l", "-c", "which mmdc"])
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

pub fn build_mmdc_command() -> Command {
    let mmdc = resolve_mmdc_binary();
    let mut cmd = Command::new(&mmdc);

    // WHY: Enrich PATH so that `#!/usr/bin/env node` can find `node`.
    if let Some(bin_dir) = mmdc.parent() {
        let current_path = std::env::var("PATH").unwrap_or_default();
        let bin_dir_str = bin_dir.to_string_lossy();
        if !current_path.split(':').any(|p| p == bin_dir_str.as_ref()) {
            cmd.env("PATH", format!("{bin_dir_str}:{current_path}"));
        }
    }
    cmd
}
