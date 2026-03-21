use crate::utils::{has_cfg_test_attr, is_allowed_number, span_location};
use crate::Violation;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

// ─────────────────────────────────────────────
// Magic Number Detection Visitor
// ─────────────────────────────────────────────

struct MagicNumberVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
    /// Nesting depth of being inside a const/static declaration.
    /// If greater than 0, numeric literals are inside a named constant and thus allowed.
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
                if !is_allowed_number(value as f64) {
                    let (line, column) = span_location(lit_int.span());
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
            syn::Lit::Float(lit_float) => {
                let value = lit_float
                    .base10_parse::<f64>()
                    .expect("syn::LitFloat should always be parseable");
                if !is_allowed_number(value) {
                    let (line, column) = span_location(lit_float.span());
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
            _ => {}
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

// ─────────────────────────────────────────────
// Prohibited Types Detection Visitor
// ─────────────────────────────────────────────

struct ProhibitedTypesVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
    in_const_context: u32,
}

impl ProhibitedTypesVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
            in_const_context: 0,
        }
    }
}

impl<'ast> Visit<'ast> for ProhibitedTypesVisitor {
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

    fn visit_impl_item_const(&mut self, node: &'ast syn::ImplItemConst) {
        self.in_const_context += 1;
        syn::visit::visit_impl_item_const(self, node);
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

    fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
        if self.in_const_context > 0 {
            syn::visit::visit_type_path(self, node);
            return;
        }
        let segment = node
            .path
            .segments
            .last()
            .expect("type path should contain at least one segment");
        if segment.ident == "HashMap" {
            let (line, column) = span_location(segment.ident.span());
            self.violations.push(Violation {
                file: self.file.clone(),
                line,
                column,
                message:
                    "Prohibited type `HashMap` detected. Please use `Vec` or a typed struct instead."
                        .to_string(),
            });
        }
        syn::visit::visit_type_path(self, node);
    }

    fn visit_type_array(&mut self, node: &'ast syn::TypeArray) {
        if self.in_const_context > 0 {
            syn::visit::visit_type_array(self, node);
            return;
        }
        use syn::spanned::Spanned;
        let (line, column) = span_location(node.span());
        self.violations.push(Violation {
            file: self.file.clone(),
            line,
            column,
            message: "Fixed-length array `[T; N]` detected. Please use `Vec<T>` instead."
                .to_string(),
        });
        syn::visit::visit_type_array(self, node);
    }

    fn visit_expr_array(&mut self, node: &'ast syn::ExprArray) {
        if self.in_const_context > 0 {
            syn::visit::visit_expr_array(self, node);
            return;
        }

        // Allow small array literals (e.g. for UI sizes or i18n params)
        if node.elems.len() <= 8 {
            syn::visit::visit_expr_array(self, node);
            return;
        }

        use syn::spanned::Spanned;
        let (line, column) = span_location(node.span());
        self.violations.push(Violation {
            file: self.file.clone(),
            line,
            column,
            message: format!(
                "Array literal `[...]` with {} elements detected. Please use `vec![...]` instead for large arrays.",
                node.elems.len()
            ),
        });
        syn::visit::visit_expr_array(self, node);
    }
}

pub fn lint_prohibited_types(path: &Path, syntax: &syn::File) -> Vec<Violation> {
    let mut visitor = ProhibitedTypesVisitor::new(path.to_path_buf());
    visitor.visit_file(syntax);
    visitor.violations
}

// ─────────────────────────────────────────────
// Lazy Code Detection Visitor
// ─────────────────────────────────────────────

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
    fn lint_magic_numbers_detects_literal_in_function() {
        let code = r#"fn foo() -> f32 { let x: f32 = 42.0; x }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
        assert!(violations[0].to_string().contains("42"));
    }

    #[test]
    fn lint_magic_numbers_allows_literal_in_const() {
        let code = r#"const FOO: f32 = 42.0;"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn lint_magic_numbers_allows_common_values() {
        let code = r#"fn foo() { let a = 0; let b = 1; let c = 2; let d = 100; }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn lint_magic_numbers_skips_test_functions() {
        let code = r#"
            #[cfg(test)]
            fn test_foo() -> i32 { 42 }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn lint_magic_numbers_skips_test_impl_methods() {
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
    fn lint_magic_numbers_detects_int_literal() {
        let code = r#"fn foo() -> i32 { let x = 42; x }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
        assert!(violations[0].to_string().contains("42"));
    }

    #[test]
    fn lint_magic_numbers_allows_negative_one() {
        let code = r#"fn foo() -> i32 { -1 }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn lint_magic_numbers_allows_static_context() {
        let code = r#"static FOO: i32 = 42;"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn lint_magic_numbers_allows_impl_item_const() {
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
    fn lint_magic_numbers_allows_zero_float() {
        let code = r#"fn foo() { let _ = 0.0; }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn lint_magic_numbers_allows_one_float() {
        let code = r#"fn foo() { let _ = 1.0; }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn lint_lazy_code_detects_todo_macro() {
        let code = r#"fn foo() { todo!(); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_lazy_code(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("todo!()"));
    }

    #[test]
    fn lint_lazy_code_detects_unimplemented_macro() {
        let code = r#"fn foo() { unimplemented!(); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_lazy_code(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("unimplemented!()"));
    }

    #[test]
    fn lint_lazy_code_detects_dbg_macro() {
        let code = r#"fn foo() { dbg!(42); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_lazy_code(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("dbg!()"));
    }

    #[test]
    fn lint_lazy_code_allows_normal_macros() {
        let code = r#"fn foo() { println!("ok"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_lazy_code(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn lint_prohibited_types_detects_hashmap() {
        let code = r#"fn foo() { let map: std::collections::HashMap<i32, i32> = std::collections::HashMap::new(); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_prohibited_types(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.iter().any(|v| v.message.contains("HashMap")));
    }

    #[test]
    fn lint_prohibited_types_detects_type_array() {
        let code = r#"fn foo() { let arr: [i32; 4] = [1, 2, 3, 4]; }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_prohibited_types(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations
            .iter()
            .any(|v| v.message.contains("Fixed-length array")));
    }

    #[test]
    fn lint_prohibited_types_detects_expr_array() {
        let code = r#"fn foo() { let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9]; }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_prohibited_types(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations
            .iter()
            .any(|v| v.message.contains("Array literal")));
    }
}
