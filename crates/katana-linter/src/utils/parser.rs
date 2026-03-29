use crate::Violation;
use std::path::Path;

/// Checks if a syn attribute contains `#[cfg(test)]`.
pub fn has_cfg_test_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("cfg") {
            let Ok(syn::Meta::Path(path)) = attr.parse_args::<syn::Meta>() else {
                return false;
            };
            return path.is_ident("test");
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
}
