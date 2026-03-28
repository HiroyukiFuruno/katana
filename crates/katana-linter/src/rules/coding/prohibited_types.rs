use crate::utils::{has_cfg_test_attr, span_location};
use crate::Violation;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

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

        // WHY: small array literals are used for UI sizes or i18n params
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn detects_hashmap() {
        let code = r#"fn foo() { let map: std::collections::HashMap<i32, i32> = std::collections::HashMap::new(); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_prohibited_types(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.iter().any(|v| v.message.contains("HashMap")));
    }

    #[test]
    fn detects_type_array() {
        let code = r#"fn foo() { let arr: [i32; 4] = [1, 2, 3, 4]; }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_prohibited_types(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations
            .iter()
            .any(|v| v.message.contains("Fixed-length array")));
    }

    #[test]
    fn detects_expr_array() {
        let code = r#"fn foo() { let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9]; }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_prohibited_types(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations
            .iter()
            .any(|v| v.message.contains("Array literal")));
    }
}
