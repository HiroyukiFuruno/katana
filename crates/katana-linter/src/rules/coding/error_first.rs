use crate::Violation;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

pub struct ErrorFirstVisitor {
    path: PathBuf,
    violations: Vec<Violation>,
}

impl ErrorFirstVisitor {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            violations: Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for ErrorFirstVisitor {
    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        if let syn::Expr::Let(expr_let) = &*node.cond {
            if is_pat_ok(&expr_let.pat) {
                let start = node.if_token.span.start();
                self.violations.push(Violation {
                    file: self.path.clone(),
                    line: start.line,
                    column: start.column,
                    message: "Do not use `if let Ok(...) = ...` to wrap success logic. Follow the Error First principle: handle errors early and return or continue. Use `?` or `let-else` instead.".to_string(),
                });
            }
        }
        syn::visit::visit_expr_if(self, node);
    }
}

fn is_pat_ok(pat: &syn::Pat) -> bool {
    match pat {
        syn::Pat::TupleStruct(pat_tuple_struct) => {
            if let Some(segment) = pat_tuple_struct.path.segments.last() {
                segment.ident == "Ok"
            } else {
                false
            }
        }
        syn::Pat::Ident(pat_ident) => pat_ident.ident == "Ok",
        _ => false,
    }
}

/// Lints a file for violations of the Error First principle (e.g., `if let Ok(...)`).
pub fn lint_error_first(path: &Path, syntax: &syn::File) -> Vec<Violation> {
    let mut visitor = ErrorFirstVisitor::new(path.to_path_buf());
    visitor.visit_file(syntax);
    visitor.violations
}
