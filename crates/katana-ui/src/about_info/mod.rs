//! Application metadata for the About dialog.
//!
//! This module provides compile-time and runtime information used by the custom
//! About window. All values are derived from `Cargo.toml` or computed at build
//! time, making them testable and reliable.
//!
//! ## Naming convention
//! - **Display name** (`APP_DISPLAY_NAME`): "KatanA" — used in UI elements.
//! - **Product name** (`APP_PRODUCT_NAME`): "KatanA Desktop" — used in About info.
//! - Internal crate names remain lowercase (e.g. `katana-core`).

/// Application display name (shown in menu bar, window title, Dock).
pub const APP_DISPLAY_NAME: &str = "KatanA";

/// Full product name (shown in About dialog).
pub const APP_PRODUCT_NAME: &str = "KatanA Desktop";

/// Application version, read from `Cargo.toml` at compile time.
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build identifier. Set via `KATANA_BUILD` env at compile time, or "dev".
pub const APP_BUILD: &str = match option_env!("KATANA_BUILD") {
    Some(v) => v,
    None => "dev",
};

/// Copyright holder.
pub const APP_COPYRIGHT: &str = "© 2026 KatanA Project";

/// License type.
pub const APP_LICENSE: &str = "MIT License";

/// Project repository URL.
pub const APP_REPOSITORY: &str = "https://github.com/HiroyukiFuruno/KatanA";

/// Documentation URL.
pub const APP_DOCS_URL: &str = "https://github.com/HiroyukiFuruno/KatanA/tree/master/docs";

/// Issue tracker URL.
pub const APP_ISSUES_URL: &str = "https://github.com/HiroyukiFuruno/KatanA/issues";

/// Sponsor / Support URL.
pub const APP_SPONSOR_URL: &str = "https://github.com/sponsors/HiroyukiFuruno";

/// Short description of the application.
pub const APP_DESCRIPTION: &str = "A fast, keyboard-driven Markdown editor built with Rust.";

// ─────────────────────────────────────────────
// Runtime info
// ─────────────────────────────────────────────

/// Runtime system information for debug/support purposes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemInfo {
    /// Operating system (e.g. "macos", "linux", "windows").
    pub os: String,
    /// CPU architecture (e.g. "aarch64", "x86_64").
    pub arch: String,
    /// Rust compiler version used to build (e.g. "rustc 1.85.0 (...)").
    pub rustc_version: String,
}

/// Collects runtime system information.
pub fn system_info() -> SystemInfo {
    SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        rustc_version: env!("KATANA_RUSTC_VERSION").to_string(),
    }
}

/// All structured About information.
#[derive(Debug, Clone)]
pub struct AboutInfo {
    pub product_name: &'static str,
    pub version: &'static str,
    pub build: &'static str,
    pub copyright: &'static str,
    pub license: &'static str,
    pub description: &'static str,
    pub repository: &'static str,
    pub docs_url: &'static str,
    pub issues_url: &'static str,
    pub sponsor_url: &'static str,
    pub system: SystemInfo,
}

/// Returns all About information as a structured object.
pub fn about_info() -> AboutInfo {
    AboutInfo {
        product_name: APP_PRODUCT_NAME,
        version: APP_VERSION,
        build: APP_BUILD,
        copyright: APP_COPYRIGHT,
        license: APP_LICENSE,
        description: APP_DESCRIPTION,
        repository: APP_REPOSITORY,
        docs_url: APP_DOCS_URL,
        issues_url: APP_ISSUES_URL,
        sponsor_url: APP_SPONSOR_URL,
        system: system_info(),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ── 1. Basic info ──

    #[test]
    fn display_name_is_katana() {
        assert_eq!(APP_DISPLAY_NAME, "KatanA");
    }

    #[test]
    fn product_name_is_katana_desktop() {
        assert_eq!(APP_PRODUCT_NAME, "KatanA Desktop");
    }

    #[test]
    fn version_is_not_empty_and_semver() {
        assert!(!APP_VERSION.is_empty());
        let parts: Vec<&str> = APP_VERSION.split('.').collect();
        assert!(parts.len() >= 2, "Version should be semver: {APP_VERSION}");
        for part in &parts {
            assert!(part.parse::<u32>().is_ok(), "'{part}' is not a number");
        }
    }

    #[test]
    fn build_is_not_empty() {
        assert!(!APP_BUILD.is_empty(), "APP_BUILD must not be empty");
    }

    #[test]
    fn copyright_contains_year_and_project() {
        assert!(APP_COPYRIGHT.contains('©'));
        assert!(APP_COPYRIGHT.contains("KatanA Project"));
    }

    #[test]
    fn copyright_year_matches_current_year() {
        // Extract the year from "© YYYY KatanA Project"
        let year_str: String = APP_COPYRIGHT
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect();
        let copyright_year: u32 = year_str
            .parse()
            .expect("Copyright should contain a 4-digit year");
        // Current year at compile time — if this fails, the copyright is stale.
        let current_year = time::OffsetDateTime::now_utc().year() as u32;
        assert_eq!(
            copyright_year, current_year,
            "Copyright year ({copyright_year}) does not match current year ({current_year}). Update APP_COPYRIGHT."
        );
    }

    #[test]
    fn description_fits_single_line() {
        // About window width ≈ 400px, at ~7px/char ≈ 57 chars.
        // Allow generous margin for font variation.
        const MAX_DESCRIPTION_LEN: usize = 60;
        let len = APP_DESCRIPTION.len();
        assert!(
            len <= MAX_DESCRIPTION_LEN,
            "APP_DESCRIPTION is {len} chars (max {MAX_DESCRIPTION_LEN}). \
             Shorten it so it fits on one line in the About dialog.",
        );
    }

    #[test]
    fn repository_url_uses_correct_github_account() {
        const EXPECTED_BASE: &str = "https://github.com/HiroyukiFuruno/KatanA";
        assert_eq!(
            APP_REPOSITORY, EXPECTED_BASE,
            "Repository URL must be {EXPECTED_BASE}"
        );
        // Derived URLs must be under the same base
        assert!(
            APP_DOCS_URL.starts_with(EXPECTED_BASE),
            "Docs URL must start with {EXPECTED_BASE}"
        );
        assert!(
            APP_ISSUES_URL.starts_with(EXPECTED_BASE),
            "Issues URL must start with {EXPECTED_BASE}"
        );
    }

    #[test]
    fn binary_name_matches_display_name() {
        // The binary name determines the macOS Dock label for unbundled executables.
        // It MUST be "KatanA" to show the correct Dock label during development.
        let cargo_toml =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
                .expect("Cargo.toml should be readable");
        // Find `name = "..."` under [[bin]] section.
        let in_bin_section = cargo_toml
            .lines()
            .skip_while(|line| !line.starts_with("[[bin]]"))
            .skip(1) // skip the [[bin]] line itself
            .take_while(|line| !line.starts_with('['))
            .find(|line| line.trim().starts_with("name"))
            .expect("[[bin]] section should have a name field");
        let bin_name = in_bin_section
            .split('=')
            .nth(1)
            .unwrap()
            .trim()
            .trim_matches('"');
        assert_eq!(
            bin_name, APP_DISPLAY_NAME,
            "Binary name '{bin_name}' in Cargo.toml must match APP_DISPLAY_NAME '{APP_DISPLAY_NAME}' for Dock label"
        );
    }

    // ── 2. Runtime info ──

    #[test]
    fn system_info_os_is_valid() {
        let info = system_info();
        assert!(!info.os.is_empty());
        let valid = ["macos", "linux", "windows"];
        assert!(
            valid.contains(&info.os.as_str()),
            "Unexpected OS: {}",
            info.os
        );
    }

    #[test]
    fn system_info_arch_is_valid() {
        let info = system_info();
        assert!(!info.arch.is_empty());
        let valid = ["aarch64", "x86_64", "x86", "arm"];
        assert!(
            valid.contains(&info.arch.as_str()),
            "Unexpected arch: {}",
            info.arch
        );
    }

    #[test]
    fn system_info_rustc_version_matches_toolchain() {
        let info = system_info();
        assert!(!info.rustc_version.is_empty());
        assert!(
            info.rustc_version.starts_with("rustc"),
            "Expected 'rustc ...' but got: {}",
            info.rustc_version
        );
        // Verify it matches the actual rustc in PATH.
        let output = std::process::Command::new("rustc")
            .arg("--version")
            .output()
            .expect("rustc should be available");
        let expected = String::from_utf8_lossy(&output.stdout).trim().to_string();
        assert_eq!(
            info.rustc_version, expected,
            "Compiled-in version must match current toolchain"
        );
    }

    // ── 3. License ──

    #[test]
    fn license_is_mit() {
        assert_eq!(APP_LICENSE, "MIT License");
    }

    // ── 4. Repository ──

    #[test]
    fn repository_url_is_https() {
        assert!(APP_REPOSITORY.starts_with("https://"));
        assert!(APP_REPOSITORY.contains("github.com"));
    }

    // ── 5. Documentation ──

    #[test]
    fn docs_url_is_under_repository() {
        assert!(APP_DOCS_URL.starts_with(APP_REPOSITORY));
    }

    // ── 6. Issue Report ──

    #[test]
    fn issues_url_is_under_repository() {
        assert!(APP_ISSUES_URL.starts_with(APP_REPOSITORY));
        assert!(APP_ISSUES_URL.contains("issues"));
    }

    // ── 7. Support / Sponsor ──

    #[test]
    fn sponsor_url_is_valid() {
        assert!(
            APP_SPONSOR_URL.starts_with("https://"),
            "Sponsor URL must be HTTPS"
        );
        assert!(
            APP_SPONSOR_URL.contains("github.com/sponsors"),
            "Sponsor URL must point to GitHub Sponsors"
        );
    }

    // ── about_info() integration ──

    #[test]
    fn about_info_contains_all_fields() {
        let info = about_info();
        assert_eq!(info.product_name, APP_PRODUCT_NAME);
        assert_eq!(info.version, APP_VERSION);
        assert_eq!(info.build, APP_BUILD);
        assert_eq!(info.copyright, APP_COPYRIGHT);
        assert_eq!(info.license, APP_LICENSE);
        assert_eq!(info.description, APP_DESCRIPTION);
        assert_eq!(info.repository, APP_REPOSITORY);
        assert_eq!(info.docs_url, APP_DOCS_URL);
        assert_eq!(info.issues_url, APP_ISSUES_URL);
        assert_eq!(info.sponsor_url, APP_SPONSOR_URL);
        assert!(!info.system.os.is_empty());
        assert!(!info.system.arch.is_empty());
    }

    #[test]
    fn about_info_system_matches_standalone() {
        let info = about_info();
        let standalone = system_info();
        assert_eq!(info.system, standalone);
    }
}
