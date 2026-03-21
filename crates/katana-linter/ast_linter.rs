// AST Linter — Custom Static Analysis Engine
//
// Mechanically enforces conventions defined in Chapters 11 and 12
// of coding-rules.md via AST traversal using the `syn` crate.
//
// This test file runs during `cargo test` and functions as a hard gate
// through lefthook's pre-commit / pre-push hooks.

use ignore::WalkBuilder;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum JsonNodeKind {
    Object,
    Array(usize),
    String,
    Number,
    Bool,
    Null,
}

impl JsonNodeKind {
    fn from_value(value: &Value) -> Self {
        match value {
            Value::Object(_) => Self::Object,
            Value::Array(items) => Self::Array(items.len()),
            Value::String(_) => Self::String,
            Value::Number(_) => Self::Number,
            Value::Bool(_) => Self::Bool,
            Value::Null => Self::Null,
        }
    }
}

impl std::fmt::Display for JsonNodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Object => write!(f, "object"),
            Self::Array(len) => write!(f, "array(len={len})"),
            Self::String => write!(f, "string"),
            Self::Number => write!(f, "number"),
            Self::Bool => write!(f, "bool"),
            Self::Null => write!(f, "null"),
        }
    }
}

// ─────────────────────────────────────────────
// Common Utilities
// ─────────────────────────────────────────────

/// Get (line, column) from `proc_macro2::Span`.
fn span_location(span: proc_macro2::Span) -> (usize, usize) {
    (span.start().line, span.start().column + 1)
}

fn locale_violation(file: &Path, message: impl Into<String>) -> Violation {
    Violation {
        file: file.to_path_buf(),
        line: 0,
        column: 0,
        message: message.into(),
    }
}

fn parse_json_file(path: &Path) -> Result<Value, Vec<Violation>> {
    let source = std::fs::read_to_string(path).map_err(|err| {
        vec![Violation {
            file: path.to_path_buf(),
            line: 0,
            column: 0,
            message: format!("Locale file read error: {err}"),
        }]
    })?;

    serde_json::from_str(&source).map_err(|err| {
        vec![Violation {
            file: path.to_path_buf(),
            line: err.line(),
            column: err.column(),
            message: format!("Locale JSON parse error: {err}"),
        }]
    })
}

fn collect_json_shape(value: &Value, path: Option<&str>, out: &mut BTreeMap<String, JsonNodeKind>) {
    let kind = JsonNodeKind::from_value(value);
    if let Some(path) = path {
        out.insert(path.to_string(), kind);
    }

    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let child_path = path
                    .map(|prefix| format!("{prefix}.{key}"))
                    .unwrap_or_else(|| key.to_string());
                collect_json_shape(child, Some(&child_path), out);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                let child_path = path
                    .map(|prefix| format!("{prefix}[{index}]"))
                    .unwrap_or_else(|| format!("[{index}]"));
                collect_json_shape(child, Some(&child_path), out);
            }
        }
        Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Null => {}
    }
}

fn collect_json_placeholders(
    value: &Value,
    path: Option<&str>,
    out: &mut BTreeMap<String, BTreeSet<String>>,
) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let child_path = path
                    .map(|prefix| format!("{prefix}.{key}"))
                    .unwrap_or_else(|| key.to_string());
                collect_json_placeholders(child, Some(&child_path), out);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                let child_path = path
                    .map(|prefix| format!("{prefix}[{index}]"))
                    .unwrap_or_else(|| format!("[{index}]"));
                collect_json_placeholders(child, Some(&child_path), out);
            }
        }
        Value::String(text) => {
            if let Some(path) = path {
                out.insert(path.to_string(), extract_placeholders(text));
            }
        }
        Value::Number(_) | Value::Bool(_) | Value::Null => {}
    }
}

fn extract_placeholders(text: &str) -> BTreeSet<String> {
    let mut placeholders = BTreeSet::new();
    let bytes = text.as_bytes();
    let mut start = 0usize;

    while start < bytes.len() {
        if bytes[start] != b'{' {
            start += 1;
            continue;
        }

        let Some(end_rel) = bytes[start + 1..].iter().position(|byte| *byte == b'}') else {
            break;
        };
        let end = start + 1 + end_rel;
        let candidate = &text[start + 1..end];
        if is_placeholder_name(candidate) {
            placeholders.insert(candidate.to_string());
        }
        start = end + 1;
    }

    placeholders
}

fn is_placeholder_name(candidate: &str) -> bool {
    let mut chars = candidate.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|char| char.is_ascii_alphanumeric() || char == '_')
}

fn collect_locale_json_files(locale_dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(locale_dir)
        .expect("Locale directory should be readable")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().is_some_and(|ext| ext == "json")
                && path
                    .file_name()
                    .is_some_and(|name| name != "languages.json")
        })
        .collect();
    files.sort();
    files
}

fn locale_code_from_path(path: &Path) -> Option<String> {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
}

fn parse_languages_catalog(locale_dir: &Path) -> Result<BTreeSet<String>, Vec<Violation>> {
    let path = locale_dir.join("languages.json");
    let value = parse_json_file(&path)?;
    let Value::Array(entries) = value else {
        return Err(vec![locale_violation(
            &path,
            "languages.json must be a JSON array.".to_string(),
        )]);
    };

    let mut codes = BTreeSet::new();
    let mut violations = Vec::new();

    for (index, entry) in entries.iter().enumerate() {
        let Value::Object(entry_obj) = entry else {
            violations.push(locale_violation(
                &path,
                format!("languages.json entry at index {index} must be an object."),
            ));
            continue;
        };

        let Some(code_value) = entry_obj.get("code") else {
            violations.push(locale_violation(
                &path,
                format!("languages.json entry at index {index} is missing `code`."),
            ));
            continue;
        };
        let Some(name_value) = entry_obj.get("name") else {
            violations.push(locale_violation(
                &path,
                format!("languages.json entry at index {index} is missing `name`."),
            ));
            continue;
        };

        let Value::String(code) = code_value else {
            violations.push(locale_violation(
                &path,
                format!("languages.json entry at index {index} has non-string `code`."),
            ));
            continue;
        };
        let Value::String(_) = name_value else {
            violations.push(locale_violation(
                &path,
                format!("languages.json entry at index {index} has non-string `name`."),
            ));
            continue;
        };

        if !codes.insert(code.clone()) {
            violations.push(locale_violation(
                &path,
                format!("languages.json contains duplicate code `{code}`."),
            ));
        }
    }

    if violations.is_empty() {
        Ok(codes)
    } else {
        Err(violations)
    }
}

fn compare_languages_catalog(
    locale_dir: &Path,
    locale_files: &[PathBuf],
    language_codes: &BTreeSet<String>,
) -> Vec<Violation> {
    let languages_path = locale_dir.join("languages.json");
    let locale_codes: BTreeSet<String> = locale_files
        .iter()
        .filter_map(|path| locale_code_from_path(path))
        .collect();
    let mut violations = Vec::new();

    for code in locale_codes
        .iter()
        .filter(|code| !language_codes.contains(code.as_str()))
    {
        violations.push(locale_violation(
            &languages_path,
            format!("Locale file `{code}.json` exists but is missing from languages.json."),
        ));
    }

    for code in language_codes
        .iter()
        .filter(|code| !locale_codes.contains(code.as_str()))
    {
        violations.push(locale_violation(
            &languages_path,
            format!("Missing locale file `{code}.json` declared in languages.json."),
        ));
    }

    violations
}

fn panic_with_violations(rule_name: &str, hint: &str, violations: &[Violation]) {
    if violations.is_empty() {
        return;
    }

    let report = violations
        .iter()
        .map(|it| it.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    panic!(
        "\n\n🚨 AST Linter [{rule_name}]: Found {} violation(s):\n\n{}\n\n\
        💡 {hint}\n\
        📖 Details: See docs/coding-rules.md\n",
        violations.len(),
        report
    );
}

fn compare_locale_shape(
    file: &Path,
    expected_shape: &BTreeMap<String, JsonNodeKind>,
    actual_shape: &BTreeMap<String, JsonNodeKind>,
) -> Vec<Violation> {
    let mut violations = Vec::new();

    for missing in expected_shape
        .keys()
        .filter(|key| !actual_shape.contains_key(*key))
    {
        violations.push(locale_violation(
            file,
            format!("Missing locale key `{missing}` compared with ja.json/en.json."),
        ));
    }

    for extra in actual_shape
        .keys()
        .filter(|key| !expected_shape.contains_key(*key))
    {
        violations.push(locale_violation(
            file,
            format!("Unexpected locale key `{extra}` not present in ja.json/en.json."),
        ));
    }

    for (path, expected_kind) in expected_shape {
        let Some(actual_kind) = actual_shape.get(path) else {
            continue;
        };
        if actual_kind != expected_kind {
            violations.push(locale_violation(
                file,
                format!(
                    "Locale node kind mismatch at `{path}`: expected {expected_kind}, found {actual_kind}."
                ),
            ));
        }
    }

    violations
}

fn compare_locale_placeholders(
    file: &Path,
    expected_shape: &BTreeMap<String, JsonNodeKind>,
    expected_placeholders: &BTreeMap<String, BTreeSet<String>>,
    actual_placeholders: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<Violation> {
    let mut violations = Vec::new();

    for (path, kind) in expected_shape {
        if kind != &JsonNodeKind::String {
            continue;
        }

        let expected = expected_placeholders.get(path).cloned().unwrap_or_default();
        let actual = actual_placeholders.get(path).cloned().unwrap_or_default();

        if actual != expected {
            violations.push(locale_violation(
                file,
                format!(
                    "Locale placeholder mismatch at `{path}`: expected {:?}, found {:?}.",
                    expected, actual
                ),
            ));
        }
    }

    violations
}

fn build_locale_baseline(
    ja_path: &Path,
    en_path: &Path,
) -> Result<
    (
        BTreeMap<String, JsonNodeKind>,
        BTreeMap<String, BTreeSet<String>>,
    ),
    Vec<Violation>,
> {
    let ja_value = parse_json_file(ja_path)?;
    let en_value = parse_json_file(en_path)?;

    let mut ja_shape = BTreeMap::new();
    let mut en_shape = BTreeMap::new();
    collect_json_shape(&ja_value, None, &mut ja_shape);
    collect_json_shape(&en_value, None, &mut en_shape);

    let mut violations = compare_locale_shape(ja_path, &en_shape, &ja_shape);
    violations.extend(compare_locale_shape(en_path, &ja_shape, &en_shape));

    let mut ja_placeholders = BTreeMap::new();
    let mut en_placeholders = BTreeMap::new();
    collect_json_placeholders(&ja_value, None, &mut ja_placeholders);
    collect_json_placeholders(&en_value, None, &mut en_placeholders);

    violations.extend(compare_locale_placeholders(
        ja_path,
        &en_shape,
        &en_placeholders,
        &ja_placeholders,
    ));
    violations.extend(compare_locale_placeholders(
        en_path,
        &ja_shape,
        &ja_placeholders,
        &en_placeholders,
    ));

    if violations.is_empty() {
        Ok((ja_shape, ja_placeholders))
    } else {
        Err(violations)
    }
}

fn lint_locale_files(locale_dir: &Path) -> Vec<Violation> {
    let locale_files = collect_locale_json_files(locale_dir);
    if locale_files.is_empty() {
        return vec![locale_violation(
            locale_dir,
            format!(
                "No locale JSON files found for analysis: {}",
                locale_dir.display()
            ),
        )];
    }

    let language_codes = match parse_languages_catalog(locale_dir) {
        Ok(codes) => codes,
        Err(violations) => return violations,
    };
    let mut all_violations = compare_languages_catalog(locale_dir, &locale_files, &language_codes);

    let ja_path = locale_dir.join("ja.json");
    let en_path = locale_dir.join("en.json");
    let (baseline_shape, baseline_placeholders) = match build_locale_baseline(&ja_path, &en_path) {
        Ok(baseline) => baseline,
        Err(violations) => {
            all_violations.extend(violations);
            return all_violations;
        }
    };

    for file in locale_files {
        let is_base_locale = file.ends_with("ja.json") || file.ends_with("en.json");
        if is_base_locale {
            continue;
        }

        let value = match parse_json_file(&file) {
            Ok(value) => value,
            Err(violations) => {
                all_violations.extend(violations);
                continue;
            }
        };

        let mut shape = BTreeMap::new();
        let mut placeholders = BTreeMap::new();
        collect_json_shape(&value, None, &mut shape);
        collect_json_placeholders(&value, None, &mut placeholders);

        all_violations.extend(compare_locale_shape(&file, &baseline_shape, &shape));
        all_violations.extend(compare_locale_placeholders(
            &file,
            &baseline_shape,
            &baseline_placeholders,
            &placeholders,
        ));
    }

    all_violations
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MarkdownHeading {
    level: u8,
    line: usize,
}

#[derive(Debug, Clone)]
struct MarkdownPair {
    base: PathBuf,
    ja: PathBuf,
}

fn collect_markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let walker = WalkBuilder::new(root)
        .standard_filters(true)
        .require_git(false)
        .build();

    for entry in walker.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
            files.push(path.to_path_buf());
        }
    }

    files.sort();
    files
}

fn markdown_pair_key(path: &Path) -> Option<(String, bool)> {
    let path_str = path.to_string_lossy();
    if let Some(prefix) = path_str.strip_suffix(".ja.md") {
        return Some((prefix.to_string(), true));
    }
    if let Some(prefix) = path_str.strip_suffix("_ja.md") {
        return Some((prefix.to_string(), true));
    }
    path_str
        .strip_suffix(".md")
        .map(|prefix| (prefix.to_string(), false))
}

fn collect_markdown_pairs(root: &Path) -> Vec<MarkdownPair> {
    let files = collect_markdown_files(root);
    let mut base_files = BTreeMap::<String, PathBuf>::new();
    let mut ja_files = BTreeMap::<String, PathBuf>::new();

    for file in files {
        let (key, is_ja) =
            markdown_pair_key(&file).expect("markdown files should always produce a pair key");
        if is_ja {
            ja_files.insert(key, file);
        } else {
            base_files.insert(key, file);
        }
    }

    let mut pairs = Vec::new();
    for (key, base) in base_files {
        let Some(ja) = ja_files.remove(&key) else {
            continue;
        };
        pairs.push(MarkdownPair { base, ja });
    }

    pairs
}

fn extract_markdown_headings(path: &Path) -> Result<Vec<MarkdownHeading>, Vec<Violation>> {
    let source = std::fs::read_to_string(path).map_err(|err| {
        vec![Violation {
            file: path.to_path_buf(),
            line: 0,
            column: 0,
            message: format!("Markdown file read error: {err}"),
        }]
    })?;

    let mut in_fence = false;
    let mut headings = Vec::new();

    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }

        let hashes = trimmed.chars().take_while(|char| *char == '#').count();
        if !(1..=6).contains(&hashes) {
            continue;
        }

        let rest = &trimmed[hashes..];
        if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
            continue;
        }

        headings.push(MarkdownHeading {
            level: hashes as u8,
            line: index + 1,
        });
    }

    Ok(headings)
}

fn parse_workspace_version_from_cargo_toml(path: &Path) -> Result<String, Vec<Violation>> {
    let source = std::fs::read_to_string(path).map_err(|err| {
        vec![Violation {
            file: path.to_path_buf(),
            line: 0,
            column: 0,
            message: format!("Cargo.toml read error: {err}"),
        }]
    })?;

    let mut in_workspace_package = false;

    for raw_line in source.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_workspace_package = line == "[workspace.package]";
            continue;
        }

        if !in_workspace_package {
            continue;
        }

        let line = line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() != "version" {
            continue;
        }

        let value = value.trim();
        let Some(value) = value.strip_prefix('"').and_then(|it| it.strip_suffix('"')) else {
            return Err(vec![Violation {
                file: path.to_path_buf(),
                line: 0,
                column: 0,
                message: "workspace.package.version must be a TOML string.".to_string(),
            }]);
        };

        return Ok(value.to_string());
    }

    Err(vec![Violation {
        file: path.to_path_buf(),
        line: 0,
        column: 0,
        message: "Missing workspace.package.version in Cargo.toml.".to_string(),
    }])
}

fn changelog_contains_version_heading(path: &Path, version: &str) -> Result<bool, Vec<Violation>> {
    let source = std::fs::read_to_string(path).map_err(|err| {
        vec![Violation {
            file: path.to_path_buf(),
            line: 0,
            column: 0,
            message: format!("CHANGELOG.md read error: {err}"),
        }]
    })?;

    let expected_prefix = format!("## [{version}]");
    Ok(source
        .lines()
        .map(str::trim_start)
        .any(|line| line.starts_with(&expected_prefix)))
}

fn lint_changelog_contains_current_version(root: &Path) -> Vec<Violation> {
    let cargo_toml = root.join("Cargo.toml");
    let changelog = root.join("CHANGELOG.md");

    let version = match parse_workspace_version_from_cargo_toml(&cargo_toml) {
        Ok(version) => version,
        Err(violations) => return violations,
    };

    match changelog_contains_version_heading(&changelog, &version) {
        Ok(true) => Vec::new(),
        Ok(false) => vec![Violation {
            file: changelog.clone(),
            line: 0,
            column: 0,
            message: format!(
                "CHANGELOG.md is missing a release heading for workspace version `{version}`."
            ),
        }],
        Err(violations) => violations,
    }
}

fn compare_markdown_heading_structure(pair: &MarkdownPair) -> Vec<Violation> {
    let base_headings = match extract_markdown_headings(&pair.base) {
        Ok(headings) => headings,
        Err(violations) => return violations,
    };
    let ja_headings = match extract_markdown_headings(&pair.ja) {
        Ok(headings) => headings,
        Err(violations) => return violations,
    };

    let mut violations = Vec::new();

    if base_headings.len() != ja_headings.len() {
        violations.push(locale_violation(
            &pair.ja,
            format!(
                "Markdown heading count mismatch between `{}` and `{}`: {} vs {}.",
                pair.base.display(),
                pair.ja.display(),
                base_headings.len(),
                ja_headings.len()
            ),
        ));
    }

    for (index, (base_heading, ja_heading)) in
        base_headings.iter().zip(ja_headings.iter()).enumerate()
    {
        if base_heading.level != ja_heading.level {
            violations.push(Violation {
                file: pair.ja.clone(),
                line: ja_heading.line,
                column: 1,
                message: format!(
                    "Markdown heading level mismatch at heading {} compared with `{}`: expected H{}, found H{}.",
                    index + 1,
                    pair.base.display(),
                    base_heading.level,
                    ja_heading.level
                ),
            });
        }
    }

    violations
}

fn lint_markdown_heading_pairs(root: &Path) -> Vec<Violation> {
    let mut violations = Vec::new();
    for pair in collect_markdown_pairs(root) {
        violations.extend(compare_markdown_heading_structure(&pair));
    }
    violations
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
    /// Skip `#[cfg(test)]` modules — test code is exempt from i18n rules.
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if has_cfg_test_attr(&node.attrs) {
            return;
        }
        syn::visit::visit_item_mod(self, node);
    }

    /// Inspect method call: `receiver.method(args)`.
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method_name = node.method.to_string();
        if ui_methods().contains(&method_name.as_str()) {
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
    let allowed: Vec<f64> = vec![
        -1.0, 0.0, 1.0, 2.0, // 100 often appears in percentages and scaling
        100.0,
    ];
    allowed.iter().any(|it| (*it - value).abs() < f64::EPSILON)
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
// Lazy Code Detection Visitor
// ─────────────────────────────────────────────

struct LazyCodeVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl LazyCodeVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for LazyCodeVisitor {
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        let segment = mac
            .path
            .segments
            .last()
            .expect("macro path should contain at least one segment");
        let ident = segment.ident.to_string();
        if ident == "todo" || ident == "unimplemented" || ident == "dbg" {
            let (line, column) = span_location(segment.ident.span());
            self.violations.push(Violation {
                file: self.file.clone(),
                line,
                column,
                message: format!(
                    "Lazy code macro `{}!()` detected. Please implement properly instead of deferring.",
                    ident
                ),
            });
        }
        syn::visit::visit_macro(self, mac);
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

/// Apply lazy code rule to a single file and return a list of violations.
fn lint_lazy_code(path: &Path, syntax: &syn::File) -> Vec<Violation> {
    let mut visitor = LazyCodeVisitor::new(path.to_path_buf());
    visitor.visit_file(syntax);
    visitor.violations
}

// ─────────────────────────────────────────────
// Prohibited Types Detection Visitor
// ─────────────────────────────────────────────

struct ProhibitedTypesVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl ProhibitedTypesVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for ProhibitedTypesVisitor {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if has_cfg_test_attr(&node.attrs) {
            return;
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
            return;
        }
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
        let segment = node
            .path
            .segments
            .last()
            .expect("type path should contain at least one segment");
        if segment.ident == "HashMap" {
            let (line, column) = span_location(segment.ident.span());
            self.violations.push(Violation {
                file: self.file.clone(),
                line,
                column,
                message:
                    "Prohibited type `HashMap` detected. Please use `Vec` or a typed struct instead."
                        .to_string(),
            });
        }
        syn::visit::visit_type_path(self, node);
    }

    fn visit_type_array(&mut self, node: &'ast syn::TypeArray) {
        use syn::spanned::Spanned;
        let (line, column) = span_location(node.span());
        self.violations.push(Violation {
            file: self.file.clone(),
            line,
            column,
            message: "Fixed-length array `[T; N]` detected. Please use `Vec<T>` instead."
                .to_string(),
        });
        syn::visit::visit_type_array(self, node);
    }

    fn visit_expr_array(&mut self, node: &'ast syn::ExprArray) {
        use syn::spanned::Spanned;
        let (line, column) = span_location(node.span());
        self.violations.push(Violation {
            file: self.file.clone(),
            line,
            column,
            message: "Array literal `[...]` detected. Please use `vec![...]` instead.".to_string(),
        });
        syn::visit::visit_expr_array(self, node);
    }
}

/// Apply prohibited types rule to a single file and return a list of violations.
fn lint_prohibited_types(path: &Path, syntax: &syn::File) -> Vec<Violation> {
    let mut visitor = ProhibitedTypesVisitor::new(path.to_path_buf());
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

fn is_raw_icon(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed == "x" || trimmed == "X" {
        return true;
    }
    trimmed.chars().any(is_emoji_or_symbol)
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

/// Apply icon facade rule to a single file and return a list of violations.
fn lint_icon_facade(path: &Path, syntax: &syn::File) -> Vec<Violation> {
    let mut visitor = IconFacadeVisitor::new(path.to_path_buf());
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
    panic_with_violations(rule_name, hint, &all_violations);
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

/// Lazy code rule: Detect todo!(), unimplemented!(), and dbg!() macros.
/// Scope: All crates.
#[test]
fn ast_linter_no_lazy_code() {
    let root = workspace_root();
    run_ast_lint(
        "lazy-code",
        "Fix: Remove `todo!()`, `unimplemented!()`, and `dbg!()` macros. Implement the actual logic.",
        &[
            root.join("crates/katana-core/src"),
            root.join("crates/katana-platform/src"),
            root.join("crates/katana-ui/src"),
        ],
        lint_lazy_code,
    );
}

/// Prohibited types rule: Detect HashMap, [T; N] and array literals.
/// Scope: All crates.
#[test]
fn ast_linter_no_prohibited_types() {
    let root = workspace_root();
    run_ast_lint(
        "prohibited-types",
        "Fix: Use `Vec` instead of `HashMap`, `[T; N]` or `[...]`.",
        &[
            root.join("crates/katana-core/src"),
            root.join("crates/katana-platform/src"),
            root.join("crates/katana-ui/src"),
        ],
        lint_prohibited_types,
    );
}

/// Locale rule: ensure locale JSON files stay aligned with ja/en structure and placeholders.
#[test]
fn ast_linter_locale_files_match_base_structure() {
    let locale_dir = workspace_root().join("crates/katana-ui/locales");
    let all_violations = lint_locale_files(&locale_dir);
    panic_with_violations(
        "locale-structure",
        "Fix: Keep every locale JSON aligned with ja.json/en.json, including placeholder names.",
        &all_violations,
    );
}

/// Icon facade rule: Detect raw icon strings like "🔄", "x", "▶".
/// Scope: All crates.
#[test]
fn ast_linter_no_raw_icons() {
    let root = workspace_root();
    run_ast_lint(
        "icon-facade",
        "Fix: Use `Icon::Name.as_str()` instead of raw icon string literals like \"🔄\".",
        &[
            root.join("crates/katana-core/src"),
            root.join("crates/katana-platform/src"),
            root.join("crates/katana-ui/src"),
        ],
        lint_icon_facade,
    );
}

#[test]
fn ast_linter_markdown_heading_pairs_match() {
    let all_violations = lint_markdown_heading_pairs(workspace_root());
    panic_with_violations(
        "markdown-heading-structure",
        "Fix: Keep each *.md and corresponding .ja/_ja markdown file aligned by heading count and heading levels.",
        &all_violations,
    );
}

#[test]
fn ast_linter_changelog_contains_current_workspace_version() {
    let all_violations = lint_changelog_contains_current_version(workspace_root());
    panic_with_violations(
        "changelog-version-sync",
        "Fix: Add a `## [x.y.z]` release heading to CHANGELOG.md that matches workspace.package.version in Cargo.toml.",
        &all_violations,
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
    use serde_json::json;
    use std::path::PathBuf;

    fn cfg_test_attr_for_item(item: &syn::Item) -> bool {
        match item {
            syn::Item::Mod(m) => has_cfg_test_attr(&m.attrs),
            _ => false,
        }
    }

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

        // Explicitly instantiate and visit to strictly enforce line 1082 coverage
        let mut visitor = MagicNumberVisitor::new(PathBuf::from("fake.rs"));
        visitor.visit_file(&syntax);
    }

    #[test]
    fn test_has_cfg_test_attr_returns_true_for_cfg_test() {
        let code = r#"
            #[cfg(test)]
            mod dummy {}
            #[cfg(target_os = "macos")]
            mod dummy_mac {}
        "#;
        let syntax = syn::parse_file(code).unwrap();
        let has_test = cfg_test_attr_for_item(&syntax.items[0]);
        let has_mac = cfg_test_attr_for_item(&syntax.items[1]);
        assert!(has_test);
        assert!(!has_mac);
    }

    #[test]
    fn test_has_cfg_test_attr_returns_false_for_non_mod_items() {
        let code = r#"
            fn dummy() {}
            const VALUE: usize = 1;
        "#;
        let syntax = syn::parse_file(code).unwrap();
        assert!(!cfg_test_attr_for_item(&syntax.items[0]));
        assert!(!cfg_test_attr_for_item(&syntax.items[1]));
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

    // ── lint_lazy_code: LazyCodeVisitor coverage ──

    #[test]
    fn lint_lazy_code_detects_todo_macro() {
        let code = r#"fn foo() { todo!(); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_lazy_code(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("todo!()"));
    }

    #[test]
    fn lint_lazy_code_detects_unimplemented_macro() {
        let code = r#"fn foo() { unimplemented!(); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_lazy_code(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("unimplemented!()"));
    }

    #[test]
    fn lint_lazy_code_detects_dbg_macro() {
        let code = r#"fn foo() { dbg!(42); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_lazy_code(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("dbg!()"));
    }

    #[test]
    fn lint_lazy_code_allows_normal_macros() {
        let code = r#"fn foo() { println!("ok"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_lazy_code(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    // ── lint_prohibited_types: ProhibitedTypesVisitor coverage ──

    #[test]
    fn lint_prohibited_types_detects_hashmap() {
        let code = r#"fn foo() { let map: std::collections::HashMap<i32, i32> = std::collections::HashMap::new(); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_prohibited_types(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.iter().any(|v| v.message.contains("HashMap")));
    }

    #[test]
    fn lint_prohibited_types_detects_type_array() {
        let code = r#"fn foo() { let arr: [i32; 4] = [1, 2, 3, 4]; }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_prohibited_types(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations
            .iter()
            .any(|v| v.message.contains("Fixed-length array")));
    }

    #[test]
    fn lint_prohibited_types_detects_expr_array() {
        let code = r#"fn foo() { let arr = [1, 2, 3]; }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_prohibited_types(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations
            .iter()
            .any(|v| v.message.contains("Array literal")));
    }

    // ── lint_icon_facade: IconFacadeVisitor coverage ──

    #[test]
    fn lint_icon_facade_detects_raw_icon_in_button() {
        let code =
            r#"fn render(ui: &mut Ui) { ui.button("🔄"); ui.label(("x")); ui.button(&"▶"); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_icon_facade(&PathBuf::from("fake.rs"), &syntax);
        assert_eq!(violations.len(), 3);
        assert!(violations
            .iter()
            .any(|v| v.message.contains("Raw icon string")));
    }

    #[test]
    fn lint_icon_facade_allows_regular_symbols() {
        let code = r#"fn render(ui: &mut Ui) { ui.label("/"); ui.label("..."); }"#;
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_icon_facade(&PathBuf::from("fake.rs"), &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn extract_placeholders_detects_named_tokens_only() {
        let placeholders = extract_placeholders(
            "Size: {size} B\nModified: {mod_time}\nfn main() { println!(\"Hello\"); }",
        );
        assert_eq!(
            placeholders,
            BTreeSet::from(["mod_time".to_string(), "size".to_string()])
        );
    }

    #[test]
    fn compare_locale_shape_detects_missing_keys() {
        let expected = BTreeMap::from([
            ("menu".to_string(), JsonNodeKind::Object),
            ("menu.file".to_string(), JsonNodeKind::String),
        ]);
        let actual = BTreeMap::from([("menu".to_string(), JsonNodeKind::Object)]);
        let violations = compare_locale_shape(Path::new("locale.json"), &expected, &actual);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("menu.file"));
    }

    #[test]
    fn json_node_kind_scalar_variants_and_display_are_covered() {
        let cases = [
            (json!({}), JsonNodeKind::Object, "object"),
            (json!([1, 2]), JsonNodeKind::Array(2), "array(len=2)"),
            (json!("text"), JsonNodeKind::String, "string"),
            (json!(1), JsonNodeKind::Number, "number"),
            (json!(true), JsonNodeKind::Bool, "bool"),
            (Value::Null, JsonNodeKind::Null, "null"),
        ];

        for (value, expected_kind, expected_display) in cases {
            let actual_kind = JsonNodeKind::from_value(&value);
            assert_eq!(actual_kind, expected_kind);
            assert_eq!(actual_kind.to_string(), expected_display);
        }
    }

    #[test]
    fn parse_json_file_returns_read_error_for_nonexistent_file() {
        let result = parse_json_file(Path::new("/nonexistent/locale.json"));
        let errors = result.expect_err("missing JSON file should fail");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Locale file read error"));
    }

    #[test]
    fn parse_json_file_returns_parse_error_for_invalid_json() {
        let tmp = tempfile::NamedTempFile::with_suffix(".json").unwrap();
        std::fs::write(tmp.path(), "{ invalid").unwrap();
        let result = parse_json_file(tmp.path());
        let errors = result.expect_err("invalid JSON should fail");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("Locale JSON parse error"));
    }

    #[test]
    fn collect_json_placeholders_ignores_non_string_values() {
        let value = json!({
            "text": "Hello {name}",
            "count": 3,
            "enabled": true,
            "missing": null
        });
        let mut placeholders = BTreeMap::new();
        collect_json_placeholders(&value, None, &mut placeholders);

        assert_eq!(
            placeholders.get("text"),
            Some(&BTreeSet::from(["name".to_string()]))
        );
        assert!(!placeholders.contains_key("count"));
        assert!(!placeholders.contains_key("enabled"));
        assert!(!placeholders.contains_key("missing"));
    }

    #[test]
    fn compare_locale_placeholders_detects_mismatch() {
        let expected_shape =
            BTreeMap::from([("status.save_failed".to_string(), JsonNodeKind::String)]);
        let expected_placeholders = BTreeMap::from([(
            "status.save_failed".to_string(),
            BTreeSet::from(["error".to_string()]),
        )]);
        let actual_placeholders = BTreeMap::from([(
            "status.save_failed".to_string(),
            BTreeSet::from(["message".to_string()]),
        )]);

        let violations = compare_locale_placeholders(
            Path::new("locale.json"),
            &expected_shape,
            &expected_placeholders,
            &actual_placeholders,
        );
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("status.save_failed"));
    }

    #[test]
    fn collect_json_shape_and_placeholders_cover_root_array_paths() {
        let value = json!([{"message": "Hello {name}"}]);
        let mut shape = BTreeMap::new();
        let mut placeholders = BTreeMap::new();
        collect_json_shape(&value, None, &mut shape);
        collect_json_placeholders(&value, None, &mut placeholders);

        assert_eq!(shape.get("[0]"), Some(&JsonNodeKind::Object));
        assert_eq!(shape.get("[0].message"), Some(&JsonNodeKind::String));
        assert_eq!(
            placeholders.get("[0].message"),
            Some(&BTreeSet::from(["name".to_string()]))
        );
    }

    #[test]
    fn collect_json_placeholders_ignores_root_string_without_path() {
        let value = json!("Hello {name}");
        let mut placeholders = BTreeMap::new();
        collect_json_placeholders(&value, None, &mut placeholders);
        assert!(placeholders.is_empty());
    }

    #[test]
    fn extract_placeholders_handles_unclosed_and_empty_placeholders() {
        assert!(extract_placeholders("Hello {name").is_empty());
        assert!(extract_placeholders("{}").is_empty());
    }

    #[test]
    fn compare_locale_shape_detects_extra_keys_and_kind_mismatch() {
        let expected = BTreeMap::from([
            ("menu".to_string(), JsonNodeKind::Object),
            ("menu.file".to_string(), JsonNodeKind::String),
        ]);
        let actual = BTreeMap::from([
            ("menu".to_string(), JsonNodeKind::String),
            ("menu.file".to_string(), JsonNodeKind::String),
            ("menu.extra".to_string(), JsonNodeKind::String),
        ]);

        let violations = compare_locale_shape(Path::new("locale.json"), &expected, &actual);
        assert_eq!(violations.len(), 2);
        assert!(violations.iter().any(|v| v.message.contains("menu.extra")));
        assert!(violations
            .iter()
            .any(|v| v.message.contains("expected object")));
    }

    #[test]
    fn build_locale_baseline_returns_errors_for_mismatched_bases() {
        let tmp = tempfile::TempDir::new().unwrap();
        let ja_path = tmp.path().join("ja.json");
        let en_path = tmp.path().join("en.json");
        std::fs::write(
            &ja_path,
            r#"{"status":{"saved":"保存しました。","failed":"失敗: {error}"}}"#,
        )
        .unwrap();
        std::fs::write(
            &en_path,
            r#"{"status":{"saved":"Saved.","failed":"Failed: {message}"}}"#,
        )
        .unwrap();

        let violations =
            build_locale_baseline(&ja_path, &en_path).expect_err("base locales should mismatch");
        assert!(!violations.is_empty());
        assert!(violations
            .iter()
            .any(|v| v.message.contains("Locale placeholder mismatch")));
    }

    #[test]
    fn lint_locale_files_reports_empty_directory() {
        let tmp = tempfile::TempDir::new().unwrap();
        let violations = lint_locale_files(tmp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0]
            .message
            .contains("No locale JSON files found for analysis"));
    }

    #[test]
    fn lint_locale_files_reports_parse_errors_in_non_base_locale() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("ja.json"),
            r#"{"menu":{"file":"ファイル","open_all":"全て開く"}}"#,
        )
        .unwrap();
        std::fs::write(
            tmp.path().join("en.json"),
            r#"{"menu":{"file":"File","open_all":"Open All"}}"#,
        )
        .unwrap();
        std::fs::write(
            tmp.path().join("languages.json"),
            r#"[{"code":"en","name":"English"},{"code":"ja","name":"日本語"},{"code":"de","name":"Deutsch"}]"#,
        )
        .unwrap();
        std::fs::write(tmp.path().join("de.json"), "{ invalid").unwrap();

        let violations = lint_locale_files(tmp.path());
        assert!(violations
            .iter()
            .any(|v| v.message.contains("Locale JSON parse error")));
    }

    #[test]
    fn lint_locale_files_reports_base_locale_mismatch() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("ja.json"),
            r#"{"status":{"saved":"保存しました。","failed":"失敗: {error}"}}"#,
        )
        .unwrap();
        std::fs::write(
            tmp.path().join("en.json"),
            r#"{"status":{"saved":"Saved.","failed":"Failed: {message}"}}"#,
        )
        .unwrap();
        std::fs::write(
            tmp.path().join("de.json"),
            r#"{"status":{"saved":"Gespeichert.","failed":"Fehlgeschlagen: {error}"}}"#,
        )
        .unwrap();
        std::fs::write(
            tmp.path().join("languages.json"),
            r#"[{"code":"en","name":"English"},{"code":"ja","name":"日本語"},{"code":"de","name":"Deutsch"}]"#,
        )
        .unwrap();

        let violations = lint_locale_files(tmp.path());
        assert!(!violations.is_empty());
        assert!(violations
            .iter()
            .any(|v| v.message.contains("Locale placeholder mismatch")));
    }

    #[test]
    fn lint_locale_files_reports_locale_file_missing_from_languages_catalog() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("ja.json"),
            r#"{"menu":{"file":"ファイル","open_all":"全て開く"}}"#,
        )
        .unwrap();
        std::fs::write(
            tmp.path().join("en.json"),
            r#"{"menu":{"file":"File","open_all":"Open All"}}"#,
        )
        .unwrap();
        std::fs::write(
            tmp.path().join("de.json"),
            r#"{"menu":{"file":"Datei","open_all":"Alle öffnen"}}"#,
        )
        .unwrap();
        std::fs::write(
            tmp.path().join("languages.json"),
            r#"[{"code":"en","name":"English"},{"code":"ja","name":"日本語"}]"#,
        )
        .unwrap();

        let violations = lint_locale_files(tmp.path());
        assert!(violations
            .iter()
            .any(|v| v.message.contains("de.json") && v.message.contains("languages.json")));
    }

    #[test]
    fn lint_locale_files_reports_catalog_entry_missing_locale_file() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("ja.json"),
            r#"{"menu":{"file":"ファイル","open_all":"全て開く"}}"#,
        )
        .unwrap();
        std::fs::write(
            tmp.path().join("en.json"),
            r#"{"menu":{"file":"File","open_all":"Open All"}}"#,
        )
        .unwrap();
        std::fs::write(
            tmp.path().join("languages.json"),
            r#"[{"code":"en","name":"English"},{"code":"ja","name":"日本語"},{"code":"de","name":"Deutsch"}]"#,
        )
        .unwrap();

        let violations = lint_locale_files(tmp.path());
        assert!(violations
            .iter()
            .any(|v| v.message.contains("de") && v.message.contains("Missing locale file")));
    }

    #[test]
    fn parse_languages_catalog_rejects_non_array_root() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join("languages.json"), r#"{"code":"en"}"#).unwrap();

        let violations =
            parse_languages_catalog(tmp.path()).expect_err("non-array root should fail");
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("must be a JSON array"));
    }

    #[test]
    fn parse_languages_catalog_propagates_read_errors() {
        let tmp = tempfile::TempDir::new().unwrap();
        let violations =
            parse_languages_catalog(tmp.path()).expect_err("missing languages.json should fail");
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Locale file read error"));
    }

    #[test]
    fn parse_languages_catalog_validates_entry_shape_and_duplicates() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("languages.json"),
            r#"[null,{},{"code":"en"},{"name":"English"},{"code":1,"name":"English"},{"code":"ja","name":1},{"code":"en","name":"English"},{"code":"en","name":"English Duplicate"}]"#,
        )
        .unwrap();

        let violations =
            parse_languages_catalog(tmp.path()).expect_err("invalid entries should fail");
        assert!(violations
            .iter()
            .any(|v| v.message.contains("must be an object")));
        assert!(violations
            .iter()
            .any(|v| v.message.contains("missing `code`")));
        assert!(violations
            .iter()
            .any(|v| v.message.contains("missing `name`")));
        assert!(violations
            .iter()
            .any(|v| v.message.contains("non-string `code`")));
        assert!(violations
            .iter()
            .any(|v| v.message.contains("non-string `name`")));
        assert!(violations
            .iter()
            .any(|v| v.message.contains("duplicate code `en`")));
    }

    #[test]
    fn lint_locale_files_returns_languages_catalog_errors_early() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("ja.json"),
            r#"{"menu":{"file":"ファイル","open_all":"全て開く"}}"#,
        )
        .unwrap();
        std::fs::write(
            tmp.path().join("en.json"),
            r#"{"menu":{"file":"File","open_all":"Open All"}}"#,
        )
        .unwrap();
        std::fs::write(tmp.path().join("languages.json"), r#"{"code":"en"}"#).unwrap();

        let violations = lint_locale_files(tmp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("must be a JSON array"));
    }

    #[test]
    fn markdown_pair_key_supports_base_and_japanese_suffixes() {
        assert_eq!(
            markdown_pair_key(Path::new("docs/guide.md")),
            Some(("docs/guide".to_string(), false))
        );
        assert_eq!(
            markdown_pair_key(Path::new("docs/guide.ja.md")),
            Some(("docs/guide".to_string(), true))
        );
        assert_eq!(
            markdown_pair_key(Path::new("docs/guide_ja.md")),
            Some(("docs/guide".to_string(), true))
        );
    }

    #[test]
    fn collect_markdown_pairs_respects_gitignore_and_pairs_dynamically() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("ignored")).unwrap();
        std::fs::write(tmp.path().join(".gitignore"), "ignored/\n").unwrap();
        std::fs::write(tmp.path().join("README.md"), "# Title\n## Section\n").unwrap();
        std::fs::write(
            tmp.path().join("README.ja.md"),
            "# タイトル\n## セクション\n",
        )
        .unwrap();
        std::fs::write(tmp.path().join("guide_ja.md"), "# ガイド\n").unwrap();
        std::fs::write(tmp.path().join("ignored/IGNORED.md"), "# Ignored\n").unwrap();
        std::fs::write(tmp.path().join("ignored/IGNORED.ja.md"), "# 無視\n").unwrap();

        let pairs = collect_markdown_pairs(tmp.path());
        assert_eq!(pairs.len(), 1);
        assert!(pairs[0].base.ends_with("README.md"));
        assert!(pairs[0].ja.ends_with("README.ja.md"));
    }

    #[test]
    fn collect_rs_files_excludes_tests_directory() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();
        std::fs::create_dir_all(tmp.path().join("tests")).unwrap();
        std::fs::write(tmp.path().join("src/lib.rs"), "fn main() {}\n").unwrap();
        std::fs::write(tmp.path().join("tests/ignored.rs"), "fn helper() {}\n").unwrap();

        let files = collect_rs_files(tmp.path());
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("src/lib.rs"));
    }

    #[test]
    fn collect_markdown_pairs_includes_changelog_pair() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join("CHANGELOG.md"), "# Changelog\n").unwrap();
        std::fs::write(tmp.path().join("CHANGELOG.ja.md"), "# 変更履歴\n").unwrap();

        let pairs = collect_markdown_pairs(tmp.path());
        assert_eq!(pairs.len(), 1);
        assert!(pairs[0].base.ends_with("CHANGELOG.md"));
        assert!(pairs[0].ja.ends_with("CHANGELOG.ja.md"));
    }

    #[test]
    fn extract_markdown_headings_ignores_code_fences_and_non_headings() {
        let tmp = tempfile::NamedTempFile::with_suffix(".md").unwrap();
        std::fs::write(
            tmp.path(),
            "# Title\n#[test]\n```md\n## ignored\n```\n### Section\n",
        )
        .unwrap();

        let headings = extract_markdown_headings(tmp.path()).unwrap();
        assert_eq!(
            headings,
            vec![
                MarkdownHeading { level: 1, line: 1 },
                MarkdownHeading { level: 3, line: 6 },
            ]
        );
    }

    #[test]
    fn compare_markdown_heading_structure_detects_count_and_level_mismatch() {
        let tmp = tempfile::TempDir::new().unwrap();
        let base = tmp.path().join("doc.md");
        let ja = tmp.path().join("doc.ja.md");
        std::fs::write(&base, "# Title\n## Section\n").unwrap();
        std::fs::write(&ja, "# タイトル\n### 節\n#### 追加\n").unwrap();

        let pair = MarkdownPair { base, ja };
        let violations = compare_markdown_heading_structure(&pair);
        assert_eq!(violations.len(), 2);
        assert!(violations
            .iter()
            .any(|v| v.message.contains("heading count mismatch")));
        assert!(violations
            .iter()
            .any(|v| v.message.contains("heading level mismatch")));
    }

    #[test]
    fn lint_markdown_heading_pairs_reports_read_errors() {
        let tmp = tempfile::TempDir::new().unwrap();
        let base = tmp.path().join("doc.md");
        let ja = tmp.path().join("doc.ja.md");
        std::fs::write(&base, "# Title\n").unwrap();
        std::fs::write(&ja, "# タイトル\n").unwrap();
        std::fs::remove_file(&ja).unwrap();

        let pair = MarkdownPair { base, ja };
        let violations = compare_markdown_heading_structure(&pair);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Markdown file read error"));
    }

    #[test]
    fn compare_markdown_heading_structure_reports_base_read_errors() {
        let tmp = tempfile::TempDir::new().unwrap();
        let base = tmp.path().join("doc.md");
        let ja = tmp.path().join("doc.ja.md");
        std::fs::write(&base, "# Title\n").unwrap();
        std::fs::write(&ja, "# タイトル\n").unwrap();
        std::fs::remove_file(&base).unwrap();

        let pair = MarkdownPair { base, ja };
        let violations = compare_markdown_heading_structure(&pair);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Markdown file read error"));
    }

    #[test]
    fn parse_workspace_version_from_cargo_toml_extracts_workspace_package_version() {
        let tmp = tempfile::NamedTempFile::with_suffix(".toml").unwrap();
        std::fs::write(
            tmp.path(),
            r#"[workspace]
members = ["crates/foo"]

[workspace.package]
version = "1.2.3"
edition = "2021"
"#,
        )
        .unwrap();

        let version = parse_workspace_version_from_cargo_toml(tmp.path()).unwrap();
        assert_eq!(version, "1.2.3");
    }

    #[test]
    fn parse_workspace_version_from_cargo_toml_skips_blank_invalid_and_non_version_lines() {
        let tmp = tempfile::NamedTempFile::with_suffix(".toml").unwrap();
        std::fs::write(
            tmp.path(),
            r#"[workspace]
members = ["crates/foo"]

[workspace.package]
# comment
edition
name = "katana"
version = "1.2.3"
"#,
        )
        .unwrap();

        let version = parse_workspace_version_from_cargo_toml(tmp.path()).unwrap();
        assert_eq!(version, "1.2.3");
    }

    #[test]
    fn parse_workspace_version_from_cargo_toml_reports_read_error_for_missing_file() {
        let violations =
            parse_workspace_version_from_cargo_toml(Path::new("/nonexistent/Cargo.toml"))
                .expect_err("missing Cargo.toml should fail");
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Cargo.toml read error"));
    }

    #[test]
    fn parse_workspace_version_from_cargo_toml_reports_non_string_version() {
        let tmp = tempfile::NamedTempFile::with_suffix(".toml").unwrap();
        std::fs::write(
            tmp.path(),
            r#"[workspace.package]
version = 123
"#,
        )
        .unwrap();

        let violations = parse_workspace_version_from_cargo_toml(tmp.path())
            .expect_err("non-string version should fail");
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("must be a TOML string"));
    }

    #[test]
    fn parse_workspace_version_from_cargo_toml_reports_missing_workspace_package_version() {
        let tmp = tempfile::NamedTempFile::with_suffix(".toml").unwrap();
        std::fs::write(
            tmp.path(),
            r#"[workspace]
members = ["crates/foo"]
"#,
        )
        .unwrap();

        let violations = parse_workspace_version_from_cargo_toml(tmp.path())
            .expect_err("missing workspace.package version should fail");
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("workspace.package.version"));
    }

    #[test]
    fn build_locale_baseline_propagates_read_errors_for_missing_base_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        let ja_path = tmp.path().join("ja.json");
        let en_path = tmp.path().join("en.json");
        std::fs::write(&ja_path, r#"{"status":{"saved":"保存しました。"}}"#).unwrap();

        let violations =
            build_locale_baseline(&ja_path, &en_path).expect_err("missing en.json should fail");
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Locale file read error"));
    }

    #[test]
    fn build_locale_baseline_propagates_read_errors_for_missing_ja_file() {
        let tmp = tempfile::TempDir::new().unwrap();
        let ja_path = tmp.path().join("ja.json");
        let en_path = tmp.path().join("en.json");
        std::fs::write(&en_path, r#"{"status":{"saved":"Saved."}}"#).unwrap();

        let violations =
            build_locale_baseline(&ja_path, &en_path).expect_err("missing ja.json should fail");
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Locale file read error"));
    }

    #[test]
    fn lint_changelog_contains_current_version_propagates_cargo_read_errors() {
        let tmp = tempfile::TempDir::new().unwrap();
        let violations = lint_changelog_contains_current_version(tmp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Cargo.toml read error"));
    }

    #[test]
    fn lint_changelog_contains_current_version_reports_missing_heading() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            r#"[workspace]
members = ["crates/foo"]

[workspace.package]
version = "2.0.0"
"#,
        )
        .unwrap();
        std::fs::write(
            tmp.path().join("CHANGELOG.md"),
            r#"# Changelog

## [1.9.9] - 2026-03-21
"#,
        )
        .unwrap();

        let violations = lint_changelog_contains_current_version(tmp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("2.0.0"));
        assert!(violations[0].message.contains("CHANGELOG.md"));
    }

    #[test]
    fn lint_changelog_contains_current_version_reports_changelog_read_errors() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            r#"[workspace]
members = ["crates/foo"]

[workspace.package]
version = "2.0.0"
"#,
        )
        .unwrap();

        let violations = lint_changelog_contains_current_version(tmp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("CHANGELOG.md read error"));
    }
}
