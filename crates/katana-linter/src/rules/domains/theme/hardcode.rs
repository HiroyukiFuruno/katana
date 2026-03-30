use crate::utils::{collect_rs_files, parse_file, span_location};
use crate::Violation;
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;

struct HardcodedColorVisitor<'a> {
    file_path: &'a Path,
    violations: Vec<Violation>,
}

impl<'a, 'ast> Visit<'ast> for HardcodedColorVisitor<'a> {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(expr_path) = &*node.func {
            let path_str = expr_path
                .path
                .segments
                .iter()
                .map(|seg| seg.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            if path_str.contains("Color32::from_rgb")
                || path_str.contains("Color32::from_rgba")
                || path_str.contains("Color32::from_black_alpha")
                || path_str.contains("Color32::from_white_alpha")
                || path_str.contains("Color32::from_gray")
            {
                let (line, col) = span_location(node.span());
                self.violations.push(Violation {
                    file: self.file_path.to_path_buf(),
                    line,
                    column: col,
                    message: format!(
                        "Hardcoded color detected: `{}`. Please define this color in `ThemeColors` and use it via the theme system.",
                        path_str
                    ),
                });
            }
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        let path_str = node
            .path
            .segments
            .iter()
            .map(|seg| seg.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");

        let forbidden_constants = [
            "Color32::RED",
            "Color32::GREEN",
            "Color32::BLUE",
            "Color32::YELLOW",
            "Color32::WHITE",
            "Color32::BLACK",
            "Color32::TRANSPARENT",
            "Color32::LIGHT_GRAY",
            "Color32::DARK_GRAY",
        ];

        for c in forbidden_constants {
            if path_str.ends_with(c) {
                let (line, col) = span_location(node.span());
                self.violations.push(Violation {
                    file: self.file_path.to_path_buf(),
                    line,
                    column: col,
                    message: format!(
                        "Hardcoded color constant detected: `{}`. Please define this color in `ThemeColors` and use it via the theme system.",
                        path_str
                    ),
                });
                break;
            }
        }
        syn::visit::visit_expr_path(self, node);
    }
}

pub fn lint_no_hardcoded_colors(workspace_root: &Path) -> Vec<Violation> {
    let ui_dir = workspace_root.join("crates/katana-ui/src");
    let ui_files = collect_rs_files(&ui_dir);
    let mut violations = Vec::new();

    for file in ui_files {
        let file_name = file.file_name().unwrap_or_default().to_string_lossy();
        if file_name == "theme_bridge.rs" || file_name == "svg_loader.rs" {
            continue;
        }

        if let Ok(ast) = parse_file(&file) {
            let mut visitor = HardcodedColorVisitor {
                file_path: &file,
                violations: Vec::new(),
            };
            visitor.visit_file(&ast);
            violations.extend(visitor.violations);
        }
    }

    violations
}
