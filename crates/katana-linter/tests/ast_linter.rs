use katana_linter::rules::domains::changelog::lint_changelog_contains_current_version;
use katana_linter::rules::domains::i18n::lint_i18n;
use katana_linter::rules::domains::i18n::lint_icon_facade;
use katana_linter::rules::domains::locales::lint_locale_files;
use katana_linter::rules::domains::markdown::lint_markdown_heading_pairs;
use katana_linter::rules::{
    lint_comment_style, lint_error_first, lint_file_length, lint_font_normalization,
    lint_function_length, lint_lazy_code, lint_magic_numbers, lint_nesting_depth, lint_performance,
    lint_prohibited_attributes, lint_prohibited_types, lint_pub_free_fn,
};
use katana_linter::run_ast_lint;
use katana_linter::utils::{panic_with_violations, workspace_root};

fn target_crates(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    vec![
        root.join("crates/katana-linter/src"),
        root.join("crates/katana-core/src"),
        // root.join("crates/katana-platform/src"), // Phase 5
        // root.join("crates/katana-ui/src"),       // Phase 6
    ]
}

#[test]
fn ast_linter_i18n_no_hardcoded_strings() {
    let root = workspace_root().expect("Test requirement");
    run_ast_lint(
        "i18n",
        "Fix: Replace string literals with i18n::t(\"key\") or i18n::tf(\"key\", &[...]).",
        &target_crates(root),
        lint_i18n,
    );
}

#[test]
fn ast_linter_no_magic_numbers() {
    let root = workspace_root().expect("Test requirement");
    run_ast_lint(
        "magic-number",
        "Fix: Extract numeric literals into named constants (const).",
        &target_crates(root),
        lint_magic_numbers,
    );
}

#[test]
fn ast_linter_no_lazy_code() {
    let root = workspace_root().expect("Test requirement");
    run_ast_lint(
        "lazy-code",
        "Fix: Remove `todo!()`, `unimplemented!()`, and `dbg!()` macros. Implement the actual logic.",
        &target_crates(root),
        lint_lazy_code,
    );
}

#[test]
fn ast_linter_no_prohibited_types() {
    let root = workspace_root().expect("Test requirement");
    run_ast_lint(
        "prohibited-types",
        "Fix: Use `Vec` instead of `HashMap`, `[T; N]` or `[...]`.",
        &target_crates(root),
        lint_prohibited_types,
    );
}

#[test]
fn ast_linter_locale_files_match_base_structure() {
    let locale_dir = workspace_root()
        .expect("Test requirement")
        .join("crates/katana-ui/locales");
    let all_violations = lint_locale_files(&locale_dir);
    panic_with_violations(
        "locale-structure",
        "Fix: Keep every locale JSON aligned with ja.json/en.json, including placeholder names.",
        &all_violations,
    );
}

#[test]
fn ast_linter_no_raw_icons() {
    let root = workspace_root().expect("Test requirement");
    run_ast_lint(
        "icon-facade",
        "Fix: Use `Icon::Name.as_str()` instead of raw icon string literals like \"🔄\".",
        &target_crates(root),
        lint_icon_facade,
    );
}

#[test]
fn ast_linter_markdown_heading_pairs_match() {
    let all_violations = lint_markdown_heading_pairs(workspace_root().expect("Test requirement"));
    panic_with_violations(
        "markdown-heading-structure",
        "Fix: Keep each *.md and corresponding .ja/_ja markdown file aligned by heading count and heading levels.",
        &all_violations,
    );
}

#[test]
fn ast_linter_changelog_contains_current_workspace_version() {
    let all_violations =
        lint_changelog_contains_current_version(workspace_root().expect("Test requirement"));
    panic_with_violations(
        "changelog-version-sync",
        "Fix: Add a `## [x.y.z]` release heading to CHANGELOG.md that matches workspace.package.version in Cargo.toml.",
        &all_violations,
    );
}

#[test]
fn ast_linter_font_normalization() {
    let root = workspace_root().expect("Test requirement");
    run_ast_lint(
        "font-normalization",
        "Fix: Use `NormalizeFonts` from `font_loader` instead of raw `FontDefinitions::default()`/`::empty()`.",
        &target_crates(root),
        lint_font_normalization,
    );
}

#[test]
fn ast_linter_no_unoptimized_performance() {
    let root = workspace_root().expect("Test requirement");
    run_ast_lint(
        "performance",
        "Fix: Avoid unconditional `request_repaint()` or `set_title()` calls in UI loops.",
        &[root.join("crates/katana-ui/src")],
        lint_performance,
    );
}

#[test]
fn ast_linter_no_allow_dead_code() {
    let root = workspace_root().expect("Test requirement");
    run_ast_lint(
        "prohibited-attributes",
        "Fix: Remove `#[allow(dead_code)]`. Dead code should be deleted, not silenced.",
        &target_crates(root),
        lint_prohibited_attributes,
    );
}

#[test]
fn ast_linter_file_length() {
    let root = workspace_root().expect("Test requirement");
    run_ast_lint(
        "file-length",
        "Fix: File exceeds 200-line limit (excluding tests). Split into smaller modules.",
        &target_crates(root),
        lint_file_length,
    );
}

#[test]
fn ast_linter_function_length() {
    let root = workspace_root().expect("Test requirement");
    run_ast_lint(
        "function-length",
        "Fix: Function exceeds 30-line limit. Extract helper methods.",
        &target_crates(root),
        lint_function_length,
    );
}

#[test]
fn ast_linter_nesting_depth() {
    let root = workspace_root().expect("Test requirement");
    run_ast_lint(
        "nesting-depth",
        "Fix: Nesting depth exceeds 3 levels. Use early returns or extract helpers.",
        &target_crates(root),
        lint_nesting_depth,
    );
}

#[test]
fn ast_linter_comment_style() {
    let root = workspace_root().expect("Test requirement");
    run_ast_lint(
        "comment-style",
        "Fix: Comments must start with `// WHY:` or `// SAFETY:`. Code should be self-documenting.",
        &target_crates(root),
        lint_comment_style,
    );
}

#[test]
fn ast_linter_error_first() {
    let root = workspace_root().expect("Test requirement");
    run_ast_lint(
        "error-first",
        "Fix: Do not nest success paths with `if let Ok(...)`. Use `?` or `let-else` to fail early.",
        &target_crates(root),
        lint_error_first,
    );
}

#[test]
#[ignore] // WHY: existing codebase has widespread pub free functions; enable after refactoring
fn ast_linter_no_pub_free_fn() {
    let root = workspace_root().expect("Test requirement");
    run_ast_lint(
        "pub-free-fn",
        "Fix: Public free functions are prohibited. Use struct + impl blocks (coding-rules §1.1).",
        &target_crates(root),
        lint_pub_free_fn,
    );
}

#[test]
fn ast_linter_no_unused_theme_colors() {
    let all_violations = katana_linter::rules::domains::theme::lint_unused_theme_colors(
        workspace_root().expect("Test requirement"),
    );
    panic_with_violations(
        "unused-theme-colors",
        "Fix: A theme color property is defined in `ThemeColors` but never accessed in UI code. Please use it or remove it.",
        &all_violations,
    );
}

#[test]
fn ast_linter_no_hardcoded_colors() {
    let all_violations = katana_linter::rules::domains::theme::lint_no_hardcoded_colors(
        workspace_root().expect("Test requirement"),
    );
    panic_with_violations(
        "hardcoded-colors",
        "Fix: A hardcoded UI color was found. Map it to a property in `ThemeColors` and use `theme_bridge::rgb_to_color32`.",
        &all_violations,
    );
}

#[test]
fn ast_linter_theme_builder_enforcement() {
    let all_violations = katana_linter::rules::domains::theme::lint_theme_builder_enforcement(
        workspace_root().expect("Test requirement"),
    );
    panic_with_violations(
        "theme-builder-enforcement",
        "Fix: Theme presets must use `ThemePresetBuilder::new(...)` to enforce DRY design. Do not instantiate `PresetColorData` directly.",
        &all_violations,
    );
}

#[test]
fn ast_linter_no_japanese_in_crates() {
    use ignore::WalkBuilder;
    let root = workspace_root().expect("Test requirement").join("crates");

    // We walk in parallel for maximum performance without degrading test speed
    let (tx, rx) = std::sync::mpsc::channel();

    let walker = WalkBuilder::new(root).build_parallel();

    walker.run(|| {
        let tx = tx.clone();
        Box::new(move |result| {
            if let Ok(entry) = result {
                let path = entry.path();
                if path.is_file() {
                    // Exclude ja.json (the only allowed source of Japanese truth)
                    if path.file_name().is_some_and(|name| name == "ja.json") {
                        return ignore::WalkState::Continue;
                    }

                    if let Ok(content) = std::fs::read_to_string(path) {
                        for (line_idx, line) in content.lines().enumerate() {
                            // Detect Japanese specifically combining Hiragana and Katakana.
                            // We intentionally exclude pure Han ideographs (\p{Han}) because Katana includes Chinese locales (zh-TW, zh-CN)
                            // which must not trigger the Japanese check.
                            if line.chars().any(|c| matches!(c, '\u{3040}'..='\u{309F}' | '\u{30A0}'..='\u{30FF}')) {
                                let _ = tx.send(format!("{}:{}: Please remove Japanese text or use Unicode escapes for test strings.", path.display(), line_idx + 1));
                                break;
                            }
                        }
                    }
                }
            }
            ignore::WalkState::Continue
        })
    });

    drop(tx); // Close the transmitter

    let mut violations = Vec::new();
    for failure in rx {
        violations.push(katana_linter::Violation {
            file: std::path::PathBuf::from(""), // dummy
            line: 0,
            column: 0,
            message: failure,
        });
    }

    if !violations.is_empty() {
        panic_with_violations(
            "no-japanese-in-workspace",
            "Fix: No Japanese text (Hiragana/Katakana) is allowed in any files except ja.json. Please translate comments to English or use Unicode escapes for test data.",
            &violations,
        );
    }
}
