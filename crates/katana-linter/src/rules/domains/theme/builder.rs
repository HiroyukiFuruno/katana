use crate::utils::{collect_rs_files, parse_file, span_location};
use crate::Violation;
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;

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
        if file.file_name().unwrap_or_default() == "mod.rs" {
            continue;
        }

        let Ok(ast) = parse_file(&file) else {
            continue;
        };
        let mut visitor = BuilderEnforcementVisitor {
            file_path: &file,
            violations: Vec::new(),
        };
        visitor.visit_file(&ast);
        violations.extend(visitor.violations);
    }

    violations
}
