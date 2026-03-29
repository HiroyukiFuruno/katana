use crate::utils::{locale_violation, parse_json_file};
use crate::Violation;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::discovery::locale_code_from_path;

fn process_catalog_entries(
    entries: &[Value],
    path: &Path,
) -> Result<BTreeSet<String>, Vec<Violation>> {
    let mut codes = BTreeSet::new();
    let mut violations = Vec::new();
    for (i, entry) in entries.iter().enumerate() {
        match validate_catalog_entry(entry, path, i) {
            Ok(code) => {
                if !codes.insert(code.clone()) {
                    violations.push(locale_violation(
                        path,
                        format!("languages.json contains duplicate code `{code}`."),
                    ));
                }
            }
            Err(violation) => violations.push(violation),
        }
    }
    if violations.is_empty() {
        Ok(codes)
    } else {
        Err(violations)
    }
}

pub fn parse_languages_catalog(locale_dir: &Path) -> Result<BTreeSet<String>, Vec<Violation>> {
    let path = locale_dir.join("languages.json");
    let Value::Array(entries) = parse_json_file(&path)? else {
        return Err(vec![locale_violation(
            &path,
            "languages.json must be a JSON array.".to_string(),
        )]);
    };
    process_catalog_entries(&entries, &path)
}

fn validate_catalog_entry(entry: &Value, path: &Path, index: usize) -> Result<String, Violation> {
    let Value::Object(entry_obj) = entry else {
        return Err(locale_violation(
            path,
            format!("languages.json entry at index {index} must be an object."),
        ));
    };

    let code = entry_obj
        .get("code")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            locale_violation(
                path,
                format!("languages.json entry {index} missing or invalid `code`."),
            )
        })?;
    let _name = entry_obj
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            locale_violation(
                path,
                format!("languages.json entry {index} missing or invalid `name`."),
            )
        })?;

    Ok(code.to_string())
}

pub fn compare_languages_catalog(
    locale_dir: &Path,
    locale_files: &[PathBuf],
    language_codes: &BTreeSet<String>,
) -> Vec<Violation> {
    let languages_path = locale_dir.join("languages.json");
    let locale_codes: BTreeSet<String> = locale_files
        .iter()
        .filter_map(|path| locale_code_from_path(path))
        .collect();
    let mut violations = Vec::new();

    for code in locale_codes
        .iter()
        .filter(|code| !language_codes.contains(code.as_str()))
    {
        violations.push(locale_violation(
            &languages_path,
            format!("Locale file `{code}.json` exists but is missing from languages.json."),
        ));
    }

    for code in language_codes
        .iter()
        .filter(|code| !locale_codes.contains(code.as_str()))
    {
        violations.push(locale_violation(
            &languages_path,
            format!("Missing locale file `{code}.json` declared in languages.json."),
        ));
    }

    violations
}
