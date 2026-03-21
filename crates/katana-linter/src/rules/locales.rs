use crate::utils::{
    collect_json_placeholders, collect_json_shape, collect_json_values, is_allowed_string,
    locale_violation, parse_json_file,
};
use crate::{JsonNodeKind, Violation};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

type LocaleBaseline = (
    BTreeMap<String, JsonNodeKind>,
    BTreeMap<String, BTreeSet<String>>,
    BTreeMap<String, String>,
);

pub fn lint_locale_files(locale_dir: &Path) -> Vec<Violation> {
    let locale_files = collect_locale_json_files(locale_dir);
    if locale_files.is_empty() {
        return vec![locale_violation(
            locale_dir,
            format!(
                "No locale JSON files found for analysis: {}",
                locale_dir.display()
            ),
        )];
    }

    let language_codes = match parse_languages_catalog(locale_dir) {
        Ok(codes) => codes,
        Err(violations) => return violations,
    };
    let mut all_violations = compare_languages_catalog(locale_dir, &locale_files, &language_codes);

    let ja_path = locale_dir.join("ja.json");
    let en_path = locale_dir.join("en.json");
    let (baseline_shape, baseline_placeholders, en_values) =
        match build_locale_baseline(&ja_path, &en_path) {
            Ok(baseline) => baseline,
            Err(violations) => {
                all_violations.extend(violations);
                return all_violations;
            }
        };

    for file in locale_files {
        let is_base_locale = file.ends_with("ja.json") || file.ends_with("en.json");
        if is_base_locale {
            continue;
        }

        let value = match parse_json_file(&file) {
            Ok(value) => value,
            Err(violations) => {
                all_violations.extend(violations);
                continue;
            }
        };

        let mut shape = BTreeMap::new();
        let mut placeholders = BTreeMap::new();
        let mut values = BTreeMap::new();
        collect_json_shape(&value, None, &mut shape);
        collect_json_placeholders(&value, None, &mut placeholders);
        collect_json_values(&value, None, &mut values);

        all_violations.extend(compare_locale_shape(&file, &baseline_shape, &shape));
        all_violations.extend(compare_locale_placeholders(
            &file,
            &baseline_shape,
            &baseline_placeholders,
            &placeholders,
        ));
        all_violations.extend(compare_locale_values(&file, &en_values, &values));
    }

    all_violations
}

fn collect_locale_json_files(locale_dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(locale_dir)
        .expect("Locale directory should be readable")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().is_some_and(|ext| ext == "json")
                && path
                    .file_name()
                    .is_some_and(|name| name != "languages.json")
        })
        .collect();
    files.sort();
    files
}

fn locale_code_from_path(path: &Path) -> Option<String> {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
}

fn parse_languages_catalog(locale_dir: &Path) -> Result<BTreeSet<String>, Vec<Violation>> {
    let path = locale_dir.join("languages.json");
    let value = parse_json_file(&path)?;
    let Value::Array(entries) = value else {
        return Err(vec![locale_violation(
            &path,
            "languages.json must be a JSON array.".to_string(),
        )]);
    };

    let mut codes = BTreeSet::new();
    let mut violations = Vec::new();

    for (index, entry) in entries.iter().enumerate() {
        let Value::Object(entry_obj) = entry else {
            violations.push(locale_violation(
                &path,
                format!("languages.json entry at index {index} must be an object."),
            ));
            continue;
        };

        let Some(code_value) = entry_obj.get("code") else {
            violations.push(locale_violation(
                &path,
                format!("languages.json entry at index {index} is missing `code`."),
            ));
            continue;
        };
        let Some(name_value) = entry_obj.get("name") else {
            violations.push(locale_violation(
                &path,
                format!("languages.json entry at index {index} is missing `name`."),
            ));
            continue;
        };

        let Value::String(code) = code_value else {
            violations.push(locale_violation(
                &path,
                format!("languages.json entry at index {index} has non-string `code`."),
            ));
            continue;
        };
        let Value::String(_) = name_value else {
            violations.push(locale_violation(
                &path,
                format!("languages.json entry at index {index} has non-string `name`."),
            ));
            continue;
        };

        if !codes.insert(code.clone()) {
            violations.push(locale_violation(
                &path,
                format!("languages.json contains duplicate code `{code}`."),
            ));
        }
    }

    if violations.is_empty() {
        Ok(codes)
    } else {
        Err(violations)
    }
}

fn compare_languages_catalog(
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

fn compare_locale_shape(
    file: &Path,
    expected_shape: &BTreeMap<String, JsonNodeKind>,
    actual_shape: &BTreeMap<String, JsonNodeKind>,
) -> Vec<Violation> {
    let mut violations = Vec::new();

    for missing in expected_shape
        .keys()
        .filter(|key| !actual_shape.contains_key(*key))
    {
        violations.push(locale_violation(
            file,
            format!("Missing locale key `{missing}` compared with ja.json/en.json."),
        ));
    }

    for extra in actual_shape
        .keys()
        .filter(|key| !expected_shape.contains_key(*key))
    {
        violations.push(locale_violation(
            file,
            format!("Unexpected locale key `{extra}` not present in ja.json/en.json."),
        ));
    }

    for (path, expected_kind) in expected_shape {
        let Some(actual_kind) = actual_shape.get(path) else {
            continue;
        };
        if actual_kind != expected_kind {
            violations.push(locale_violation(
                file,
                format!(
                    "Locale node kind mismatch at `{path}`: expected {expected_kind}, found {actual_kind}."
                ),
            ));
        }
    }

    violations
}

fn compare_locale_placeholders(
    file: &Path,
    expected_shape: &BTreeMap<String, JsonNodeKind>,
    expected_placeholders: &BTreeMap<String, BTreeSet<String>>,
    actual_placeholders: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<Violation> {
    let mut violations = Vec::new();

    for (path, kind) in expected_shape {
        if kind != &JsonNodeKind::String {
            continue;
        }

        let expected = expected_placeholders.get(path).cloned().unwrap_or_default();
        let actual = actual_placeholders.get(path).cloned().unwrap_or_default();

        if actual != expected {
            violations.push(locale_violation(
                file,
                format!(
                    "Locale placeholder mismatch at `{path}`: expected {:?}, found {:?}.",
                    expected, actual
                ),
            ));
        }
    }

    violations
}

fn compare_locale_values(
    file: &Path,
    en_values: &BTreeMap<String, String>,
    actual_values: &BTreeMap<String, String>,
) -> Vec<Violation> {
    let mut violations = Vec::new();

    for (path, en_val) in en_values {
        let Some(actual_val) = actual_values.get(path) else {
            continue;
        };

        if actual_val == en_val && !is_allowed_duplicate(path, actual_val) {
            violations.push(locale_violation(
                file,
                format!(
                    "Locale value at `{path}` is identical to English baseline (\"{actual_val}\"). Please translate it."
                ),
            ));
        }
    }

    violations
}

fn is_allowed_duplicate(path: &str, value: &str) -> bool {
    if is_allowed_string(value) {
        return true;
    }

    LOCALE_VALUE_EXCEPTIONS.iter().any(|ex| {
        let key_matches = ex.key == "*"
            || ex.key == path
            || path.split('.').next_back().is_some_and(|k| k == ex.key);
        key_matches && (ex.value == "*" || ex.value == value)
    })
}

struct LocaleException {
    key: &'static str,
    value: &'static str,
}

/// 翻訳値の検証から除外する例外リスト。
///
/// 【運用ルール】
/// 1. 原則として、すべての言語で同一の値（英語と同一）になるものはここに追加する。
/// 2. 固有名詞（アプリ名、ツール名等）、プログラミング用語（Rust等）、およびバージョン番号が対象。
/// 3. キー(`key`) または 値(`value`) に `*` を使用することで、広範なマッチングが可能。
/// 4. ただし、意味のある単語（"File", "Search" 等）を広範に除外してはならない。それらは翻訳が必要。
///
/// 【除外してはいけないパターンの例】
/// - "Edit", "View", "Help" などの一般的なメニュー項目（各言語で翻訳すべき）。
/// - 文章の断片や、ユーザーに意味を伝えるメッセージ本文。
///
/// 【追加の手順】
/// 新しい固有名詞や普遍的な識別子（v1.0.0等）が導入された際に、Linterが誤検知した場合は
/// 慎重に検討した上でこのリストに追加すること。
const LOCALE_VALUE_EXCEPTIONS: &[LocaleException] = &[
    LocaleException {
        key: "rust",
        value: "Rust",
    },
    LocaleException {
        key: "support",
        value: "Support",
    },
    LocaleException {
        key: "action_close",
        value: "OK",
    },
    LocaleException {
        key: "key",
        value: "*",
    },
    LocaleException {
        key: "kind",
        value: "*",
    },
    LocaleException {
        key: "*",
        value: "KatanA",
    },
    LocaleException {
        key: "*",
        value: "PlantUML",
    },
    LocaleException {
        key: "*",
        value: "wkhtmltopdf",
    },
    LocaleException {
        key: "*",
        value: "Rust",
    },
    LocaleException {
        key: "render_error",
        value: "*",
    },
    LocaleException {
        key: "*",
        value: "Copyright",
    },
    LocaleException {
        key: "*",
        value: "Runtime",
    },
    LocaleException {
        key: "*",
        value: "Build",
    },
    LocaleException {
        key: "*",
        value: "Version",
    },
    LocaleException {
        key: "*",
        value: "v1.0.0",
    },
    LocaleException {
        key: "*",
        value: "Version: {version}",
    },
    LocaleException {
        key: "*",
        value: "File",
    },
    LocaleException {
        key: "*",
        value: "Sponsor",
    },
    LocaleException {
        key: "*",
        value: "Layout",
    },
    LocaleException {
        key: "*",
        value: "Code",
    },
    LocaleException {
        key: "*",
        value: "Links",
    },
    LocaleException {
        key: "*",
        value: "Theme",
    },
    LocaleException {
        key: "*",
        value: "Architecture",
    },
    LocaleException {
        key: "*",
        value: "Documentation",
    },
    LocaleException {
        key: "*",
        value: "Text",
    },
];

fn build_locale_baseline(ja_path: &Path, en_path: &Path) -> Result<LocaleBaseline, Vec<Violation>> {
    let ja_value = parse_json_file(ja_path)?;
    let en_value = parse_json_file(en_path)?;

    let mut ja_shape = BTreeMap::new();
    let mut en_shape = BTreeMap::new();
    collect_json_shape(&ja_value, None, &mut ja_shape);
    collect_json_shape(&en_value, None, &mut en_shape);

    let mut violations = compare_locale_shape(ja_path, &en_shape, &ja_shape);
    violations.extend(compare_locale_shape(en_path, &ja_shape, &en_shape));

    let mut ja_placeholders = BTreeMap::new();
    let mut en_placeholders = BTreeMap::new();
    collect_json_placeholders(&ja_value, None, &mut ja_placeholders);
    collect_json_placeholders(&en_value, None, &mut en_placeholders);

    let mut en_values = BTreeMap::new();
    collect_json_values(&en_value, None, &mut en_values);

    violations.extend(compare_locale_placeholders(
        ja_path,
        &en_shape,
        &en_placeholders,
        &ja_placeholders,
    ));
    violations.extend(compare_locale_placeholders(
        en_path,
        &ja_shape,
        &ja_placeholders,
        &en_placeholders,
    ));

    if violations.is_empty() {
        Ok((ja_shape, ja_placeholders, en_values))
    } else {
        Err(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_locale_shape_detects_missing_keys() {
        let expected = BTreeMap::from([
            ("menu".to_string(), JsonNodeKind::Object),
            ("menu.file".to_string(), JsonNodeKind::String),
        ]);
        let actual = BTreeMap::from([("menu".to_string(), JsonNodeKind::Object)]);
        let violations = compare_locale_shape(Path::new("locale.json"), &expected, &actual);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("menu.file"));
    }

    #[test]
    fn compare_locale_placeholders_detects_mismatch() {
        let expected_shape =
            BTreeMap::from([("status.save_failed".to_string(), JsonNodeKind::String)]);
        let expected_placeholders = BTreeMap::from([(
            "status.save_failed".to_string(),
            BTreeSet::from(["error".to_string()]),
        )]);
        let actual_placeholders = BTreeMap::from([(
            "status.save_failed".to_string(),
            BTreeSet::from(["message".to_string()]),
        )]);

        let violations = compare_locale_placeholders(
            Path::new("locale.json"),
            &expected_shape,
            &expected_placeholders,
            &actual_placeholders,
        );
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("status.save_failed"));
    }

    #[test]
    fn build_locale_baseline_returns_errors_for_mismatched_bases() {
        let tmp = tempfile::TempDir::new().unwrap();
        let ja_path = tmp.path().join("ja.json");
        let en_path = tmp.path().join("en.json");
        std::fs::write(
            &ja_path,
            r#"{"status":{"saved":"保存しました。","failed":"失敗: {error}"}}"#,
        )
        .unwrap();
        std::fs::write(
            &en_path,
            r#"{"status":{"saved":"Saved.","failed":"Failed: {message}"}}"#,
        )
        .unwrap();

        let violations =
            build_locale_baseline(&ja_path, &en_path).expect_err("base locales should mismatch");
        assert!(!violations.is_empty());
        assert!(violations
            .iter()
            .any(|v| v.message.contains("Locale placeholder mismatch")));
    }

    #[test]
    fn lint_locale_files_reports_empty_directory() {
        let tmp = tempfile::TempDir::new().unwrap();
        let violations = lint_locale_files(tmp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0]
            .message
            .contains("No locale JSON files found for analysis"));
    }
}
