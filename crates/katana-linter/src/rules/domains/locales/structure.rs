use crate::utils::{
    collect_json_placeholders, collect_json_shape, collect_json_values, locale_violation,
    parse_json_file,
};
use crate::{JsonNodeKind, Violation};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

type LocaleBaseline = (
    BTreeMap<String, JsonNodeKind>,
    BTreeMap<String, BTreeSet<String>>,
    BTreeMap<String, String>,
);

pub fn build_locale_baseline(
    ja_path: &Path,
    en_path: &Path,
) -> Result<LocaleBaseline, Vec<Violation>> {
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

pub fn compare_locale_shape(
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

pub fn compare_locale_placeholders(
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
            r#"{"status":{"saved":"saved","failed":"failed: {error}"}}"#,
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
}
