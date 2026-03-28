use crate::utils::{
    collect_json_placeholders, collect_json_shape, collect_json_values, parse_json_file,
};
use crate::Violation;
use std::collections::BTreeMap;
use std::path::Path;

pub mod catalog;
pub mod discovery;
pub mod structure;
pub mod values;

use catalog::{compare_languages_catalog, parse_languages_catalog};
use discovery::collect_locale_json_files;
use structure::{build_locale_baseline, compare_locale_placeholders, compare_locale_shape};
use values::compare_locale_values;

pub fn lint_locale_files(locale_dir: &Path) -> Vec<Violation> {
    let locale_files = collect_locale_json_files(locale_dir);
    if locale_files.is_empty() {
        return vec![crate::utils::locale_violation(
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

#[cfg(test)]
mod tests {
    use super::*;

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
