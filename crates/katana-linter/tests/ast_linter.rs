use katana_linter::rules::changelog::lint_changelog_contains_current_version;
use katana_linter::rules::i18n::lint_i18n;
use katana_linter::rules::i18n::lint_icon_facade;
use katana_linter::rules::locales::lint_locale_files;
use katana_linter::rules::markdown::lint_markdown_heading_pairs;
use katana_linter::rules::rust::{
    lint_font_normalization, lint_lazy_code, lint_magic_numbers, lint_performance,
    lint_prohibited_types,
};
use katana_linter::run_ast_lint;
use katana_linter::utils::{panic_with_violations, workspace_root};

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

#[test]
fn ast_linter_font_normalization() {
    let root = workspace_root();
    run_ast_lint(
        "font-normalization",
        "Fix: Use `NormalizeFonts` from `font_loader` instead of raw `FontDefinitions::default()`/`::empty()`.",
        &[
            root.join("crates/katana-core/src"),
            root.join("crates/katana-platform/src"),
            root.join("crates/katana-ui/src"),
        ],
        lint_font_normalization,
    );
}

#[test]
fn ast_linter_no_unoptimized_performance() {
    let root = workspace_root();
    run_ast_lint(
        "performance",
        "Fix: Avoid unconditional `request_repaint()` or `set_title()` calls in UI loops.",
        &[root.join("crates/katana-ui/src")],
        lint_performance,
    );
}

#[test]
fn ast_linter_no_unused_theme_colors() {
    let all_violations = katana_linter::rules::theme::lint_unused_theme_colors(workspace_root());
    panic_with_violations(
        "unused-theme-colors",
        "Fix: A theme color property is defined in `ThemeColors` but never accessed in UI code. Please use it or remove it.",
        &all_violations,
    );
}

#[test]
fn ast_linter_no_hardcoded_colors() {
    let all_violations = katana_linter::rules::theme::lint_no_hardcoded_colors(workspace_root());
    panic_with_violations(
        "hardcoded-colors",
        "Fix: A hardcoded UI color was found. Map it to a property in `ThemeColors` and use `theme_bridge::rgb_to_color32`.",
        &all_violations,
    );
}

#[test]
fn ast_linter_theme_builder_enforcement() {
    let all_violations =
        katana_linter::rules::theme::lint_theme_builder_enforcement(workspace_root());
    panic_with_violations(
        "theme-builder-enforcement",
        "Fix: Theme presets must use `ThemePresetBuilder::new(...)` to enforce DRY design. Do not instantiate `PresetColorData` directly.",
        &all_violations,
    );
}

#[test]
fn ast_linter_no_japanese_in_crates() {
    use ignore::WalkBuilder;
    let root = workspace_root().join("crates");

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
