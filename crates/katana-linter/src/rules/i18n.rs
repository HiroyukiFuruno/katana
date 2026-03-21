use crate::utils::{has_cfg_test_attr, is_allowed_string, is_emoji_or_symbol, span_location};
use crate::Violation;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

// ─────────────────────────────────────────────
// i18n Hardcoded String Detection Visitor
// ─────────────────────────────────────────────

/// List of UI method names to inspect.
fn ui_methods() -> Vec<&'static str> {
    vec![
        "label",
        "heading",
        "button",
        "on_hover_text",
        "selectable_label",
        "checkbox",
        "radio",
        "radio_value",
        "small_button",
        "text_edit_singleline",
        "hyperlink_to",
        "collapsing",
    ]
}

/// List of function calls (`Type::func()` format) to inspect.
fn ui_functions() -> Vec<&'static str> {
    vec!["new"]
}

/// Target type names for function calls.
fn ui_types_for_new() -> Vec<&'static str> {
    vec!["RichText", "Button"]
}

struct I18nHardcodeVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl I18nHardcodeVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }

    /// Detect hardcoded string literals from an argument list.
    fn check_string_literal_args(
        &mut self,
        args: &syn::punctuated::Punctuated<syn::Expr, syn::token::Comma>,
        method_name: &str,
    ) {
        for arg in args.iter() {
            self.check_expr_for_hardcoded_string(arg, method_name);
        }
    }

    /// Recursively check if an expression is a hardcoded string.
    fn check_expr_for_hardcoded_string(&mut self, expr: &syn::Expr, method_name: &str) {
        match expr {
            syn::Expr::Lit(expr_lit) => {
                if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                    let value = lit_str.value();
                    if !is_allowed_string(&value) {
                        let (line, column) = span_location(lit_str.span());
                        self.violations.push(Violation {
                            file: self.file.clone(),
                            line,
                            column,
                            message: format!(
                                "Hardcoded string \"{value}\" detected in {method_name}().\
                                 Please use i18n::t() or i18n::tf()."
                            ),
                        });
                    }
                }
            }
            syn::Expr::Macro(expr_macro) => {
                if is_format_macro(&expr_macro.mac) {
                    let (line, column) = span_location(
                        expr_macro
                            .mac
                            .path
                            .segments
                            .last()
                            .map(|it| it.ident.span())
                            .unwrap_or_else(proc_macro2::Span::call_site),
                    );
                    self.violations.push(Violation {
                        file: self.file.clone(),
                        line,
                        column,
                        message: format!(
                            "Hardcoded string synthesis using format!() detected in {method_name}().\
                             Please use i18n::tf()."
                        ),
                    });
                }
            }
            syn::Expr::Reference(expr_ref) => {
                self.check_expr_for_hardcoded_string(&expr_ref.expr, method_name);
            }
            syn::Expr::Paren(expr_paren) => {
                self.check_expr_for_hardcoded_string(&expr_paren.expr, method_name);
            }
            syn::Expr::Group(expr_group) => {
                self.check_expr_for_hardcoded_string(&expr_group.expr, method_name);
            }
            _ => {}
        }
    }

    fn check_call_for_ui_violation(&mut self, node: &syn::ExprCall) {
        let syn::Expr::Path(expr_path) = &*node.func else {
            return;
        };
        let last_segment = expr_path
            .path
            .segments
            .last()
            .expect("syn::Path always has at least one segment");
        let func_name = last_segment.ident.to_string();
        if !ui_functions().contains(&func_name.as_str()) {
            return;
        }
        let Some(type_name) = extract_type_from_call(&node.func) else {
            return;
        };
        if !ui_types_for_new().contains(&type_name.as_str()) {
            return;
        }
        self.check_string_literal_args(&node.args, &format!("{type_name}::{func_name}"));
    }
}

fn is_format_macro(mac: &syn::Macro) -> bool {
    mac.path
        .segments
        .last()
        .map(|it| it.ident == "format")
        .unwrap_or(false)
}

fn extract_type_from_call(func: &syn::Expr) -> Option<String> {
    if let syn::Expr::Path(expr_path) = func {
        let segments = &expr_path.path.segments;
        if segments.len() >= 2 {
            return Some(segments[segments.len() - 2].ident.to_string());
        }
    }
    None
}

impl<'ast> Visit<'ast> for I18nHardcodeVisitor {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if has_cfg_test_attr(&node.attrs) {
            return;
        }
        syn::visit::visit_item_mod(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method_name = node.method.to_string();
        if ui_methods().contains(&method_name.as_str()) {
            self.check_string_literal_args(&node.args, &method_name);
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        self.check_call_for_ui_violation(node);
        syn::visit::visit_expr_call(self, node);
    }
}

pub fn lint_i18n(path: &Path, syntax: &syn::File) -> Vec<Violation> {
    let mut visitor = I18nHardcodeVisitor::new(path.to_path_buf());
    visitor.visit_file(syntax);
    visitor.violations
}

// ─────────────────────────────────────────────
// Icon Facade Detection Visitor
// ─────────────────────────────────────────────

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

// ─────────────────────────────────────────────
// Allowlist Helpers
// ─────────────────────────────────────────────

fn is_raw_icon(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed == "x" || trimmed == "X" {
        return true;
    }
    trimmed.chars().any(is_emoji_or_symbol)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn lint_i18n_allows_symbol_strings() {
        let code = r#"fn render(ui: &mut Ui) { ui.label("x"); ui.label("●"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn lint_i18n_detects_richtext_new_via_path_call() {
        let code = r#"
            fn render(ui: &mut Ui) {
                ui.label(egui::RichText::new("Hardcoded Text"));
            }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
    }

    #[test]
    fn is_emoji_or_symbol_tag_range() {
        assert!(is_emoji_or_symbol('\u{E0001}'));
        assert!(is_emoji_or_symbol('\u{E007F}'));
    }

    #[test]
    fn lint_i18n_detects_paren_wrapped_string() {
        let code = r#"fn render(ui: &mut Ui) { ui.label(("Hello")); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
    }

    #[test]
    fn lint_i18n_detects_reference_wrapped_string() {
        let code = r#"fn render(ui: &mut Ui) { ui.label(&"Hello"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
    }

    #[test]
    fn lint_i18n_ignores_non_ui_type_new() {
        let code = r#"fn render() { let _ = SomeOtherType::new("not flagged"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn lint_i18n_detects_format_in_button() {
        let code = r#"fn render(ui: &mut Ui) { ui.button(format!("Save {}", x)); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
    }

    #[test]
    fn check_expr_for_hardcoded_string_handles_group_expr() {
        let mut visitor = I18nHardcodeVisitor {
            file: PathBuf::from("test.rs"),
            violations: Vec::new(),
        };
        let lit = syn::parse_str::<syn::Expr>("\"hardcoded\"").unwrap();
        let group = syn::Expr::Group(syn::ExprGroup {
            attrs: vec![],
            group_token: syn::token::Group::default(),
            expr: Box::new(lit),
        });
        visitor.check_expr_for_hardcoded_string(&group, "label");
        assert!(!visitor.violations.is_empty());
    }

    #[test]
    fn lint_i18n_skips_non_ui_type_new_with_string() {
        let code = r#"fn render() { let _ = HashMap::new("some string"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

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
