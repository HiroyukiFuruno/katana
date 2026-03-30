use crate::Violation;
use std::path::Path;

pub mod discovery;
pub mod structure;

pub fn lint_markdown_heading_pairs(root: &Path) -> Vec<Violation> {
    let mut violations = Vec::new();
    for pair in discovery::collect_markdown_pairs(root) {
        violations.extend(structure::compare_markdown_heading_structure(&pair));
    }
    violations
}
