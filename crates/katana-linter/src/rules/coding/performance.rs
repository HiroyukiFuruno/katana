use crate::utils::span_location;
use crate::Violation;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

/// Detects potentially unoptimized UI update patterns.
/// Specifically:
/// 1. Unconditional `request_repaint()` calls in `update` loops.
/// 2. Unconditional `window.set_title()` calls.
struct PerformanceVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
    /// Nesting depth of being inside a conditional block (if/match).
    in_conditional: u32,
}

impl PerformanceVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
            in_conditional: 0,
        }
    }
}

impl<'ast> Visit<'ast> for PerformanceVisitor {
    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        self.in_conditional += 1;
        syn::visit::visit_expr_if(self, node);
        self.in_conditional -= 1;
    }

    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        self.in_conditional += 1;
        syn::visit::visit_expr_match(self, node);
        self.in_conditional -= 1;
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if self.in_conditional == 0 {
            let method_name = node.method.to_string();
            if method_name == "request_repaint" {
                use syn::spanned::Spanned;
                let (line, column) = span_location(node.span());
                self.violations.push(Violation {
                    file: self.file.clone(),
                    line,
                    column,
                    message: "Unconditional `request_repaint()` detected. \
                              This can cause 100% CPU usage. \
                              Please wrap in a condition or use `request_repaint_after()`."
                        .to_string(),
                });
            } else if method_name == "set_title" {
                use syn::spanned::Spanned;
                let (line, column) = span_location(node.span());
                self.violations.push(Violation {
                    file: self.file.clone(),
                    line,
                    column,
                    message: "Unconditional `set_title()` detected. \
                              This causes excessive system calls. \
                              Please wrap in a check (e.g. `if title != last_title { ... }`)."
                        .to_string(),
                });
            }
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

pub fn lint_performance(path: &Path, syntax: &syn::File) -> Vec<Violation> {
    let mut visitor = PerformanceVisitor::new(path.to_path_buf());
    visitor.visit_file(syntax);
    visitor.violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn detects_unconditional_repaint() {
        let code = r#"fn update() { ctx.request_repaint(); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_performance(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("request_repaint"));
    }

    #[test]
    fn allows_conditional_repaint() {
        let code = r#"fn update() { if true { ctx.request_repaint(); } }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_performance(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn detects_unconditional_set_title() {
        let code = r#"fn update() { window.set_title("foo"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_performance(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("set_title"));
    }

    #[test]
    fn allows_conditional_set_title() {
        let code = r#"fn update() { if title != last { window.set_title(title); } }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_performance(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }
}
