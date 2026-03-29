use crate::utils::{collect_rs_files, parse_file};
use crate::Violation;
use std::collections::HashSet;
use std::path::Path;
use syn::visit::Visit;

struct ThemePropertyExtractor {
    properties: Vec<(String, usize, usize)>,
}

impl<'ast> Visit<'ast> for ThemePropertyExtractor {
    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.extract_color_properties(node);
        syn::visit::visit_item_struct(self, node);
    }
}

impl ThemePropertyExtractor {
    fn extract_color_properties(&mut self, node: &syn::ItemStruct) {
        use crate::utils::span_location;
        let name = node.ident.to_string();
        if name != "ThemeColors"
            && name != "SystemColors"
            && name != "CodeColors"
            && name != "PreviewColors"
        {
            return;
        }

        for field in &node.fields {
            let Some(ident) = &field.ident else { continue };
            let syn::Type::Path(type_path) = &field.ty else {
                continue;
            };
            let Some(segment) = type_path.path.segments.last() else {
                continue;
            };

            let type_name = segment.ident.to_string();
            if type_name == "Rgb" || type_name == "Rgba" {
                let (line, col) = span_location(ident.span());
                self.properties.push((ident.to_string(), line, col));
            }
        }
    }
}

struct FieldAccessVisitor {
    used_fields: HashSet<String>,
}

impl<'ast> Visit<'ast> for FieldAccessVisitor {
    fn visit_expr_field(&mut self, node: &'ast syn::ExprField) {
        if let syn::Member::Named(ident) = &node.member {
            self.used_fields.insert(ident.to_string());
        }
        syn::visit::visit_expr_field(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if node.path.is_ident("vec") {
            let Ok(exprs) = node.parse_body_with(
                syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
            ) else {
                return syn::visit::visit_macro(self, node);
            };
            for expr in exprs {
                self.visit_expr(&expr);
            }
        }
        syn::visit::visit_macro(self, node);
    }
}

pub fn lint_unused_theme_colors(workspace_root: &Path) -> Vec<Violation> {
    let types_rs_path = workspace_root.join("crates/katana-platform/src/theme/types.rs");
    let types_ast = get_types_ast(&types_rs_path);

    let mut extractor = ThemePropertyExtractor {
        properties: Vec::new(),
    };
    extractor.visit_file(&types_ast);

    let (general_access, settings_access) = scan_ui_files(workspace_root);
    let mut violations = Vec::new();

    for (prop_name, line, col) in extractor.properties {
        check_unused_property(
            &prop_name,
            line,
            col,
            &types_rs_path,
            &general_access,
            &mut violations,
        );
        check_unexposed_property(
            &prop_name,
            line,
            col,
            &types_rs_path,
            &settings_access,
            &mut violations,
        );
    }
    violations
}

fn get_types_ast(path: &Path) -> syn::File {
    parse_file(path).unwrap_or_else(|e| {
        panic!(
            "Failed to parse theme/types.rs for ast_linter_no_unused_theme_colors: {:?}",
            e
        )
    })
}

fn scan_ui_files(workspace_root: &Path) -> (FieldAccessVisitor, FieldAccessVisitor) {
    let ui_dir = workspace_root.join("crates/katana-ui/src");
    let mut general_access = FieldAccessVisitor {
        used_fields: HashSet::new(),
    };
    let mut settings_access = FieldAccessVisitor {
        used_fields: HashSet::new(),
    };

    for file in collect_rs_files(&ui_dir) {
        let Ok(ast) = parse_file(&file) else {
            continue;
        };
        general_access.visit_file(&ast);
        if file.file_name().unwrap_or_default() == "settings_window.rs" {
            settings_access.visit_file(&ast);
        }
    }
    (general_access, settings_access)
}

fn check_unused_property(
    prop_name: &str,
    line: usize,
    col: usize,
    types_rs_path: &Path,
    general_access: &FieldAccessVisitor,
    violations: &mut Vec<Violation>,
) {
    if !general_access.used_fields.contains(prop_name) {
        violations.push(Violation {
            file: types_rs_path.to_path_buf(),
            line,
            column: col,
            message: format!(
                "Theme color property `{}` is defined in ThemeColors but never accessed in UI code. Please wire it up to `katana-ui` or remove it.",
                prop_name
            ),
        });
    }
}

fn check_unexposed_property(
    prop_name: &str,
    line: usize,
    col: usize,
    types_rs_path: &Path,
    settings_access: &FieldAccessVisitor,
    violations: &mut Vec<Violation>,
) {
    if !settings_access.used_fields.contains(prop_name) {
        violations.push(Violation {
            file: types_rs_path.to_path_buf(),
            line,
            column: col,
            message: format!(
                "Theme color property `{}` is not exposed in `settings_window.rs`. All custom colors must be editable by the user.",
                prop_name
            ),
        });
    }
}
