use crate::utils::{has_cfg_test_attr, is_allowed_number, span_location};
use crate::Violation;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

struct MagicNumberVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
    in_const_context: u32,
}

impl MagicNumberVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
            in_const_context: 0,
        }
    }

    fn check_lit(&mut self, lit: &syn::Lit) {
        if self.in_const_context > 0 {
            return;
        }
        match lit {
            syn::Lit::Int(lit_int) => {
                let value = lit_int
                    .base10_parse::<i64>()
                    .expect("syn::LitInt should always be parseable");
                self.check_numeric_value(value as f64, lit_int.span());
            }
            syn::Lit::Float(lit_float) => {
                let value = lit_float
                    .base10_parse::<f64>()
                    .expect("syn::LitFloat should always be parseable");
                self.check_numeric_value(value, lit_float.span());
            }
            _ => {}
        }
    }

    fn check_numeric_value(&mut self, value: f64, span: proc_macro2::Span) {
        if !is_allowed_number(value) {
            let (line, column) = span_location(span);
            self.violations.push(Violation {
                file: self.file.clone(),
                line,
                column,
                message: format!(
                    "Magic number {value} detected. Please extract to a named constant."
                ),
            });
        }
    }
}

impl<'ast> Visit<'ast> for MagicNumberVisitor {
    fn visit_item_const(&mut self, node: &'ast syn::ItemConst) {
        self.in_const_context += 1;
        syn::visit::visit_item_const(self, node);
        self.in_const_context -= 1;
    }

    fn visit_item_static(&mut self, node: &'ast syn::ItemStatic) {
        self.in_const_context += 1;
        syn::visit::visit_item_static(self, node);
        self.in_const_context -= 1;
    }

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

    fn visit_impl_item_const(&mut self, node: &'ast syn::ImplItemConst) {
        self.in_const_context += 1;
        syn::visit::visit_impl_item_const(self, node);
        self.in_const_context -= 1;
    }

    fn visit_local(&mut self, node: &'ast syn::Local) {
        syn::visit::visit_local(self, node);
    }

    fn visit_expr_lit(&mut self, node: &'ast syn::ExprLit) {
        self.check_lit(&node.lit);
        syn::visit::visit_expr_lit(self, node);
    }
}

pub fn lint_magic_numbers(path: &Path, syntax: &syn::File) -> Vec<Violation> {
    let mut visitor = MagicNumberVisitor::new(path.to_path_buf());
    visitor.visit_file(syntax);
    visitor.violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn detects_literal_in_function() {
        let code = r#"fn foo() -> f32 { let x: f32 = 42.0; x }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
        assert!(violations[0].to_string().contains("42"));
    }

    #[test]
    fn allows_literal_in_const() {
        let code = r#"const FOO: f32 = 42.0;"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn allows_common_values() {
        let code = r#"fn foo() { let a = 0; let b = 1; let c = 2; let d = 100; }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn skips_test_functions() {
        let code = r#"
            #[cfg(test)]
            fn test_foo() -> i32 { 42 }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn skips_test_impl_methods() {
        let code = r#"
            impl Foo {
                #[cfg(test)]
                fn test_foo_method() -> i32 { 42 }
            }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn detects_int_literal() {
        let code = r#"fn foo() -> i32 { let x = 42; x }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
        assert!(violations[0].to_string().contains("42"));
    }

    #[test]
    fn allows_negative_one() {
        let code = r#"fn foo() -> i32 { -1 }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn allows_static_context() {
        let code = r#"static FOO: i32 = 42;"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn allows_impl_item_const() {
        let code = r#"
            struct Foo;
            impl Foo {
                const BAR: f32 = 14.0;
            }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn allows_zero_float() {
        let code = r#"fn foo() { let _ = 0.0; }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn allows_one_float() {
        let code = r#"fn foo() { let _ = 1.0; }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }
}
