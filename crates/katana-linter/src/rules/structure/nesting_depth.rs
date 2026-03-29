use crate::utils::{has_cfg_test_attr, span_location};
use crate::Violation;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

const MAX_NESTING_DEPTH: u32 = 3;

struct NestingDepthVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
    current_depth: u32,
    in_test_context: bool,
    current_fn_name: Option<String>,
}

impl NestingDepthVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
            current_depth: 0,
            in_test_context: false,
            current_fn_name: None,
        }
    }

    fn report_if_exceeded(&mut self, span: proc_macro2::Span) {
        if self.current_depth > MAX_NESTING_DEPTH {
            let (line, column) = span_location(span);
            let fn_ctx = self.current_fn_name.as_deref().unwrap_or("<anonymous>");
            self.violations.push(Violation {
                file: self.file.clone(),
                line,
                column,
                message: format!(
                    "Nesting depth {} in `{fn_ctx}` exceeds the {MAX_NESTING_DEPTH}-level limit. \
                     Extract to a helper method or use early returns.",
                    self.current_depth
                ),
            });
        }
    }
}

impl<'ast> Visit<'ast> for NestingDepthVisitor {
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

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if self.in_test_context || has_cfg_test_attr(&node.attrs) {
            return;
        }
        let prev_name = self.current_fn_name.take();
        self.current_fn_name = Some(node.sig.ident.to_string());
        self.current_depth = 0;
        syn::visit::visit_item_fn(self, node);
        self.current_fn_name = prev_name;
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        if self.in_test_context || has_cfg_test_attr(&node.attrs) {
            return;
        }
        let prev_name = self.current_fn_name.take();
        self.current_fn_name = Some(node.sig.ident.to_string());
        self.current_depth = 0;
        syn::visit::visit_impl_item_fn(self, node);
        self.current_fn_name = prev_name;
    }

    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        self.current_depth += 1;
        use syn::spanned::Spanned;
        self.report_if_exceeded(node.span());
        syn::visit::visit_expr_if(self, node);
        self.current_depth -= 1;
    }

    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        self.current_depth += 1;
        use syn::spanned::Spanned;
        self.report_if_exceeded(node.span());
        syn::visit::visit_expr_match(self, node);
        self.current_depth -= 1;
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        self.current_depth += 1;
        use syn::spanned::Spanned;
        self.report_if_exceeded(node.span());
        syn::visit::visit_expr_for_loop(self, node);
        self.current_depth -= 1;
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        self.current_depth += 1;
        use syn::spanned::Spanned;
        self.report_if_exceeded(node.span());
        syn::visit::visit_expr_while(self, node);
        self.current_depth -= 1;
    }

    fn visit_expr_loop(&mut self, node: &'ast syn::ExprLoop) {
        self.current_depth += 1;
        use syn::spanned::Spanned;
        self.report_if_exceeded(node.span());
        syn::visit::visit_expr_loop(self, node);
        self.current_depth -= 1;
    }
}

pub fn lint_nesting_depth(path: &Path, syntax: &syn::File) -> Vec<Violation> {
    let mut visitor = NestingDepthVisitor::new(path.to_path_buf());
    visitor.visit_file(syntax);
    visitor.violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn passes_shallow_nesting() {
        let code = r#"
            fn foo() {
                if true {
                    if true {
                        if true {
                            let x = 1;
                        }
                    }
                }
            }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_nesting_depth(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn detects_deep_nesting() {
        let code = r#"
            fn foo() {
                if true {
                    if true {
                        if true {
                            if true {
                                let x = 1;
                            }
                        }
                    }
                }
            }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_nesting_depth(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("foo"));
    }

    #[test]
    fn detects_deep_for_loop() {
        let code = r#"
            fn bar() {
                for _ in 0..1 {
                    for _ in 0..1 {
                        for _ in 0..1 {
                            for _ in 0..1 {
                                let x = 1;
                            }
                        }
                    }
                }
            }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_nesting_depth(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn skips_test_functions() {
        let code = r#"
            #[cfg(test)]
            mod tests {
                fn deep_test() {
                    if true {
                        if true {
                            if true {
                                if true { let x = 1; }
                            }
                        }
                    }
                }
            }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_nesting_depth(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }
}
