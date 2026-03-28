use crate::{JsonNodeKind, Violation};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

/// Checks if a syn attribute contains `#[cfg(test)]`.
pub fn has_cfg_test_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("cfg") {
            if let Ok(syn::Meta::Path(path)) = attr.parse_args::<syn::Meta>() {
                return path.is_ident("test");
            }
        }
        attr.path().is_ident("test")
    })
}

/// Checks if a number corresponds to typical UI edge cases (0, 1, 2, 100, -1).
pub fn is_allowed_number(value: f64) -> bool {
    // WHY: 0, 1, 2, 100, -1 are commonly used in UI layouts or logic
    (value - 0.0).abs() < f64::EPSILON
        || (value - 1.0).abs() < f64::EPSILON
        || (value - 2.0).abs() < f64::EPSILON
        || (value - 100.0).abs() < f64::EPSILON
        || (value - (-1.0)).abs() < f64::EPSILON
}

/// Get (line, column) from `proc_macro2::Span`.
pub fn span_location(span: proc_macro2::Span) -> (usize, usize) {
    (span.start().line, span.start().column + 1)
}

/// Parses a file path into a `syn::File` AST tree.
///
/// # Errors
/// Returns a list of `Violation` if the file cannot be read or its syntax is invalid.
pub fn parse_file(path: &Path) -> Result<syn::File, Vec<Violation>> {
    let source = std::fs::read_to_string(path).map_err(|err| {
        vec![Violation {
            file: path.to_path_buf(),
            line: 0,
            column: 0,
            message: format!("Rust file read error: {err}"),
        }]
    })?;

    syn::parse_file(&source).map_err(|err| {
        let (line, column) = span_location(err.span());
        vec![Violation {
            file: path.to_path_buf(),
            line,
            column,
            message: format!("Syntax parse error: {err}"),
        }]
    })
}

/// Parses a custom locale JSON file into a `serde_json::Value`.
///
/// # Errors
/// Returns a list of `Violation` if the file reading or JSON parsing fails.
pub fn parse_json_file(path: &Path) -> Result<Value, Vec<Violation>> {
    let source = std::fs::read_to_string(path).map_err(|err| {
        vec![Violation {
            file: path.to_path_buf(),
            line: 0,
            column: 0,
            message: format!("Locale file read error: {err}"),
        }]
    })?;

    serde_json::from_str(&source).map_err(|err| {
        vec![Violation {
            file: path.to_path_buf(),
            line: err.line(),
            column: err.column(),
            message: format!("Locale JSON parse error: {err}"),
        }]
    })
}

/// Recursively flattens a JSON structure to collect paths and their value types (shapes).
pub fn collect_json_shape(
    value: &Value,
    path: Option<&str>,
    out: &mut BTreeMap<String, JsonNodeKind>,
) {
    let kind = JsonNodeKind::from_value(value);
    if let Some(path) = path {
        out.insert(path.to_string(), kind);
    }

    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let child_path = path
                    .map(|prefix| format!("{prefix}.{key}"))
                    .unwrap_or_else(|| key.to_string());
                collect_json_shape(child, Some(&child_path), out);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                let child_path = path
                    .map(|prefix| format!("{prefix}[{index}]"))
                    .unwrap_or_else(|| format!("[{index}]"));
                collect_json_shape(child, Some(&child_path), out);
            }
        }
        Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Null => {}
    }
}

/// Flattens a JSON structure to collect paths mapping to their actual string values.
pub fn collect_json_values(value: &Value, path: Option<&str>, out: &mut BTreeMap<String, String>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let child_path = path
                    .map(|prefix| format!("{prefix}.{key}"))
                    .unwrap_or_else(|| key.to_string());
                collect_json_values(child, Some(&child_path), out);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                let child_path = path
                    .map(|prefix| format!("{prefix}[{index}]"))
                    .unwrap_or_else(|| format!("[{index}]"));
                collect_json_values(child, Some(&child_path), out);
            }
        }
        Value::String(text) => {
            if let Some(path) = path {
                out.insert(path.to_string(), text.clone());
            }
        }
        _ => {}
    }
}

/// Extracts embedded `{placeholders}` for each key within the JSON object tree.
pub fn collect_json_placeholders(
    value: &Value,
    path: Option<&str>,
    out: &mut BTreeMap<String, BTreeSet<String>>,
) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let child_path = path
                    .map(|prefix| format!("{prefix}.{key}"))
                    .unwrap_or_else(|| key.to_string());
                collect_json_placeholders(child, Some(&child_path), out);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                let child_path = path
                    .map(|prefix| format!("{prefix}[{index}]"))
                    .unwrap_or_else(|| format!("[{index}]"));
                collect_json_placeholders(child, Some(&child_path), out);
            }
        }
        Value::String(text) => {
            if let Some(path) = path {
                out.insert(path.to_string(), extract_placeholders(text));
            }
        }
        Value::Number(_) | Value::Bool(_) | Value::Null => {}
    }
}

/// Scans text to extract `{placeholder_name}` embedded parameter tags.
pub fn extract_placeholders(text: &str) -> BTreeSet<String> {
    let mut placeholders = BTreeSet::new();
    let bytes = text.as_bytes();
    let mut start = 0usize;

    while start < bytes.len() {
        if bytes[start] != b'{' {
            start += 1;
            continue;
        }

        let Some(end_rel) = bytes[start + 1..].iter().position(|byte| *byte == b'}') else {
            break;
        };
        let end = start + 1 + end_rel;
        let candidate = &text[start + 1..end];
        if is_placeholder_name(candidate) {
            placeholders.insert(candidate.to_string());
        }
        start = end + 1;
    }

    placeholders
}

/// Checks if a string acts like a valid template placeholder name.
pub fn is_placeholder_name(candidate: &str) -> bool {
    let mut chars = candidate.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|char| char.is_ascii_alphanumeric() || char == '_')
}

/// Identifies non-standard strings like ui icons or typical symbols to avoid false positives.
pub fn is_allowed_string(s: &str) -> bool {
    let trimmed = s.trim();

    if trimmed.is_empty() {
        return true;
    }

    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() == 1 {
        let c = chars[0];
        // WHY: Allow if it's not an ASCII alphabet (a-z, A-Z)
        if !c.is_ascii_alphabetic() {
            return true;
        }
        // WHY: Allow single letter "x" (often used as close button in UI, etc.)
        if c == 'x' || c == 'X' {
            return true;
        }
        return false;
    }

    // WHY: All characters are non-alphabetic (symbol, emoji, number, or whitespace only)
    if trimmed
        .chars()
        .all(|c| !c.is_alphabetic() || is_emoji_or_symbol(c))
    {
        return true;
    }

    false
}

/// Matches common emoji and block elements to permit them within strict string rules.
pub fn is_emoji_or_symbol(c: char) -> bool {
    matches!(c,
        '\u{2000}'..='\u{2BFF}'
        | '\u{2E00}'..='\u{2E7F}'
        | '\u{3000}'..='\u{303F}'
        | '\u{FE00}'..='\u{FE0F}'
        | '\u{FE30}'..='\u{FE4F}'
        | '\u{1F000}'..='\u{1FAFF}'
        | '\u{E0000}'..='\u{E007F}'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_allowed_string_allows_multiplication_sign() {
        assert!(is_allowed_string("×"));
    }

    #[test]
    fn is_allowed_string_allows_common_ui_shorthands() {
        assert!(is_allowed_string("x"));
        assert!(is_allowed_string("X"));
    }

    #[test]
    fn is_allowed_string_denies_normal_words() {
        assert!(!is_allowed_string("Hello"));
        assert!(!is_allowed_string("Save"));
        assert!(!is_allowed_string("a"));
    }

    #[test]
    fn is_allowed_string_allows_symbols_and_numbers() {
        assert!(is_allowed_string("123"));
        assert!(is_allowed_string("1.0"));
        assert!(is_allowed_string("(!)"));
        assert!(is_allowed_string("🔄"));
    }

    #[test]
    fn parse_file_handles_missing_file() {
        let result = parse_file(Path::new("missing_file_random_123.rs"));
        let Err(violations) = result else {
            panic!("Expected error for missing file");
        };
        assert!(violations[0].message.contains("Rust file read error"));
    }

    #[test]
    fn parse_file_handles_invalid_syntax() {
        let tmp = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
        std::fs::write(tmp.path(), "invalid rust code").unwrap();
        let result = parse_file(tmp.path());
        let Err(violations) = result else {
            panic!("Expected error for invalid syntax");
        };
        assert!(violations[0].message.contains("Syntax parse error"));
    }

    #[test]
    fn parse_json_file_handles_missing_file() {
        let result = parse_json_file(Path::new("missing_file_random_123.json"));
        let Err(violations) = result else {
            panic!("Expected error for missing file");
        };
        assert!(violations[0].message.contains("Locale file read error"));
    }

    #[test]
    fn parse_json_file_handles_invalid_json() {
        let tmp = tempfile::NamedTempFile::with_suffix(".json").unwrap();
        std::fs::write(tmp.path(), "{invalid json}").unwrap();
        let result = parse_json_file(tmp.path());
        let Err(violations) = result else {
            panic!("Expected error for invalid JSON");
        };
        assert!(violations[0].message.contains("Locale JSON parse error"));
    }

    #[test]
    fn extract_placeholders_handles_edge_cases() {
        let set = extract_placeholders("Hello {unclosed");
        assert!(set.is_empty());

        let set = extract_placeholders("Hello {}");
        assert!(set.is_empty());

        let set = extract_placeholders("Hello {name}");
        assert_eq!(set.len(), 1);
        assert!(set.contains("name"));
    }

    #[test]
    fn collect_json_placeholders_handles_root_array() {
        let value = serde_json::json!(["{item}"]);
        let mut out = BTreeMap::new();
        collect_json_placeholders(&value, None, &mut out);
        assert_eq!(out.get("[0]").unwrap().len(), 1);
    }

    #[test]
    fn collect_json_placeholders_handles_primitives() {
        let value = serde_json::json!(123);
        let mut out = BTreeMap::new();
        collect_json_placeholders(&value, None, &mut out);
        assert!(out.is_empty());
    }
}
