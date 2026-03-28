#![deny(
    warnings,
    dead_code,
    clippy::too_many_lines,
    clippy::cognitive_complexity,
    clippy::wildcard_imports,
    clippy::unwrap_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented
)]
#![warn(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::missing_errors_doc,
    missing_docs
)]

pub mod rules;
pub mod utils;

use serde_json::Value;
use std::path::{Path, PathBuf};

// ─────────────────────────────────────────────
// Violation Report
// ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Violation {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "  {}:{}:{} — {}",
            self.file.display(),
            self.line,
            self.column,
            self.message
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonNodeKind {
    Object,
    Array(usize),
    String,
    Number,
    Bool,
    Null,
}

impl JsonNodeKind {
    pub fn from_value(value: &Value) -> Self {
        match value {
            Value::Object(_) => Self::Object,
            Value::Array(items) => Self::Array(items.len()),
            Value::String(_) => Self::String,
            Value::Number(_) => Self::Number,
            Value::Bool(_) => Self::Bool,
            Value::Null => Self::Null,
        }
    }
}

impl std::fmt::Display for JsonNodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Object => write!(f, "object"),
            Self::Array(len) => write!(f, "array(len={len})"),
            Self::String => write!(f, "string"),
            Self::Number => write!(f, "number"),
            Self::Bool => write!(f, "bool"),
            Self::Null => write!(f, "null"),
        }
    }
}

/// Common execution logic for AST Lint.
pub fn run_ast_lint(
    rule_name: &str,
    hint: &str,
    target_dirs: &[PathBuf],
    lint_fn: fn(&Path, &syn::File) -> Vec<Violation>,
) {
    let mut all_violations: Vec<Violation> = Vec::new();

    for target_dir in target_dirs {
        let rs_files = utils::collect_rs_files(target_dir);
        assert!(
            !rs_files.is_empty(),
            "No .rs files found for analysis: {}",
            target_dir.display()
        );

        for file in &rs_files {
            match utils::parse_file(file) {
                Ok(syntax) => {
                    let violations = lint_fn(file, &syntax);
                    all_violations.extend(violations);
                }
                Err(errors) => {
                    all_violations.extend(errors);
                }
            }
        }
    }
    utils::panic_with_violations(rule_name, hint, &all_violations);
}
