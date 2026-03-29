use crate::utils::{is_allowed_string, locale_violation};
use crate::Violation;
use std::collections::BTreeMap;
use std::path::Path;

struct LocaleException {
    key: &'static str,
    value: &'static str,
}

/* WHY: Exclusion list for translation value validation.

[Operating Rules]
1. As a general rule, values that are identical across all languages (same as English) should be added here.
2. Applies to proper nouns (App names, Tool names, etc.), programming terms (Rust, etc.), and version numbers.
3. Using `*` for the key (`key`) or value (`value`) allows for broad matching.
4. However, meaningful words ("File", "Search", etc.) MUST NOT be broadly excluded. They require translation.

[Examples of patterns that must NOT be excluded]
- Common menu items like "Edit", "View", "Help" (should be translated in each language).
- Sentence fragments or message text that conveys meaning to the user.

[Procedure for adding]
When new proper nouns or universal identifiers (e.g. v1.0.0) are introduced, if the Linter generates false positives,
carefully review and consider before adding them to this list. */
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

pub fn compare_locale_values(
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

pub fn is_allowed_duplicate(path: &str, value: &str) -> bool {
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
