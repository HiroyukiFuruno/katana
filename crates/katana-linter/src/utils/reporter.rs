use crate::Violation;
use std::path::Path;

pub fn locale_violation(file: &Path, message: impl Into<String>) -> Violation {
    Violation {
        file: file.to_path_buf(),
        line: 0,
        column: 0,
        message: message.into(),
    }
}

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

pub fn panic_with_violations(rule_name: &str, hint: &str, violations: &[Violation]) {
    if let Err(e) = format_violations(rule_name, hint, violations) {
        #[allow(clippy::panic)]
        {
            panic!("{e}");
        }
    }
}
