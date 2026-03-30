use crate::utils::{has_cfg_test_attr, span_location};
use crate::Violation;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

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
            let syn::Meta::List(meta_list) = &attr.meta else {
                continue;
            };
            if !meta_list.path.is_ident("allow") {
                continue;
            }
            let tokens = meta_list.tokens.to_string();
            if !tokens.contains("dead_code") {
                continue;
            }
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

    // WHY: extern "C" blocks need #[allow(dead_code)] — Rust cannot see FFI call sites
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
    fn detects_allow_dead_code_on_fn() {
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
    fn detects_allow_dead_code_on_struct() {
        let code = r#"
            #[allow(dead_code)]
            struct Unused;
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_prohibited_attributes(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn detects_allow_dead_code_on_field() {
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
    fn detects_allow_dead_code_on_enum_variant() {
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
    fn skips_test_code() {
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
    fn allows_other_allow_attrs() {
        let code = r#"
            #[allow(unused_imports)]
            use std::io;
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_prohibited_attributes(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn detects_allow_dead_code_on_impl_method() {
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
    fn skips_extern_blocks() {
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
