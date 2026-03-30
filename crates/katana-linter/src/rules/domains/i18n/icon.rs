use crate::utils::{has_cfg_test_attr, span_location};
use crate::Violation;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

use super::helpers::{
    extract_type_from_call, is_raw_icon, ui_functions, ui_methods, ui_types_for_new,
};

struct IconFacadeVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl IconFacadeVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }

    fn check_expr_for_raw_icon(&mut self, expr: &syn::Expr, context: &str) {
        match expr {
            syn::Expr::Lit(expr_lit) => {
                if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                    let value = lit_str.value();
                    if is_raw_icon(&value) {
                        let (line, column) = span_location(lit_str.span());
                        self.violations.push(Violation {
                            file: self.file.clone(),
                            line,
                            column,
                            message: format!(
                                "Raw icon string \"{value}\" detected in {context}. \
                                 Please use `Icon::Name.as_str()` instead."
                            ),
                        });
                    }
                }
            }
            syn::Expr::Reference(expr_ref) => self.check_expr_for_raw_icon(&expr_ref.expr, context),
            syn::Expr::Paren(expr_paren) => self.check_expr_for_raw_icon(&expr_paren.expr, context),
            syn::Expr::Group(expr_group) => self.check_expr_for_raw_icon(&expr_group.expr, context),
            _ => {}
        }
    }
}

impl<'ast> Visit<'ast> for IconFacadeVisitor {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if has_cfg_test_attr(&node.attrs) {
            return;
        }
        syn::visit::visit_item_mod(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method_name = node.method.to_string();
        if ui_methods().contains(&method_name.as_str()) {
            for arg in node.args.iter() {
                self.check_expr_for_raw_icon(arg, &format!("{}()", method_name));
            }
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(expr_path) = &*node.func {
            if let Some(last_segment) = expr_path.path.segments.last() {
                let func_name = last_segment.ident.to_string();
                if ui_functions().contains(&func_name.as_str()) {
                    if let Some(type_name) = extract_type_from_call(&node.func) {
                        if ui_types_for_new().contains(&type_name.as_str()) {
                            for arg in node.args.iter() {
                                self.check_expr_for_raw_icon(
                                    arg,
                                    &format!("{}::{}", type_name, func_name),
                                );
                            }
                        }
                    }
                }
            }
        }
        syn::visit::visit_expr_call(self, node);
    }
}

pub fn lint_icon_facade(path: &Path, syntax: &syn::File) -> Vec<Violation> {
    let mut visitor = IconFacadeVisitor::new(path.to_path_buf());
    visitor.visit_file(syntax);
    visitor.violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lint_i18n_detects_raw_icon_in_label() {
        let code = r#"fn render(ui: &mut Ui) { ui.label("🔄"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_icon_facade(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
        assert!(violations[0].message.contains("Raw icon string"));
    }

    #[test]
    fn lint_i18n_detects_raw_icon_x_in_button() {
        let code = r#"fn render(ui: &mut Ui) { ui.button("x"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_icon_facade(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
    }
}
