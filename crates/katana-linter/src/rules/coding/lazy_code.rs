use crate::utils::span_location;
use crate::Violation;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

struct LazyCodeVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl LazyCodeVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for LazyCodeVisitor {
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        let segment = mac
            .path
            .segments
            .last()
            .expect("macro path should contain at least one segment");
        let ident = segment.ident.to_string();
        if ident == "todo" || ident == "unimplemented" || ident == "dbg" {
            let (line, column) = span_location(segment.ident.span());
            self.violations.push(Violation {
                file: self.file.clone(),
                line,
                column,
                message: format!(
                    "Lazy code macro `{}!()` detected. Please implement properly instead of deferring.",
                    ident
                ),
            });
        } else if ident == "println" || ident == "eprintln" {
            let (line, column) = span_location(segment.ident.span());
            self.violations.push(Violation {
                file: self.file.clone(),
                line,
                column,
                message: format!(
                    "Debug output macro `{}!()` detected. Use `tracing::debug!` or `tracing::info!` instead.",
                    ident
                ),
            });
        }
        syn::visit::visit_macro(self, mac);
    }
}

pub fn lint_lazy_code(path: &Path, syntax: &syn::File) -> Vec<Violation> {
    let mut visitor = LazyCodeVisitor::new(path.to_path_buf());
    visitor.visit_file(syntax);
    visitor.violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn detects_todo_macro() {
        let code = r#"fn foo() { todo!(); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_lazy_code(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("todo!()"));
    }

    #[test]
    fn detects_unimplemented_macro() {
        let code = r#"fn foo() { unimplemented!(); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_lazy_code(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("unimplemented!()"));
    }

    #[test]
    fn detects_dbg_macro() {
        let code = r#"fn foo() { dbg!(42); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_lazy_code(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("dbg!()"));
    }

    #[test]
    fn allows_normal_macros() {
        let code = r#"fn foo() { vec![1, 2, 3]; }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_lazy_code(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn detects_println_macro() {
        let code = r#"fn foo() { println!("debug"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_lazy_code(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("println!()"));
    }

    #[test]
    fn detects_eprintln_macro() {
        let code = r#"fn foo() { eprintln!("error debug"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_lazy_code(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("eprintln!()"));
    }
}
