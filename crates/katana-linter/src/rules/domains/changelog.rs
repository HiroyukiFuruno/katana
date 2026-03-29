use crate::Violation;
use std::path::Path;

pub fn lint_changelog_contains_current_version(root: &Path) -> Vec<Violation> {
    let cargo_toml = root.join("Cargo.toml");
    let changelog = root.join("CHANGELOG.md");

    let version = match parse_workspace_version_from_cargo_toml(&cargo_toml) {
        Ok(version) => version,
        Err(violations) => return violations,
    };

    match changelog_contains_version_heading(&changelog, &version) {
        Ok(true) => Vec::new(),
        Ok(false) => vec![Violation {
            file: changelog.clone(),
            line: 0,
            column: 0,
            message: format!(
                "CHANGELOG.md is missing a release heading for workspace version `{version}`."
            ),
        }],
        Err(violations) => violations,
    }
}

fn parse_workspace_version_from_cargo_toml(path: &Path) -> Result<String, Vec<Violation>> {
    let source = std::fs::read_to_string(path).map_err(|err| {
        vec![Violation {
            file: path.to_path_buf(),
            line: 0,
            column: 0,
            message: format!("Cargo.toml read error: {err}"),
        }]
    })?;

    parse_workspace_version_from_str(&source, path)
}

fn parse_workspace_version_from_str(source: &str, path: &Path) -> Result<String, Vec<Violation>> {
    let mut in_workspace_package = false;

    for raw_line in source.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_workspace_package = line == "[workspace.package]";
            continue;
        }

        if !in_workspace_package {
            continue;
        }

        if let Some(version) = extract_version_value(line, path)? {
            return Ok(version);
        }
    }

    Err(vec![Violation {
        file: path.to_path_buf(),
        line: 0,
        column: 0,
        message: "Missing workspace.package.version in Cargo.toml.".to_string(),
    }])
}

fn extract_version_value(line: &str, path: &Path) -> Result<Option<String>, Vec<Violation>> {
    let line = line.split('#').next().unwrap_or_default().trim();
    if line.is_empty() {
        return Ok(None);
    }

    let Some((key, value)) = line.split_once('=') else {
        return Ok(None);
    };
    if key.trim() != "version" {
        return Ok(None);
    }

    let value = value.trim();
    let Some(value) = value.strip_prefix('"').and_then(|it| it.strip_suffix('"')) else {
        return Err(vec![Violation {
            file: path.to_path_buf(),
            line: 0,
            column: 0,
            message: "workspace.package.version must be a TOML string.".to_string(),
        }]);
    };

    Ok(Some(value.to_string()))
}

fn changelog_contains_version_heading(path: &Path, version: &str) -> Result<bool, Vec<Violation>> {
    let source = std::fs::read_to_string(path).map_err(|err| {
        vec![Violation {
            file: path.to_path_buf(),
            line: 0,
            column: 0,
            message: format!("CHANGELOG.md read error: {err}"),
        }]
    })?;

    let expected_prefix = format!("## [{version}]");
    Ok(source
        .lines()
        .map(str::trim_start)
        .any(|line| line.starts_with(&expected_prefix)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_workspace_version_from_cargo_toml_extracts_version() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), "[workspace.package]\nversion = \"1.2.3\"").unwrap();
        let version = parse_workspace_version_from_cargo_toml(tmp.path()).unwrap();
        assert_eq!(version, "1.2.3");
    }

    #[test]
    fn changelog_contains_version_heading_detects_heading() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), "# Changelog\n\n## [1.2.3] - 2026-03-21").unwrap();
        let contains = changelog_contains_version_heading(tmp.path(), "1.2.3").unwrap();
        assert!(contains);
    }
}
