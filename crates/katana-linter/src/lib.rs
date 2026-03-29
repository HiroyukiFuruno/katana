#![deny(warnings, clippy::all)]
#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::too_many_lines,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unwrap_or_default,
    clippy::wildcard_imports,
    clippy::match_wild_err_arm,
    clippy::let_and_return,
    clippy::manual_ok_err,
    clippy::cognitive_complexity
)]
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::panic,
        clippy::expect_used,
        clippy::indexing_slicing
    )
)]

pub mod rules;
pub mod utils;

use serde_json::Value;
use std::path::{Path, PathBuf};

// ─────────────────────────────────────────────
// WHY: Violation Report
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

pub fn run_ast_lint(
    rule_name: &str,
    hint: &str,
    target_dirs: &[PathBuf],
    lint_fn: fn(&Path, &syn::File) -> Vec<Violation>,
) {
    let mut all_violations: Vec<Violation> = Vec::new();

    for target_dir in target_dirs {
        for file in &utils::collect_rs_files(target_dir) {
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

    if let Err(e) = utils::format_violations(rule_name, hint, &all_violations) {
        /* WHY: The AST Linter executes as part of the test suite boundary.
        Failing tests fundamentally require panics to communicate failure natively. */
        #[allow(clippy::panic)]
        {
            panic!("{e}");
        }
    }
}
