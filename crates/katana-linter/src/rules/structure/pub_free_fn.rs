use crate::utils::{has_cfg_test_attr, span_location};
use crate::Violation;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

struct PubFreeFnVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
    in_impl: bool,
    in_test_context: bool,
}

impl PubFreeFnVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
            in_impl: false,
            in_test_context: false,
        }
    }

    fn is_main_fn(node: &syn::ItemFn) -> bool {
        node.sig.ident == "main"
    }
}

impl<'ast> Visit<'ast> for PubFreeFnVisitor {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if has_cfg_test_attr(&node.attrs) {
            let prev = self.in_test_context;
            self.in_test_context = true;
            syn::visit::visit_item_mod(self, node);
            self.in_test_context = prev;
            return;
        }
        syn::visit::visit_item_mod(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        let prev = self.in_impl;
        self.in_impl = true;
        syn::visit::visit_item_impl(self, node);
        self.in_impl = prev;
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if self.in_test_context || self.in_impl {
            syn::visit::visit_item_fn(self, node);
            return;
        }

        if has_cfg_test_attr(&node.attrs) || Self::is_main_fn(node) {
            return;
        }

        let is_pub = matches!(node.vis, syn::Visibility::Public(_))
            || matches!(&node.vis, syn::Visibility::Restricted(r) if r.path.is_ident("crate"));

        if is_pub {
            let (line, column) = span_location(node.sig.ident.span());
            self.violations.push(Violation {
                file: self.file.clone(),
                line,
                column,
                message: format!(
                    "Public free function `{}` detected at module level. \
                     Domain logic should be in `struct` + `impl` blocks (coding-rules §1.1).",
                    node.sig.ident
                ),
            });
        }

        syn::visit::visit_item_fn(self, node);
    }
}

pub fn lint_pub_free_fn(path: &Path, syntax: &syn::File) -> Vec<Violation> {
    let mut visitor = PubFreeFnVisitor::new(path.to_path_buf());
    visitor.visit_file(syntax);
    visitor.violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn detects_pub_free_fn() {
        let code = r#"pub fn helper() {}"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_pub_free_fn(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("helper"));
    }

    #[test]
    fn detects_pub_crate_free_fn() {
        let code = r#"pub(crate) fn internal_helper() {}"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_pub_free_fn(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn allows_private_free_fn() {
        let code = r#"fn private_helper() {}"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_pub_free_fn(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn allows_main_fn() {
        let code = r#"pub fn main() {}"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_pub_free_fn(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn allows_impl_method() {
        let code = r#"
            struct Foo;
            impl Foo {
                pub fn method(&self) {}
            }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_pub_free_fn(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn skips_test_module() {
        let code = r#"
            #[cfg(test)]
            mod tests {
                pub fn test_helper() {}
            }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_pub_free_fn(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }
}
