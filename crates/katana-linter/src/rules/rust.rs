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

// ─────────────────────────────────────────────
// Font Normalization Enforcement Visitor
// ─────────────────────────────────────────────

/// Detects direct usage of `FontDefinitions::default()` or `FontDefinitions::empty()`
/// outside of `font_loader.rs`. All font setup must go through `NormalizeFonts`
/// to ensure consistent CJK baseline alignment.
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
        if self.is_font_loader_file() {
            syn::visit::visit_expr_call(self, node);
            return;
        }

        // Detect FontDefinitions::default() or FontDefinitions::empty()
        if let syn::Expr::Path(path_expr) = &*node.func {
            let segments: Vec<_> = path_expr
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect();

            if segments.len() >= 2 {
                let type_name = &segments[segments.len() - 2];
                let method_name = &segments[segments.len() - 1];

                if type_name == "FontDefinitions"
                    && (method_name == "default" || method_name == "empty")
                {
                    use syn::spanned::Spanned;
                    let (line, column) = span_location(node.span());
                    self.violations.push(Violation {
                        file: self.file.clone(),
                        line,
                        column,
                        message: format!(
                            "Direct `FontDefinitions::{method_name}()` detected. \
                             Use `NormalizeFonts` from `font_loader` module instead \
                             to ensure consistent CJK baseline alignment."
                        ),
                    });
                }
            }
        }
        syn::visit::visit_expr_call(self, node);
    }
}

pub fn lint_font_normalization(path: &Path, syntax: &syn::File) -> Vec<Violation> {
    let mut visitor = FontNormalizationVisitor::new(path.to_path_buf());
    visitor.visit_file(syntax);
    visitor.violations
}

// ─────────────────────────────────────────────
// Performance Enforcement Visitor (CPU Optimization)
// ─────────────────────────────────────────────

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

// ─────────────────────────────────────────────
// Prohibited Attributes Detection Visitor
// ─────────────────────────────────────────────

/// Detects `#[allow(dead_code)]` attributes in production code.
/// Dead code should be deleted, not silenced with attributes.
struct ProhibitedAttributeVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl ProhibitedAttributeVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }

    fn check_attrs(&mut self, attrs: &[syn::Attribute]) {
        for attr in attrs {
            if let syn::Meta::List(meta_list) = &attr.meta {
                if meta_list.path.is_ident("allow") {
                    let tokens = meta_list.tokens.to_string();
                    if tokens.contains("dead_code") {
                        use syn::spanned::Spanned;
                        let (line, column) = span_location(meta_list.path.span());
                        self.violations.push(Violation {
                            file: self.file.clone(),
                            line,
                            column,
                            message: "`#[allow(dead_code)]` detected. \
                                 Dead code should be deleted, not silenced."
                                .to_string(),
                        });
                    }
                }
            }
        }
    }
}

impl<'ast> Visit<'ast> for ProhibitedAttributeVisitor {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if has_cfg_test_attr(&node.attrs) {
            return;
        }
        self.check_attrs(&node.attrs);
        syn::visit::visit_item_mod(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if has_cfg_test_attr(&node.attrs) {
            return;
        }
        self.check_attrs(&node.attrs);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.check_attrs(&node.attrs);
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        self.check_attrs(&node.attrs);
        syn::visit::visit_item_enum(self, node);
    }

    fn visit_item_const(&mut self, node: &'ast syn::ItemConst) {
        self.check_attrs(&node.attrs);
        syn::visit::visit_item_const(self, node);
    }

    fn visit_item_static(&mut self, node: &'ast syn::ItemStatic) {
        self.check_attrs(&node.attrs);
        syn::visit::visit_item_static(self, node);
    }

    fn visit_item_type(&mut self, node: &'ast syn::ItemType) {
        self.check_attrs(&node.attrs);
        syn::visit::visit_item_type(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        self.check_attrs(&node.attrs);
        syn::visit::visit_item_impl(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        if has_cfg_test_attr(&node.attrs) {
            return;
        }
        self.check_attrs(&node.attrs);
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_field(&mut self, node: &'ast syn::Field) {
        self.check_attrs(&node.attrs);
        syn::visit::visit_field(self, node);
    }

    fn visit_variant(&mut self, node: &'ast syn::Variant) {
        self.check_attrs(&node.attrs);
        syn::visit::visit_variant(self, node);
    }

    // extern "C" blocks legitimately need #[allow(dead_code)] because
    // Rust cannot see cross-language (FFI) call sites. Skip entirely.
    fn visit_item_foreign_mod(&mut self, _node: &'ast syn::ItemForeignMod) {}
}

pub fn lint_prohibited_attributes(path: &Path, syntax: &syn::File) -> Vec<Violation> {
    let mut visitor = ProhibitedAttributeVisitor::new(path.to_path_buf());
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
        let code = r#"fn foo() { vec![1, 2, 3]; }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_lazy_code(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn lint_lazy_code_detects_println_macro() {
        let code = r#"fn foo() { println!("debug"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_lazy_code(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("println!()"));
    }

    #[test]
    fn lint_lazy_code_detects_eprintln_macro() {
        let code = r#"fn foo() { eprintln!("error debug"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_lazy_code(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("eprintln!()"));
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

    #[test]
    fn lint_font_normalization_detects_raw_font_definitions_default() {
        let code = r#"fn setup() { let f = FontDefinitions::default(); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_font_normalization(&PathBuf::from("shell.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("NormalizeFonts"));
    }

    #[test]
    fn lint_font_normalization_detects_raw_font_definitions_empty() {
        let code = r#"fn setup() { let f = FontDefinitions::empty(); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_font_normalization(&PathBuf::from("shell.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("NormalizeFonts"));
    }

    #[test]
    fn lint_font_normalization_allows_in_font_loader() {
        let code = r#"fn build() { let f = FontDefinitions::default(); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_font_normalization(&PathBuf::from("font_loader.rs"), &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn lint_font_normalization_skips_test_code() {
        let code = r#"
            #[cfg(test)]
            fn test_foo() { let f = FontDefinitions::default(); }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_font_normalization(&PathBuf::from("shell.rs"), &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn lint_performance_detects_unconditional_repaint() {
        let code = r#"fn update() { ctx.request_repaint(); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_performance(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("request_repaint"));
    }

    #[test]
    fn lint_performance_allows_conditional_repaint() {
        let code = r#"fn update() { if true { ctx.request_repaint(); } }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_performance(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn lint_performance_detects_unconditional_set_title() {
        let code = r#"fn update() { window.set_title("foo"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_performance(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("set_title"));
    }

    #[test]
    fn lint_performance_allows_conditional_set_title() {
        let code = r#"fn update() { if title != last { window.set_title(title); } }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_performance(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn lint_prohibited_attributes_detects_allow_dead_code_on_fn() {
        let code = r#"
            #[allow(dead_code)]
            fn unused() {}
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_prohibited_attributes(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("dead_code"));
    }

    #[test]
    fn lint_prohibited_attributes_detects_allow_dead_code_on_struct() {
        let code = r#"
            #[allow(dead_code)]
            struct Unused;
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_prohibited_attributes(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn lint_prohibited_attributes_detects_allow_dead_code_on_field() {
        let code = r#"
            struct Foo {
                #[allow(dead_code)]
                bar: i32,
            }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_prohibited_attributes(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn lint_prohibited_attributes_detects_allow_dead_code_on_enum_variant() {
        let code = r#"
            enum Foo {
                #[allow(dead_code)]
                Bar,
            }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_prohibited_attributes(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn lint_prohibited_attributes_skips_test_code() {
        let code = r#"
            #[cfg(test)]
            mod tests {
                #[allow(dead_code)]
                fn helper() {}
            }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_prohibited_attributes(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn lint_prohibited_attributes_allows_other_allow_attrs() {
        let code = r#"
            #[allow(unused_imports)]
            use std::io;
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_prohibited_attributes(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn lint_prohibited_attributes_detects_allow_dead_code_on_impl_method() {
        let code = r#"
            struct Foo;
            impl Foo {
                #[allow(dead_code)]
                fn unused_method(&self) {}
            }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_prohibited_attributes(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn lint_prohibited_attributes_skips_extern_blocks() {
        let code = r#"
            #[allow(dead_code)]
            extern "C" {
                fn some_ffi_function();
            }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_prohibited_attributes(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }
}
