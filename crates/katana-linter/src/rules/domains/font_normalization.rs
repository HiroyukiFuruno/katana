use crate::utils::{has_cfg_test_attr, span_location};
use crate::Violation;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

struct FontNormalizationVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl FontNormalizationVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }

    fn is_font_loader_file(&self) -> bool {
        self.file
            .to_str()
            .is_some_and(|p| p.contains("font_loader"))
    }
}

impl<'ast> Visit<'ast> for FontNormalizationVisitor {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if has_cfg_test_attr(&node.attrs) {
            return;
        }
        syn::visit::visit_item_mod(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if has_cfg_test_attr(&node.attrs) {
            return;
        }
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        if has_cfg_test_attr(&node.attrs) {
            return;
        }
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if !self.is_font_loader_file() {
            self.check_font_definitions_call(node);
        }
        syn::visit::visit_expr_call(self, node);
    }
}

impl FontNormalizationVisitor {
    fn check_font_definitions_call(&mut self, node: &syn::ExprCall) {
        let syn::Expr::Path(path_expr) = &*node.func else {
            return;
        };
        let segments: Vec<_> = path_expr
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect();
        if segments.len() < 2 {
            return;
        }
        let type_name = &segments[segments.len() - 2];
        let method_name = &segments[segments.len() - 1];
        if type_name != "FontDefinitions" || (method_name != "default" && method_name != "empty") {
            return;
        }
        use syn::spanned::Spanned;
        let (line, column) = span_location(node.span());
        self.violations.push(Violation {
            file: self.file.clone(),
            line,
            column,
            message: format!(
                "Direct `FontDefinitions::{method_name}()` detected. \
                 Use `NormalizeFonts` from `font_loader` module instead."
            ),
        });
    }
}

pub fn lint_font_normalization(path: &Path, syntax: &syn::File) -> Vec<Violation> {
    let mut visitor = FontNormalizationVisitor::new(path.to_path_buf());
    visitor.visit_file(syntax);
    visitor.violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn detects_raw_font_definitions_default() {
        let code = r#"fn setup() { let f = FontDefinitions::default(); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_font_normalization(&PathBuf::from("shell.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("NormalizeFonts"));
    }

    #[test]
    fn detects_raw_font_definitions_empty() {
        let code = r#"fn setup() { let f = FontDefinitions::empty(); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_font_normalization(&PathBuf::from("shell.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("NormalizeFonts"));
    }

    #[test]
    fn allows_in_font_loader() {
        let code = r#"fn build() { let f = FontDefinitions::default(); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_font_normalization(&PathBuf::from("font_loader.rs"), &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn skips_test_code() {
        let code = r#"
            #[cfg(test)]
            fn test_foo() { let f = FontDefinitions::default(); }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_font_normalization(&PathBuf::from("shell.rs"), &syntax);
        assert!(violations.is_empty());
    }
}
