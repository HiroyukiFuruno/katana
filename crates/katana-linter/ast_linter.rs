// AST Linter — Custom Static Analysis Engine
//
// Mechanically enforces conventions defined in Chapters 11 and 12
// of coding-rules.md via AST traversal using the `syn` crate.
//
// This test file runs during `cargo test` and functions as a hard gate
// through lefthook's pre-commit / pre-push hooks.

use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

// ─────────────────────────────────────────────
// Violation Report
// ─────────────────────────────────────────────

#[derive(Debug)]
struct Violation {
    file: PathBuf,
    line: usize,
    column: usize,
    message: String,
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "  {}:{}:{} — {}",
            self.file.display(),
            self.line,
            self.column,
            self.message
        )
    }
}

// ─────────────────────────────────────────────
// Common Utilities
// ─────────────────────────────────────────────

/// Get (line, column) from `proc_macro2::Span`.
fn span_location(span: proc_macro2::Span) -> (usize, usize) {
    (span.start().line, span.start().column + 1)
}

// ─────────────────────────────────────────────
// Allowlist — Bypass symbols, emojis, and numbers
// ─────────────────────────────────────────────

/// Determine if a string can be classified as "No translation needed".
///
/// Strings matching the following criteria are bypassed by the Allowlist:
/// - Empty string or whitespace only
/// - Single ASCII symbol (`/`, `+`, `-`, `*`, `x`, `#`, etc.)
/// - Emoji only (For UI icons: `🔄`, `▶`, `▼`, etc.)
/// - Numbers only (`100`, `0.5`, etc.)
/// - Path separators or layout characters only (`/`, `›`, etc.)
fn is_allowed_string(s: &str) -> bool {
    let trimmed = s.trim();

    // Empty string or whitespace only
    if trimmed.is_empty() {
        return true;
    }

    // Single character, non-alphabet (symbol, number, punctuation, etc.)
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() == 1 {
        let c = chars[0];
        // Allow if it's not an ASCII alphabet (a-z, A-Z)
        if !c.is_ascii_alphabetic() {
            return true;
        }
        // Allow single letter "x" (often used as close button in UI, etc.)
        if c == 'x' || c == 'X' {
            return true;
        }
        return false;
    }

    // All characters are non-alphabetic (symbol, emoji, number, or whitespace only)
    if trimmed
        .chars()
        .all(|c| !c.is_alphabetic() || is_emoji_or_symbol(c))
    {
        return true;
    }

    false
}

/// Determine if a character is an "emoji-like symbol" in Unicode.
/// Rather than strict emoji detection, this covers "decorative symbols"
/// excluding ASCII alphabets, Hiragana, Katakana, and CJK Kanji.
fn is_emoji_or_symbol(c: char) -> bool {
    // Various symbol and emoji blocks
    matches!(c,
        '\u{2000}'..='\u{2BFF}'  // General Punctuation, Superscripts, Currency, Symbols
        | '\u{2E00}'..='\u{2E7F}' // Supplemental Punctuation
        | '\u{3000}'..='\u{303F}' // CJK Symbols and Punctuation
        | '\u{FE00}'..='\u{FE0F}' // Variation Selectors
        | '\u{FE30}'..='\u{FE4F}' // CJK Compatibility Forms
        | '\u{1F000}'..='\u{1FAFF}' // Emoji blocks
        | '\u{E0000}'..='\u{E007F}' // Tags
    )
}

// ─────────────────────────────────────────────
// i18n Hardcoded String Detection Visitor
// ─────────────────────────────────────────────

/// List of UI method names to inspect.
const UI_METHODS: &[&str] = &[
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
];

/// List of function calls (`Type::func()` format) to inspect.
const UI_FUNCTIONS: &[&str] = &["new"];

/// Target type names for function calls.
const UI_TYPES_FOR_NEW: &[&str] = &["RichText"];

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
            // Direct string literal: "Hello"
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
            // format!(...) macro: format!("Saved: {}", val)
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
            // RichText::new("...") inside a method chain is handled by visit_expr_call.
            // References or groupings recursively check their contents.
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

    /// Inspect `Type::func(args)` style function calls to ensure strings are not passed to UI type constructors.
    fn check_call_for_ui_violation(&mut self, node: &syn::ExprCall) {
        let syn::Expr::Path(expr_path) = &*node.func else {
            return;
        };
        // syn parser invariant: Path always has at least one segment
        let last_segment = expr_path
            .path
            .segments
            .last()
            .expect("syn::Path always has at least one segment");
        let func_name = last_segment.ident.to_string();
        if !UI_FUNCTIONS.contains(&func_name.as_str()) {
            return;
        }
        let Some(type_name) = extract_type_from_call(&node.func) else {
            return;
        };
        if !UI_TYPES_FOR_NEW.contains(&type_name.as_str()) {
            return;
        }
        self.check_string_literal_args(&node.args, &format!("{type_name}::{func_name}"));
    }
}

/// Determine if it is a `format!` macro.
fn is_format_macro(mac: &syn::Macro) -> bool {
    mac.path
        .segments
        .last()
        .map(|it| it.ident == "format")
        .unwrap_or(false)
}

/// Extract type name from the last segment of the method path.
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
    /// Inspect method call: `receiver.method(args)`.
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method_name = node.method.to_string();
        if UI_METHODS.contains(&method_name.as_str()) {
            self.check_string_literal_args(&node.args, &method_name);
        }

        // Continue exploring child nodes
        syn::visit::visit_expr_method_call(self, node);
    }

    /// Inspect function call: `Type::func(args)`.
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        self.check_call_for_ui_violation(node);

        // Continue exploring child nodes
        syn::visit::visit_expr_call(self, node);
    }
}

// ─────────────────────────────────────────────
// Common Helpers
// ─────────────────────────────────────────────

/// Check if the attribute list contains `#[cfg(test)]`.
fn has_cfg_test_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("cfg") {
            return false;
        }
        // Stringify #[cfg(test)] contents and check if it contains "test"
        attr.meta
            .require_list()
            .ok()
            .map(|list| list.tokens.to_string().contains("test"))
            .unwrap_or(false)
    })
}

// ─────────────────────────────────────────────
// Magic Number Detection Visitor
// ─────────────────────────────────────────────

/// Numeric literals allowed as magic numbers.
/// These have clear intent and do not need to be extracted into named constants.
fn is_allowed_number(value: f64) -> bool {
    const ALLOWED: &[f64] = &[
        -1.0, 0.0, 1.0, 2.0, // 100 often appears in percentages and scaling
        100.0,
    ];
    ALLOWED.iter().any(|it| (*it - value).abs() < f64::EPSILON)
}

struct MagicNumberVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
    /// Nesting depth of being inside a const/static declaration.
    /// If greater than 0, numeric literals are inside a named constant and thus allowed.
    in_const_context: u32,
}

impl MagicNumberVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
            in_const_context: 0,
        }
    }

    fn check_lit(&mut self, lit: &syn::Lit) {
        if self.in_const_context > 0 {
            return;
        }
        match lit {
            syn::Lit::Int(lit_int) => {
                // syn's LitInt is always a valid integer literal
                let value = lit_int
                    .base10_parse::<i64>()
                    .expect("syn::LitInt should always be parseable");
                if !is_allowed_number(value as f64) {
                    let (line, column) = span_location(lit_int.span());
                    self.violations.push(Violation {
                        file: self.file.clone(),
                        line,
                        column,
                        message: format!(
                            "Magic number {value} detected. Please extract to a named constant."
                        ),
                    });
                }
            }
            syn::Lit::Float(lit_float) => {
                // syn's LitFloat is always a valid floating-point literal
                let value = lit_float
                    .base10_parse::<f64>()
                    .expect("syn::LitFloat should always be parseable");
                if !is_allowed_number(value) {
                    let (line, column) = span_location(lit_float.span());
                    self.violations.push(Violation {
                        file: self.file.clone(),
                        line,
                        column,
                        message: format!(
                            "Magic number {value} detected. Please extract to a named constant."
                        ),
                    });
                }
            }
            _ => {}
        }
    }
}

impl<'ast> Visit<'ast> for MagicNumberVisitor {
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

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if has_cfg_test_attr(&node.attrs) {
            return; // Skip #[cfg(test)] mod
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
            return; // Skip #[cfg(test)] impl method
        }
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_impl_item_const(&mut self, node: &'ast syn::ImplItemConst) {
        self.in_const_context += 1;
        syn::visit::visit_impl_item_const(self, node);
        self.in_const_context -= 1;
    }

    // `const` fields or local const (`const X: f32 = 42.0;` inside fn)
    fn visit_local(&mut self, node: &'ast syn::Local) {
        // `let` binding — inspect as usual
        syn::visit::visit_local(self, node);
    }

    fn visit_expr_lit(&mut self, node: &'ast syn::ExprLit) {
        self.check_lit(&node.lit);
        syn::visit::visit_expr_lit(self, node);
    }
}

// ─────────────────────────────────────────────
// File Traversal Engine
// ─────────────────────────────────────────────

/// Collect all `.rs` files under the specified path (respecting `.gitignore`).
fn collect_rs_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let walker = WalkBuilder::new(root).standard_filters(true).build();

    for entry in walker.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
            // Test directory itself is excluded from analysis
            // (To avoid false positives in sample code within Linter tests)
            let relative = path.strip_prefix(root).unwrap_or(path);
            if !relative.starts_with("tests") {
                files.push(path.to_path_buf());
            }
        }
    }
    files
}

/// Parse a single file and return the AST. Return a Violation on error.
fn parse_file(path: &Path) -> Result<syn::File, Vec<Violation>> {
    let source = std::fs::read_to_string(path).map_err(|err| {
        vec![Violation {
            file: path.to_path_buf(),
            line: 0,
            column: 0,
            message: format!("File read error: {err}"),
        }]
    })?;
    syn::parse_file(&source).map_err(|err| {
        vec![Violation {
            file: path.to_path_buf(),
            line: 0,
            column: 0,
            message: format!("Syntax parse error: {err}"),
        }]
    })
}

/// Apply i18n rule to a single file and return a list of violations.
fn lint_i18n(path: &Path, syntax: &syn::File) -> Vec<Violation> {
    let mut visitor = I18nHardcodeVisitor::new(path.to_path_buf());
    visitor.visit_file(syntax);
    visitor.violations
}

/// Apply magic number rule to a single file and return a list of violations.
fn lint_magic_numbers(path: &Path, syntax: &syn::File) -> Vec<Violation> {
    let mut visitor = MagicNumberVisitor::new(path.to_path_buf());
    visitor.visit_file(syntax);
    visitor.violations
}

// ─────────────────────────────────────────────
// Test Entry Point
// ─────────────────────────────────────────────

/// Return the workspace root.
fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|it| it.parent())
        .expect("Workspace root not found")
}

/// Common execution logic for AST Lint.
/// Applies the lint function to all .rs files in the specified directories,
/// and panics if any violations are found.
fn run_ast_lint(
    rule_name: &str,
    hint: &str,
    target_dirs: &[PathBuf],
    lint_fn: fn(&Path, &syn::File) -> Vec<Violation>,
) {
    let mut all_violations: Vec<Violation> = Vec::new();

    for target_dir in target_dirs {
        let rs_files = collect_rs_files(target_dir);
        assert!(
            !rs_files.is_empty(),
            "No .rs files found for analysis: {}",
            target_dir.display()
        );

        for file in &rs_files {
            match parse_file(file) {
                Ok(syntax) => {
                    let violations = lint_fn(file, &syntax);
                    all_violations.extend(violations);
                }
                Err(errors) => {
                    all_violations.extend(errors);
                }
            }
        }
    }

    if !all_violations.is_empty() {
        let report = all_violations
            .iter()
            .map(|it| it.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        panic!(
            "\n\n🚨 AST Linter [{rule_name}]: Found {} violation(s):\n\n{}\n\n\
            💡 {hint}\n\
            📖 Details: See docs/coding-rules.md\n",
            all_violations.len(),
            report
        );
    }
}

/// i18n rule: Detect hardcoded strings in UI methods.
/// Scope: All crates (detects UI code added to any crate in the future).
#[test]
fn ast_linter_i18n_no_hardcoded_strings() {
    let root = workspace_root();
    run_ast_lint(
        "i18n",
        "Fix: Replace string literals with i18n::t(\"key\") or i18n::tf(\"key\", &[...]).",
        &[
            root.join("crates/katana-core/src"),
            root.join("crates/katana-platform/src"),
            root.join("crates/katana-ui/src"),
        ],
        lint_i18n,
    );
}

/// Magic number rule: Detect numeric literals outside of const/static.
/// Scope: All crates (coding conventions apply project-wide).
#[test]
fn ast_linter_no_magic_numbers() {
    let root = workspace_root();
    run_ast_lint(
        "magic-number",
        "Fix: Extract numeric literals into named constants (const).",
        &[
            root.join("crates/katana-core/src"),
            root.join("crates/katana-platform/src"),
            root.join("crates/katana-ui/src"),
        ],
        lint_magic_numbers,
    );
}

// ─────────────────────────────────────────────
// Allowlist Unit Tests
// ─────────────────────────────────────────────

#[cfg(test)]
mod allowlist_tests {
    use super::is_allowed_string;

    #[test]
    fn allowlist_allows_empty_strings() {
        assert!(is_allowed_string(""));
    }

    #[test]
    fn allowlist_allows_whitespace_only() {
        assert!(is_allowed_string("   "));
        assert!(is_allowed_string("\n"));
        assert!(is_allowed_string("\t"));
    }

    #[test]
    fn allowlist_allows_single_symbol() {
        assert!(is_allowed_string("/"));
        assert!(is_allowed_string("+"));
        assert!(is_allowed_string("-"));
        assert!(is_allowed_string("*"));
        assert!(is_allowed_string("#"));
        assert!(is_allowed_string("●"));
        assert!(is_allowed_string("›"));
        assert!(is_allowed_string("▶"));
        assert!(is_allowed_string("▼"));
    }

    #[test]
    fn allowlist_allows_single_letter_x() {
        // Often used for close buttons
        assert!(is_allowed_string("x"));
        assert!(is_allowed_string("X"));
    }

    #[test]
    fn allowlist_rejects_single_letter() {
        assert!(!is_allowed_string("a"));
        assert!(!is_allowed_string("S"));
    }

    #[test]
    fn allowlist_allows_emojis_only() {
        assert!(is_allowed_string("🔄"));
        assert!(is_allowed_string("⬇"));
    }

    #[test]
    fn allowlist_allows_numbers_only() {
        assert!(is_allowed_string("100"));
        assert!(is_allowed_string("0.5"));
    }

    #[test]
    fn allowlist_allows_symbols_combined_with_numbers() {
        assert!(is_allowed_string("100%"));
    }

    #[test]
    fn allowlist_rejects_english_texts() {
        assert!(!is_allowed_string("Hello"));
        assert!(!is_allowed_string("Save"));
        assert!(!is_allowed_string("Ready"));
        assert!(!is_allowed_string("English"));
    }

    #[test]
    fn allowlist_rejects_japanese_texts() {
        assert!(!is_allowed_string("Save"));
        assert!(!is_allowed_string("Preview"));
        assert!(!is_allowed_string("Japanese"));
    }

    #[test]
    fn allowlist_rejects_japanese_texts_mixed_with_symbols() {
        assert!(!is_allowed_string("⚠ Error"));
        assert!(!is_allowed_string("⬇ Download"));
    }

    #[test]
    fn allowlist_allows_multiple_symbols() {
        assert!(is_allowed_string("..."));
        assert!(is_allowed_string("---"));
        assert!(is_allowed_string("==="));
    }
}

// ─────────────────────────────────────────────
// Additional Unit Tests for Internal Logic
// ─────────────────────────────────────────────

#[cfg(test)]
mod internal_tests {
    use super::*;
    use std::path::PathBuf;

    // Violation::fmt (L26-35)
    #[test]
    fn violation_display_format() {
        let v = Violation {
            file: PathBuf::from("src/shell.rs"),
            line: 42,
            column: 7,
            message: "test violation".to_string(),
        };
        let s = v.to_string();
        assert!(s.contains("src/shell.rs"));
        assert!(s.contains("42"));
        assert!(s.contains("7"));
        assert!(s.contains("test violation"));
    }

    // is_emoji_or_symbol (L96-107)
    #[test]
    fn is_emoji_or_symbol_returns_true_for_emoji() {
        // 🔄 is in \u{1F000}..\u{1FAFF}
        assert!(is_emoji_or_symbol('🔄'));
        // ← (U+2190) is in \u{2000}..\u{2BFF}
        assert!(is_emoji_or_symbol('←'));
    }

    #[test]
    fn is_emoji_or_symbol_returns_false_for_ascii() {
        assert!(!is_emoji_or_symbol('a'));
        assert!(!is_emoji_or_symbol('Z'));
        assert!(!is_emoji_or_symbol('5'));
    }

    #[test]
    fn is_emoji_or_symbol_returns_false_for_katakana() {
        // Katakana U+30A0..U+30FF — not in emoji block
        assert!(!is_emoji_or_symbol('A'));
        assert!(!is_emoji_or_symbol('B'));
    }

    // is_format_macro (L220-226)
    #[test]
    fn is_format_macro_detects_format_macro() {
        let code = r#"fn f() { let _ = format!("hello"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        // lint_i18n won't flag format! in a non-UI context, but parse should succeed
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    // lint_i18n: detect hardcoded string in ui.label()
    #[test]
    fn lint_i18n_detects_label_with_hardcoded_string() {
        let code = r#"fn render(ui: &mut Ui) { ui.label("Hello World"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
        assert!(violations[0].to_string().contains("Hello World"));
    }

    // lint_i18n: detect hardcoded string in RichText::new()
    #[test]
    fn lint_i18n_detects_richtext_new_with_hardcoded_string() {
        let code = r#"fn render(ui: &mut Ui) { ui.label(RichText::new("Hardcoded")); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
    }

    // lint_i18n: format!() in ui.label() triggers violation
    #[test]
    fn lint_i18n_detects_format_macro_in_label() {
        let code = r#"fn render(ui: &mut Ui) { ui.label(format!("Saved: {}", name)); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
    }

    // lint_i18n: allowed strings don't trigger violation
    #[test]
    fn lint_i18n_allows_symbol_strings() {
        // "x" is allowed, "●" is allowed
        let code = r#"fn render(ui: &mut Ui) { ui.label("x"); ui.label("●"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    // lint_magic_numbers: magic number in non-const context
    #[test]
    fn lint_magic_numbers_detects_literal_in_function() {
        let code = r#"fn foo() -> f32 { let x: f32 = 42.0; x }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
        assert!(violations[0].to_string().contains("42"));
    }

    // lint_magic_numbers: number in const is allowed
    #[test]
    fn lint_magic_numbers_allows_literal_in_const() {
        let code = r#"const FOO: f32 = 42.0;"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    // lint_magic_numbers: allowed numbers (0, 1, 2, 100, -1) are not flagged
    #[test]
    fn lint_magic_numbers_allows_common_values() {
        let code = r#"fn foo() { let a = 0; let b = 1; let c = 2; let d = 100; }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    // lint_magic_numbers: cfg(test) fn is skipped
    #[test]
    fn lint_magic_numbers_skips_test_functions() {
        let code = r#"
            #[cfg(test)]
            fn test_foo() -> i32 { 42 }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    // lint_magic_numbers: cfg(test) impl method is skipped
    #[test]
    fn lint_magic_numbers_skips_test_impl_methods() {
        let code = r#"
            impl Foo {
                #[cfg(test)]
                fn test_foo_method() -> i32 { 42 }
            }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    // has_cfg_test_attr: test attribute detection (L279-291)
    #[test]
    fn has_cfg_test_attr_returns_true_for_test_attr() {
        let code = r#"
            #[cfg(test)]
            mod tests {}
        "#;
        let syntax = syn::parse_file(code).unwrap();
        // If there's a cfg(test) mod, lint_magic_numbers won't visit it
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    // collect_rs_files / parse_file integration: parse a known bad syntax file
    #[test]
    fn parse_file_returns_error_for_invalid_syntax() {
        let tmp = tempfile::NamedTempFile::with_suffix(".rs").unwrap();
        std::fs::write(tmp.path(), "fn broken(").unwrap();
        let result = parse_file(tmp.path());
        assert!(result.is_err());
        let errors = result.err().expect("should have failed with errors");
        assert!(!errors.is_empty());
        assert!(errors[0].to_string().contains("Syntax parse error"));
    }

    // extract_type_from_call: path with >= 2 segments (L229-237)
    #[test]
    fn lint_i18n_detects_richtext_new_via_path_call() {
        let code = r#"
            fn render(ui: &mut Ui) {
                ui.label(egui::RichText::new("Hardcoded Text"));
            }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        // egui::RichText::new is detected
        assert!(!violations.is_empty());
    }

    // is_emoji_or_symbol: Tag range U+E0000..U+E007F (L105)
    #[test]
    fn is_emoji_or_symbol_tag_range() {
        assert!(is_emoji_or_symbol('\u{E0001}'));
        assert!(is_emoji_or_symbol('\u{E007F}'));
    }

    // is_emoji_or_symbol: Supplemental Punctuation U+2E00..U+2E7F (L100)
    #[test]
    fn is_emoji_or_symbol_supplemental_punctuation() {
        assert!(is_emoji_or_symbol('\u{2E00}'));
    }

    // is_emoji_or_symbol: CJK Symbols U+3000..U+303F (L101)
    #[test]
    fn is_emoji_or_symbol_cjk_symbols() {
        assert!(is_emoji_or_symbol('\u{3000}')); // ideographic space
        assert!(is_emoji_or_symbol('\u{3001}')); // 。
    }

    // is_emoji_or_symbol: Variation Selectors U+FE00..U+FE0F (L102)
    #[test]
    fn is_emoji_or_symbol_variation_selectors() {
        assert!(is_emoji_or_symbol('\u{FE00}'));
        assert!(is_emoji_or_symbol('\u{FE0F}'));
    }

    // is_emoji_or_symbol: CJK Compatibility Forms U+FE30..U+FE4F (L103)
    #[test]
    fn is_emoji_or_symbol_cjk_compat() {
        assert!(is_emoji_or_symbol('\u{FE30}'));
    }

    // check_expr_for_hardcoded_string: Recursion for Paren macro (L208-210)
    #[test]
    fn lint_i18n_detects_paren_wrapped_string() {
        let code = r#"fn render(ui: &mut Ui) { ui.label(("Hello")); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
    }

    // check_expr_for_hardcoded_string: Recursion for Reference expression (L205-207)
    #[test]
    fn lint_i18n_detects_reference_wrapped_string() {
        let code = r#"fn render(ui: &mut Ui) { ui.label(&"Hello"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
    }

    // check_lit: Int magic numbers (L329-342)
    #[test]
    fn lint_magic_numbers_detects_int_literal() {
        let code = r#"fn foo() -> i32 { let x = 42; x }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
        assert!(violations[0].to_string().contains("42"));
    }

    // visit_expr_call: Ignores new() from non-UI types (L264-267)
    #[test]
    fn lint_i18n_ignores_non_ui_type_new() {
        let code = r#"fn render() { let _ = SomeOtherType::new("not flagged"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    // parse_file: File does not exist (L435-442)
    #[test]
    fn parse_file_returns_error_for_nonexistent_file() {
        let result = parse_file(std::path::Path::new("/nonexistent/file.rs"));
        assert!(result.is_err());
        let errors = result.err().unwrap();
        assert!(errors[0].to_string().contains("File read error"));
    }

    // lint_magic_numbers: Negative value -1 is allowed
    #[test]
    fn lint_magic_numbers_allows_negative_one() {
        let code = r#"fn foo() -> i32 { -1 }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    // lint_magic_numbers: Numbers in static are allowed
    #[test]
    fn lint_magic_numbers_allows_static_context() {
        let code = r#"static FOO: i32 = 42;"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    // lint_magic_numbers: Associated const in impl block is allowed
    #[test]
    fn lint_magic_numbers_allows_impl_item_const() {
        let code = r#"
            struct Foo;
            impl Foo {
                const BAR: f32 = 14.0;
            }
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    // Detect hardcoded format in format!() (L178-201)
    #[test]
    fn lint_i18n_detects_format_in_button() {
        let code = r#"fn render(ui: &mut Ui) { ui.button(format!("Save {}", x)); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
    }

    // Recursive check for Expr::Group (L211-213)
    // Group expressions appear when using proc_macro2 grouping
    #[test]
    fn check_expr_for_hardcoded_string_handles_group_expr() {
        // Direct test for Group expr handling, manipulating Visitor directly
        // without lint_i18n
        let mut visitor = I18nHardcodeVisitor {
            file: PathBuf::from("test.rs"),
            violations: Vec::new(),
        };
        // Group expr: syn::Expr::Group is typically built through macro expansion
        // Here we ensure proc_macro2::Group containing our string gets detected
        let lit = syn::parse_str::<syn::Expr>("\"hardcoded\"").unwrap();
        let group = syn::Expr::Group(syn::ExprGroup {
            attrs: vec![],
            group_token: syn::token::Group::default(),
            expr: Box::new(lit),
        });
        visitor.check_expr_for_hardcoded_string(&group, "label");
        assert!(!visitor.violations.is_empty());
    }

    // visit_expr_call: Functions matching UI_FUNCTIONS but types without UI_TYPES_FOR_NEW (L264)
    #[test]
    fn lint_i18n_skips_non_ui_type_new_with_string() {
        // Function is named `new` (in UI_FUNCTIONS), but type is not in UI_TYPES_FOR_NEW
        let code = r#"fn render() { let _ = HashMap::new("some string"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    // visit_expr_call: extract_type_from_call returns None for single segment path (L266-267)
    #[test]
    fn lint_i18n_skips_simple_function_call() {
        // Single segment path: new("string") -> extract_type_from_call becomes None
        let code = r#"fn render() { new("some string"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    // lint_magic_numbers: Allowed float (closing brace of L342)
    #[test]
    fn lint_magic_numbers_allows_zero_float() {
        let code = r#"fn foo() { let _ = 0.0; }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    // lint_magic_numbers: base10_parse succeeds and reaches allowed value check (closing brace of L357)
    #[test]
    fn lint_magic_numbers_allows_one_float() {
        let code = r#"fn foo() { let _ = 1.0; }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    // run_lint_on_dirs: Panics on violations (L512-522)
    #[test]
    #[should_panic(expected = "AST Linter")]
    fn run_lint_on_dirs_panics_on_violations() {
        let tmp = tempfile::TempDir::new().unwrap();
        let file = tmp.path().join("bad.rs");
        std::fs::write(
            &file,
            r#"fn render(ui: &mut Ui) { ui.label("Bad String"); }"#,
        )
        .unwrap();
        run_ast_lint(
            "test_rule",
            "fix it",
            &[tmp.path().to_path_buf()],
            lint_i18n,
        );
    }

    // run_lint_on_dirs: Gathers parse errors as violations (L504-506)
    #[test]
    #[should_panic(expected = "AST Linter")]
    fn run_lint_on_dirs_collects_parse_errors() {
        let tmp = tempfile::TempDir::new().unwrap();
        let file = tmp.path().join("broken.rs");
        std::fs::write(&file, "fn broken(").unwrap();
        run_ast_lint(
            "test_rule",
            "fix it",
            &[tmp.path().to_path_buf()],
            lint_i18n,
        );
    }

    // run_lint_on_dirs: Panics when there are no files (L495)
    #[test]
    #[should_panic(expected = "No .rs files found for analysis")]
    fn run_lint_on_dirs_panics_when_no_rs_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        run_ast_lint(
            "test_rule",
            "fix it",
            &[tmp.path().to_path_buf()],
            lint_i18n,
        );
    }

    // check_expr_for_hardcoded_string: Non-String literals (like Integers) bypass condition (L178)
    #[test]
    fn lint_i18n_ignores_non_string_literal_in_label() {
        let code = r#"fn render(ui: &mut Ui) { ui.label(42); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    // check_expr_for_hardcoded_string: Non format! macro triggers false for is_format_macro (L201)
    #[test]
    fn lint_i18n_ignores_non_format_macro_in_label() {
        let code = r#"fn render(ui: &mut Ui) { ui.label(vec!["a"]); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    // extract_type_from_call: Single segment path translates to None (L235)
    #[test]
    fn extract_type_from_call_returns_none_for_single_segment() {
        let expr = syn::parse_str::<syn::Expr>("foo()").unwrap();
        assert!(extract_type_from_call(&expr).is_none());
    }

    // visit_expr_call: Non UI_FUNCTIONS function name (L266-267)
    #[test]
    fn lint_i18n_ignores_non_ui_function_path() {
        let code = r#"fn render() { SomeType::render("not flagged"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    // check_lit: Allowed int avoids triggering if block (L342)
    #[test]
    fn lint_magic_numbers_int_allowed_value_reaches_closing_brace() {
        // 0 and 1 are allowed values. parse succeeds + is_allowed_number is true -> doesn't hit `if`
        // reaches `}`
        let code = r#"fn foo() { let _ = 0; let _ = 1; let _ = 2; }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    // check_lit: Allowed float avoids triggering if block (L357)
    #[test]
    fn lint_magic_numbers_float_allowed_value_reaches_closing_brace() {
        let code = r#"fn foo() { let _ = 0.0; let _ = 1.0; }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    // L221: check_call_for_ui_violation hitting `ExprCall` without a Path in `node.func`
    // Paren wrapped function call like `(callback)("string")` generates ExprCall but func is ExprParen
    #[test]
    fn lint_i18n_ignores_paren_expr_call() {
        let code = r#"fn render() { (get_func())("some string"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    // L338/354: let-else return check within `check_lit` (tests structural limits where base10_parse won't fail)
    // syn's LitInt/LitFloat successfully parses strings natively,
    // thereby triggering violations exclusively when the content value falls off the allowed list
    // executing the successful pass (if !is_allowed_number -> true).
    #[test]
    fn lint_magic_numbers_non_allowed_int_triggers_violation() {
        let code = r#"fn foo() { let _ = 42; }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
    }

    #[test]
    fn lint_magic_numbers_non_allowed_float_triggers_violation() {
        let code = r#"fn foo() { let _ = 3.14; }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
    }
}
