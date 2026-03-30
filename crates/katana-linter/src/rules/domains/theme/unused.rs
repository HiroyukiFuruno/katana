use crate::utils::{collect_rs_files, parse_file, span_location};
use crate::Violation;
use std::collections::HashSet;
use std::path::Path;
use syn::visit::Visit;

struct ThemePropertyExtractor {
    properties: Vec<(String, usize, usize)>,
}

impl<'ast> Visit<'ast> for ThemePropertyExtractor {
    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        let name = node.ident.to_string();
        if name == "ThemeColors"
            || name == "SystemColors"
            || name == "CodeColors"
            || name == "PreviewColors"
        {
            for field in &node.fields {
                if let Some(ident) = &field.ident {
                    if let syn::Type::Path(type_path) = &field.ty {
                        if let Some(segment) = type_path.path.segments.last() {
                            let type_name = segment.ident.to_string();
                            if type_name == "Rgb" || type_name == "Rgba" {
                                let (line, col) = span_location(ident.span());
                                self.properties.push((ident.to_string(), line, col));
                            }
                        }
                    }
                }
            }
        }
        syn::visit::visit_item_struct(self, node);
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
            if let Ok(exprs) = node.parse_body_with(
                syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
            ) {
                for expr in exprs {
                    self.visit_expr(&expr);
                }
            }
        }
        syn::visit::visit_macro(self, node);
    }
}

pub fn lint_unused_theme_colors(workspace_root: &Path) -> Vec<Violation> {
    let types_rs_path = workspace_root.join("crates/katana-platform/src/theme/types.rs");
    let types_ast = parse_file(&types_rs_path).unwrap_or_else(|e| {
        panic!(
            "Failed to parse theme/types.rs for ast_linter_no_unused_theme_colors: {:?}",
            e
        )
    });

    let mut extractor = ThemePropertyExtractor {
        properties: Vec::new(),
    };
    extractor.visit_file(&types_ast);

    let ui_dir = workspace_root.join("crates/katana-ui/src");
    let ui_files = collect_rs_files(&ui_dir);

    let mut general_access = FieldAccessVisitor {
        used_fields: HashSet::new(),
    };
    let mut settings_access = FieldAccessVisitor {
        used_fields: HashSet::new(),
    };

    for file in ui_files {
        if let Ok(ast) = parse_file(&file) {
            general_access.visit_file(&ast);

            if file.file_name().unwrap_or_default() == "settings_window.rs" {
                settings_access.visit_file(&ast);
            }
        }
    }

    let mut violations = Vec::new();

    for (prop_name, line, col) in extractor.properties {
        if !general_access.used_fields.contains(&prop_name) {
            violations.push(Violation {
                file: types_rs_path.clone(),
                line,
                column: col,
                message: format!(
                    "Theme color property `{}` is defined in ThemeColors but never accessed in UI code. Please wire it up to `katana-ui` or remove it.",
                    prop_name
                ),
            });
        }

        if !settings_access.used_fields.contains(&prop_name) {
            violations.push(Violation {
                file: types_rs_path.clone(),
                line,
                column: col,
                message: format!(
                    "Theme color property `{}` is not exposed in `settings_window.rs`. All custom colors must be editable by the user.",
                    prop_name
                ),
            });
        }
    }

    violations
}
