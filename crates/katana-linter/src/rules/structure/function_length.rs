use crate::utils::{has_cfg_test_attr, span_location};
use crate::Violation;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

const MAX_FUNCTION_LINES: usize = 30;

struct FunctionLengthVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
    in_test_context: bool,
}

impl FunctionLengthVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
            in_test_context: false,
        }
    }

    fn check_fn_body(&mut self, name: &str, sig_span: proc_macro2::Span, block: &syn::Block) {
        use syn::spanned::Spanned;
        let start = block.span().start().line;
        let end = block.span().end().line;
        let body_lines = end.saturating_sub(start).saturating_sub(1);

        if body_lines > MAX_FUNCTION_LINES {
            let (line, column) = span_location(sig_span);
            self.violations.push(Violation {
                file: self.file.clone(),
                line,
                column,
                message: format!(
                    "Function `{name}` has {body_lines} lines, exceeding the {MAX_FUNCTION_LINES}-line limit. \
                     Extract helper methods or simplify logic."
                ),
            });
        }
    }
}

impl<'ast> Visit<'ast> for FunctionLengthVisitor {
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
        self.check_fn_body(
            &node.sig.ident.to_string(),
            node.sig.ident.span(),
            &node.block,
        );
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        if self.in_test_context || has_cfg_test_attr(&node.attrs) {
            return;
        }
        self.check_fn_body(
            &node.sig.ident.to_string(),
            node.sig.ident.span(),
            &node.block,
        );
        syn::visit::visit_impl_item_fn(self, node);
    }
}

/// Lints a file for functions that exceed the maximum allowed line count (30 lines).
pub fn lint_function_length(path: &Path, syntax: &syn::File) -> Vec<Violation> {
    let mut visitor = FunctionLengthVisitor::new(path.to_path_buf());
    visitor.visit_file(syntax);
    visitor.violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn passes_short_function() {
        let code = "fn foo() {\n    let x = 1;\n}\n";
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_function_length(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn detects_long_function() {
        let mut code = String::from("fn foo() {\n");
        for i in 0..31 {
            code.push_str(&format!("    let x{i} = {i};\n"));
        }
        code.push_str("}\n");
        let syntax = syn::parse_file(&code).unwrap();
        let violations = lint_function_length(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("foo"));
    }

    #[test]
    fn skips_test_functions() {
        let mut code = String::from("#[cfg(test)]\nmod tests {\n    fn long_test() {\n");
        for i in 0..50 {
            code.push_str(&format!("        let x{i} = {i};\n"));
        }
        code.push_str("    }\n}\n");
        let syntax = syn::parse_file(&code).unwrap();
        let violations = lint_function_length(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn detects_long_impl_method() {
        let mut code = String::from("struct Foo;\nimpl Foo {\n    fn bar(&self) {\n");
        for i in 0..31 {
            code.push_str(&format!("        let x{i} = {i};\n"));
        }
        code.push_str("    }\n}\n");
        let syntax = syn::parse_file(&code).unwrap();
        let violations = lint_function_length(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("bar"));
    }
}
