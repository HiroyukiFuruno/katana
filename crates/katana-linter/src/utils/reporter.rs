use crate::Violation;
use std::path::Path;

/// Creates a locale parsing violation for the provided file.
pub fn locale_violation(file: &Path, message: impl Into<String>) -> Violation {
    Violation {
        file: file.to_path_buf(),
        line: 0,
        column: 0,
        message: message.into(),
    }
}

/// Formats an error report if any violations exist.
/// Intended specifically for ast_linter integration tests to bubble up errors.
///
/// # Errors
/// Returns an `Err` containing the formatted assertion report if violations are present.
pub fn format_violations(
    rule_name: &str,
    hint: &str,
    violations: &[Violation],
) -> Result<(), String> {
    if violations.is_empty() {
        return Ok(());
    }

    let report = violations
        .iter()
        .map(|it| it.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    Err(format!(
        "\n\n🚨 AST Linter [{rule_name}]: Found {} violation(s):\n\n{}\n\n\
        💡 {hint}\n\
        📖 Details: See docs/coding-rules.md\n",
        violations.len(),
        report
    ))
}
