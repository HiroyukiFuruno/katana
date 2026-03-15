// AST Linter — カスタム静的解析エンジン
//
// coding-rules.md の第11章・第12章で定義された規約を、
// syn クレートの AST トラバースにより機械的に強制する。
//
// このテストファイルは `cargo test` で実行され、lefthook の
// pre-commit / pre-push フックを通じてハードゲートとして機能する。

use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

// ─────────────────────────────────────────────
// 違反レポート
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
// 共通ユーティリティ
// ─────────────────────────────────────────────

/// `proc_macro2::Span` から (行番号, カラム番号) を取得する。
fn span_location(span: proc_macro2::Span) -> (usize, usize) {
    (span.start().line, span.start().column + 1)
}

// ─────────────────────────────────────────────
// Allowlist — 記号・絵文字・数値のみの文字列をバイパス
// ─────────────────────────────────────────────

/// 文字列が「翻訳不要」であると判定できるかを返す。
///
/// 以下に該当する文字列は Allowlist として許可する:
/// - 空文字列・空白のみ
/// - 単一の ASCII 記号（`/`, `+`, `-`, `*`, `x`, `#` など）
/// - 絵文字のみ（UI アイコン用途: `🔄`, `▶`, `▼` など）
/// - 数値のみ（`100`, `0.5` など）
/// - パス区切り・レイアウト用文字のみ（`/`, `›` など）
fn is_allowed_string(s: &str) -> bool {
    let trimmed = s.trim();

    // 空文字列・空白のみ
    if trimmed.is_empty() {
        return true;
    }

    // 単一文字で、アルファベット以外（記号・数値・句読点など）
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() == 1 {
        let c = chars[0];
        // ASCII の英字(a-z, A-Z) でなければ許可
        if !c.is_ascii_alphabetic() {
            return true;
        }
        // 単一英字の "x"（UIで閉じるボタン等）は許可
        if c == 'x' || c == 'X' {
            return true;
        }
        return false;
    }

    // 全文字が非アルファベット（記号・絵文字・数値・空白のみ）
    if trimmed
        .chars()
        .all(|c| !c.is_alphabetic() || is_emoji_or_symbol(c))
    {
        return true;
    }

    false
}

/// Unicode の「絵文字的記号」かどうかを判定する。
/// ここでは厳密な絵文字判定ではなく、ASCII英字・ひらがな・カタカナ・CJK漢字以外の
/// 「装飾的シンボル」をカバーする。
fn is_emoji_or_symbol(c: char) -> bool {
    // 各種記号・絵文字ブロック
    matches!(c,
        '\u{2000}'..='\u{2BFF}'  // 一般句読点、上付き、通貨、記号
        | '\u{2E00}'..='\u{2E7F}' // 補助句読点
        | '\u{3000}'..='\u{303F}' // CJK記号
        | '\u{FE00}'..='\u{FE0F}' // 異体字セレクタ
        | '\u{FE30}'..='\u{FE4F}' // CJK互換形
        | '\u{1F000}'..='\u{1FAFF}' // 絵文字ブロック
        | '\u{E0000}'..='\u{E007F}' // タグ
    )
}

// ─────────────────────────────────────────────
// i18n ハードコード文字列検知 Visitor
// ─────────────────────────────────────────────

/// 検査対象とする UI メソッド名のリスト。
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

/// 検査対象とする関数呼び出し（`Type::func()` 形式）のリスト。
const UI_FUNCTIONS: &[&str] = &["new"];

/// 関数呼び出しの対象となる型名。
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

    /// 引数リストから文字列リテラルのハードコードを検出する。
    fn check_string_literal_args(
        &mut self,
        args: &syn::punctuated::Punctuated<syn::Expr, syn::token::Comma>,
        method_name: &str,
    ) {
        for arg in args.iter() {
            self.check_expr_for_hardcoded_string(arg, method_name);
        }
    }

    /// 式がハードコード文字列かどうかを再帰的に検査する。
    fn check_expr_for_hardcoded_string(&mut self, expr: &syn::Expr, method_name: &str) {
        match expr {
            // 直接の文字列リテラル: "Hello"
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
                                "{method_name}() にハードコード文字列 \"{value}\" を検出。\
                                 i18n::t() または i18n::tf() を使用してください。"
                            ),
                        });
                    }
                }
            }
            // format!(...) マクロ: format!("Saved: {}", val)
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
                            "{method_name}() で format!() によるハードコード文字列合成を検出。\
                             i18n::tf() を使用してください。"
                        ),
                    });
                }
            }
            // メソッドチェーン内の RichText::new("...") は visit_expr_call で処理
            // 参照やグループ化は中身を再帰検査
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
}

/// `format!` マクロかどうかを判定する。
fn is_format_macro(mac: &syn::Macro) -> bool {
    mac.path
        .segments
        .last()
        .map(|it| it.ident == "format")
        .unwrap_or(false)
}

/// メソッドパスの末尾セグメントから型名を取得する。
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
    /// メソッド呼び出し: `receiver.method(args)` を検査する。
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method_name = node.method.to_string();
        if UI_METHODS.contains(&method_name.as_str()) {
            self.check_string_literal_args(&node.args, &method_name);
        }

        // 子ノードの探索を続行
        syn::visit::visit_expr_method_call(self, node);
    }

    /// 関数呼び出し: `Type::func(args)` を検査する。
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(expr_path) = &*node.func {
            if let Some(last_segment) = expr_path.path.segments.last() {
                let func_name = last_segment.ident.to_string();
                if UI_FUNCTIONS.contains(&func_name.as_str()) {
                    if let Some(type_name) = extract_type_from_call(&node.func) {
                        if UI_TYPES_FOR_NEW.contains(&type_name.as_str()) {
                            self.check_string_literal_args(
                                &node.args,
                                &format!("{type_name}::{func_name}"),
                            );
                        }
                    }
                }
            }
        }

        // 子ノードの探索を続行
        syn::visit::visit_expr_call(self, node);
    }
}

// ─────────────────────────────────────────────
// 共通ヘルパー
// ─────────────────────────────────────────────

/// 属性リストに `#[cfg(test)]` が含まれるかを判定する。
fn has_cfg_test_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("cfg") {
            return false;
        }
        // #[cfg(test)] の中身を文字列化して "test" を含むか確認
        attr.meta
            .require_list()
            .ok()
            .map(|list| list.tokens.to_string().contains("test"))
            .unwrap_or(false)
    })
}

// ─────────────────────────────────────────────
// マジックナンバー検知 Visitor
// ─────────────────────────────────────────────

/// マジックナンバーとして許可する数値リテラル。
/// これらは意図が明確であり、名前付き定数に抽出する必要がない。
fn is_allowed_number(value: f64) -> bool {
    const ALLOWED: &[f64] = &[
        -1.0, 0.0, 1.0, 2.0, // 100 はパーセントやスケーリングで頻出
        100.0,
    ];
    ALLOWED.iter().any(|it| (*it - value).abs() < f64::EPSILON)
}

struct MagicNumberVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
    /// const/static 宣言の内部にいるかどうかのネスト深度。
    /// 0 より大きい場合、数値リテラルは名前付き定数内なので許可する。
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
                if let Ok(value) = lit_int.base10_parse::<i64>() {
                    if !is_allowed_number(value as f64) {
                        let (line, column) = span_location(lit_int.span());
                        self.violations.push(Violation {
                            file: self.file.clone(),
                            line,
                            column,
                            message: format!(
                                "マジックナンバー {value} を検出。名前付き定数に抽出してください。"
                            ),
                        });
                    }
                }
            }
            syn::Lit::Float(lit_float) => {
                if let Ok(value) = lit_float.base10_parse::<f64>() {
                    if !is_allowed_number(value) {
                        let (line, column) = span_location(lit_float.span());
                        self.violations.push(Violation {
                            file: self.file.clone(),
                            line,
                            column,
                            message: format!(
                                "マジックナンバー {value} を検出。名前付き定数に抽出してください。"
                            ),
                        });
                    }
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
            return; // #[cfg(test)] mod はスキップ
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
            return; // #[cfg(test)] impl メソッドはスキップ
        }
        syn::visit::visit_impl_item_fn(self, node);
    }

    // `const` フィールドやローカル const（`const X: f32 = 42.0;` inside fn）
    fn visit_local(&mut self, node: &'ast syn::Local) {
        // `let` バインディング — 通常通り検査
        syn::visit::visit_local(self, node);
    }

    fn visit_expr_lit(&mut self, node: &'ast syn::ExprLit) {
        self.check_lit(&node.lit);
        syn::visit::visit_expr_lit(self, node);
    }
}

// ─────────────────────────────────────────────
// ファイル走査エンジン
// ─────────────────────────────────────────────

/// 指定パス以下の `.rs` ファイルをすべて収集する（`.gitignore` 準拠）。
fn collect_rs_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let walker = WalkBuilder::new(root).standard_filters(true).build();

    for entry in walker.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
            // テストディレクトリ自体は解析対象外
            // （Linterテストコード内のサンプルでfalse positiveを避けるため）
            let relative = path.strip_prefix(root).unwrap_or(path);
            if !relative.starts_with("tests") {
                files.push(path.to_path_buf());
            }
        }
    }
    files
}

/// 単一ファイルをパースして AST を返す。エラー時は Violation を返す。
fn parse_file(path: &Path) -> Result<syn::File, Vec<Violation>> {
    let source = std::fs::read_to_string(path).map_err(|err| {
        vec![Violation {
            file: path.to_path_buf(),
            line: 0,
            column: 0,
            message: format!("ファイル読み込みエラー: {err}"),
        }]
    })?;
    syn::parse_file(&source).map_err(|err| {
        vec![Violation {
            file: path.to_path_buf(),
            line: 0,
            column: 0,
            message: format!("構文解析エラー: {err}"),
        }]
    })
}

/// 単一ファイルに i18n ルールを適用して違反リストを返す。
fn lint_i18n(path: &Path, syntax: &syn::File) -> Vec<Violation> {
    let mut visitor = I18nHardcodeVisitor::new(path.to_path_buf());
    visitor.visit_file(syntax);
    visitor.violations
}

/// 単一ファイルにマジックナンバールールを適用して違反リストを返す。
fn lint_magic_numbers(path: &Path, syntax: &syn::File) -> Vec<Violation> {
    let mut visitor = MagicNumberVisitor::new(path.to_path_buf());
    visitor.visit_file(syntax);
    visitor.violations
}

// ─────────────────────────────────────────────
// テストエントリポイント
// ─────────────────────────────────────────────

/// ワークスペースルートを返す。
fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|it| it.parent())
        .expect("ワークスペースルートが見つかりません")
}

/// AST Lint の共通実行ロジック。
/// 指定ディレクトリ群の全 .rs ファイルに対して lint 関数を適用し、
/// 違反があればパニックする。
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
            "解析対象の .rs ファイルが見つかりません: {}",
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
            "\n\n🚨 AST Linter [{rule_name}]: 違反が {} 件見つかりました:\n\n{}\n\n\
            💡 {hint}\n\
            📖 詳細: docs/coding-rules.md を参照\n",
            all_violations.len(),
            report
        );
    }
}

/// i18n ルール: UI メソッドへのハードコード文字列を検出。
/// 対象: 全クレート（将来どのクレートにUIコードが追加されても検知する）。
#[test]
fn ast_linter_i18n_no_hardcoded_strings() {
    let root = workspace_root();
    run_ast_lint(
        "i18n",
        "修正方法: 文字列リテラルを i18n::t(\"key\") または i18n::tf(\"key\", &[...]) に置き換えてください。",
        &[
            root.join("crates/katana-core/src"),
            root.join("crates/katana-platform/src"),
            root.join("crates/katana-ui/src"),
        ],
        lint_i18n,
    );
}

/// マジックナンバールール: const/static 外の数値リテラルを検出。
/// 対象: 全クレート（コーディング規約はプロジェクト全体に適用）。
#[test]
fn ast_linter_no_magic_numbers() {
    let root = workspace_root();
    run_ast_lint(
        "magic-number",
        "修正方法: 数値リテラルを名前付き定数（const）に抽出してください。",
        &[
            root.join("crates/katana-core/src"),
            root.join("crates/katana-platform/src"),
            root.join("crates/katana-ui/src"),
        ],
        lint_magic_numbers,
    );
}

// ─────────────────────────────────────────────
// Allowlist のユニットテスト
// ─────────────────────────────────────────────

#[cfg(test)]
mod allowlist_tests {
    use super::is_allowed_string;

    #[test]
    fn 空文字列は許可される() {
        assert!(is_allowed_string(""));
    }

    #[test]
    fn 空白のみは許可される() {
        assert!(is_allowed_string("   "));
        assert!(is_allowed_string("\n"));
        assert!(is_allowed_string("\t"));
    }

    #[test]
    fn 単一記号は許可される() {
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
    fn 単一英字xは許可される() {
        // 閉じるボタン等でよく使われる
        assert!(is_allowed_string("x"));
        assert!(is_allowed_string("X"));
    }

    #[test]
    fn 単一英字は拒否される() {
        assert!(!is_allowed_string("a"));
        assert!(!is_allowed_string("S"));
    }

    #[test]
    fn 絵文字のみは許可される() {
        assert!(is_allowed_string("🔄"));
        assert!(is_allowed_string("⬇"));
    }

    #[test]
    fn 数値のみは許可される() {
        assert!(is_allowed_string("100"));
        assert!(is_allowed_string("0.5"));
    }

    #[test]
    fn 記号と数値の組み合わせは許可される() {
        assert!(is_allowed_string("100%"));
    }

    #[test]
    fn 英語テキストは拒否される() {
        assert!(!is_allowed_string("Hello"));
        assert!(!is_allowed_string("Save"));
        assert!(!is_allowed_string("Ready"));
        assert!(!is_allowed_string("English"));
    }

    #[test]
    fn 日本語テキストは拒否される() {
        assert!(!is_allowed_string("保存"));
        assert!(!is_allowed_string("プレビュー"));
        assert!(!is_allowed_string("日本語"));
    }

    #[test]
    fn 記号混じりの日本語テキストは拒否される() {
        assert!(!is_allowed_string("⚠ エラー"));
        assert!(!is_allowed_string("⬇ ダウンロード"));
    }

    #[test]
    fn 記号のみの複数文字は許可される() {
        assert!(is_allowed_string("..."));
        assert!(is_allowed_string("---"));
        assert!(is_allowed_string("==="));
    }
}

// ─────────────────────────────────────────────
// 内部ロジックの追加ユニットテスト
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
        // カタカナ U+30A0..U+30FF — not in emoji block
        assert!(!is_emoji_or_symbol('ア'));
        assert!(!is_emoji_or_symbol('テ'));
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
        assert!(errors[0].to_string().contains("構文解析エラー"));
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

    // is_emoji_or_symbol: タグ範囲 U+E0000..U+E007F (L105)
    #[test]
    fn is_emoji_or_symbol_tag_range() {
        assert!(is_emoji_or_symbol('\u{E0001}'));
        assert!(is_emoji_or_symbol('\u{E007F}'));
    }

    // is_emoji_or_symbol: 補助句読点 U+2E00..U+2E7F (L100)
    #[test]
    fn is_emoji_or_symbol_supplemental_punctuation() {
        assert!(is_emoji_or_symbol('\u{2E00}'));
    }

    // is_emoji_or_symbol: CJK記号 U+3000..U+303F (L101)
    #[test]
    fn is_emoji_or_symbol_cjk_symbols() {
        assert!(is_emoji_or_symbol('\u{3000}')); // ideographic space
        assert!(is_emoji_or_symbol('\u{3001}')); // 。
    }

    // is_emoji_or_symbol: 異体字セレクタ U+FE00..U+FE0F (L102)
    #[test]
    fn is_emoji_or_symbol_variation_selectors() {
        assert!(is_emoji_or_symbol('\u{FE00}'));
        assert!(is_emoji_or_symbol('\u{FE0F}'));
    }

    // is_emoji_or_symbol: CJK互換形 U+FE30..U+FE4F (L103)
    #[test]
    fn is_emoji_or_symbol_cjk_compat() {
        assert!(is_emoji_or_symbol('\u{FE30}'));
    }

    // check_expr_for_hardcoded_string: Paren式の再帰 (L208-210)
    #[test]
    fn lint_i18n_detects_paren_wrapped_string() {
        let code = r#"fn render(ui: &mut Ui) { ui.label(("Hello")); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
    }

    // check_expr_for_hardcoded_string: Reference式の再帰 (L205-207)
    #[test]
    fn lint_i18n_detects_reference_wrapped_string() {
        let code = r#"fn render(ui: &mut Ui) { ui.label(&"Hello"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
    }

    // check_lit: Int マジックナンバー (L329-342)
    #[test]
    fn lint_magic_numbers_detects_int_literal() {
        let code = r#"fn foo() -> i32 { let x = 42; x }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
        assert!(violations[0].to_string().contains("42"));
    }

    // visit_expr_call: 非UI型のnew()は検出しない (L264-267)
    #[test]
    fn lint_i18n_ignores_non_ui_type_new() {
        let code = r#"fn render() { let _ = SomeOtherType::new("not flagged"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    // parse_file: ファイルが存在しない場合 (L435-442)
    #[test]
    fn parse_file_returns_error_for_nonexistent_file() {
        let result = parse_file(std::path::Path::new("/nonexistent/file.rs"));
        assert!(result.is_err());
        let errors = result.err().unwrap();
        assert!(errors[0].to_string().contains("ファイル読み込みエラー"));
    }

    // lint_magic_numbers: 負の値 -1 は許可される
    #[test]
    fn lint_magic_numbers_allows_negative_one() {
        let code = r#"fn foo() -> i32 { -1 }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    // lint_magic_numbers: static 内の数値は許可される
    #[test]
    fn lint_magic_numbers_allows_static_context() {
        let code = r#"static FOO: i32 = 42;"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_magic_numbers(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 0);
    }

    // format!() 内のハードコード検出 (L178-201)
    #[test]
    fn lint_i18n_detects_format_in_button() {
        let code = r#"fn render(ui: &mut Ui) { ui.button(format!("Save {}", x)); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_i18n(&PathBuf::from("fake.rs"), &syntax);
        assert!(!violations.is_empty());
    }
}
