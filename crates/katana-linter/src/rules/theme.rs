use crate::utils::{collect_rs_files, parse_file, span_location};
use crate::Violation;
use std::collections::HashSet;
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;

// ─────────────────────────────────────────────
// Theme Property Definition Extractor
// ─────────────────────────────────────────────

struct ThemePropertyExtractor {
    /// Stores the (property_name, line, column) of found color fields.
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
                            // We only track fields that are specifically defined as colors
                            if type_name == "Rgb" || type_name == "Rgba" {
                                use crate::utils::span_location;
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

// ─────────────────────────────────────────────
// Field Access Collector
// ─────────────────────────────────────────────

struct FieldAccessVisitor {
    /// Stores all struct field names that were accessed (e.g. `foo.bar` -> stores "bar").
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

// ─────────────────────────────────────────────
// Lint Runner
// ─────────────────────────────────────────────

pub fn lint_unused_theme_colors(workspace_root: &Path) -> Vec<Violation> {
    let types_rs_path = workspace_root.join("crates/katana-platform/src/theme/types.rs");
    let types_ast = parse_file(&types_rs_path).unwrap_or_else(|e| {
        panic!(
            "Failed to parse theme/types.rs for ast_linter_no_unused_theme_colors: {:?}",
            e
        )
    });

    // 1. Extract all color properties from the theme struct definition
    let mut extractor = ThemePropertyExtractor {
        properties: Vec::new(),
    };
    extractor.visit_file(&types_ast);

    // 2. Discover all UI files
    let ui_dir = workspace_root.join("crates/katana-ui/src");
    let ui_files = collect_rs_files(&ui_dir);

    // 3. Collect field accesses in all those files, and specifically in settings_window.rs
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

    // 4. Compare definitions against usages
    let mut violations = Vec::new();

    for (prop_name, line, col) in extractor.properties {
        // Enforce that the property is wired up to the UI *somehow* (e.g. mapped in theme_bridge or shell_ui)
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

        // Enforce that the property is exposed in the Settings Window so the user can change it
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

// ─────────────────────────────────────────────
// Hardcoded Color Detection
// ─────────────────────────────────────────────

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
        // theme_bridge.rs naturally handles Color32 conversion, so we ignore it.
        // We also ignore svgs / icon logic if it just defines 1x1 default pixels,
        // but let's blanket ignore theme_bridge and let the rest be scanned.
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

// ─────────────────────────────────────────────
// Builder Enforcement Detection
// ─────────────────────────────────────────────

struct BuilderEnforcementVisitor<'a> {
    file_path: &'a Path,
    violations: Vec<Violation>,
}

impl<'a, 'ast> Visit<'ast> for BuilderEnforcementVisitor<'a> {
    fn visit_expr_struct(&mut self, node: &'ast syn::ExprStruct) {
        let path_str = node
            .path
            .segments
            .iter()
            .map(|seg| seg.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");

        if path_str == "PresetColorData" {
            let (line, col) = span_location(node.span());
            self.violations.push(Violation {
                file: self.file_path.to_path_buf(),
                line,
                column: col,
                message: "Theme presets must use `ThemePresetBuilder::new(...)` instead of instantiating `PresetColorData` directly to enforce DRY design.".to_string(),
            });
        }
        syn::visit::visit_expr_struct(self, node);
    }
}

pub fn lint_theme_builder_enforcement(workspace_root: &Path) -> Vec<Violation> {
    let presets_dir = workspace_root.join("crates/katana-platform/src/theme/presets");
    let preset_files = collect_rs_files(&presets_dir);
    let mut violations = Vec::new();

    for file in preset_files {
        // We only enforce this on individual preset files, not mod.rs which shouldn't have structs anyway.
        if file.file_name().unwrap_or_default() == "mod.rs" {
            continue;
        }

        if let Ok(ast) = parse_file(&file) {
            let mut visitor = BuilderEnforcementVisitor {
                file_path: &file,
                violations: Vec::new(),
            };
            visitor.visit_file(&ast);
            violations.extend(visitor.violations);
        }
    }

    violations
}
