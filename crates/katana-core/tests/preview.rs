use katana_core::markdown::diagram::DiagramKind;
use katana_core::preview::*;

#[test]
fn plain_markdown_becomes_one_section() {
    let src = "# Hello\n\nWorld";
    let sections = split_into_sections(src);
    assert_eq!(sections.len(), 1);
    assert!(matches!(sections[0], PreviewSection::Markdown(_)));
}

#[test]
fn mermaid_fence_is_split_into_diagram_section() {
    let src = "before\n```mermaid\ngraph TD; A-->B\n```\nafter";
    let sections = split_into_sections(src);
    assert_eq!(sections.len(), 3);
    assert!(matches!(sections[0], PreviewSection::Markdown(_)));
    assert!(matches!(
        sections[1],
        PreviewSection::Diagram {
            kind: DiagramKind::Mermaid,
            ..
        }
    ));
    assert!(matches!(sections[2], PreviewSection::Markdown(_)));
}

#[test]
fn unknown_fence_remains_as_markdown() {
    let src = "intro\n```rust\nfn main() {}\n```\nfin";
    let sections = split_into_sections(src);
    // rust fence is not a diagram so it is fully included in Markdown section.
    assert!(sections
        .iter()
        .all(|s| matches!(s, PreviewSection::Markdown(_))));
}

#[test]
fn multiple_diagrams_are_split_correctly() {
    let src = "A\n```mermaid\ngraph TD; A-->B\n```\nB\n```drawio\n<mxGraphModel/>\n```\nC";
    let sections = split_into_sections(src);
    let diagram_count = sections
        .iter()
        .filter(|s| matches!(s, PreviewSection::Diagram { .. }))
        .count();
    assert_eq!(diagram_count, 2);
}

// try_parse_diagram_fence: no closing fence -> None path (? at L65)
#[test]
fn diagram_without_closing_fence_remains_as_markdown() {
    // Starts with ``` but lacks closing ``` -> does not become Diagram
    let src = "before\n```mermaid\ngraph TD; A-->B";
    let sections = split_into_sections(src);
    // Treats everything as Markdown
    assert!(sections
        .iter()
        .all(|s| matches!(s, PreviewSection::Markdown(_))));
}

// try_parse_diagram_fence: no newline in info line -> None path (? at L61)
// Case where there is nothing immediately after ``` at end of file
#[test]
fn markdown_if_fence_ends_without_newline() {
    // No info line after ``` (EOF)
    let src = "text\n```";
    let sections = split_into_sections(src);
    assert!(sections
        .iter()
        .all(|s| matches!(s, PreviewSection::Markdown(_))));
}

// Diagram fence at the very start of the file (no preceding newline).
#[test]
fn diagram_fence_at_start_of_file_is_detected() {
    let src = "```mermaid\ngraph TD; A-->B\n```\n";
    let sections = split_into_sections(src);
    assert_eq!(sections.len(), 1);
    assert!(matches!(
        sections[0],
        PreviewSection::Diagram {
            kind: DiagramKind::Mermaid,
            ..
        }
    ));
}

// Diagram fence at the start with trailing text.
#[test]
fn diagram_fence_at_start_with_trailing_text() {
    let src = "```mermaid\ngraph TD; A-->B\n```\nSome text after";
    let sections = split_into_sections(src);
    assert_eq!(sections.len(), 2);
    assert!(matches!(
        sections[0],
        PreviewSection::Diagram {
            kind: DiagramKind::Mermaid,
            ..
        }
    ));
    assert!(matches!(sections[1], PreviewSection::Markdown(_)));
}

// ── resolve_image_paths tests ────────────────────────────────────────────

use katana_core::preview::resolve_image_paths;
use std::path::Path;

#[test]
fn resolve_image_paths_converts_relative_to_absolute() {
    let dir = tempfile::tempdir().unwrap();
    let md_path = dir.path().join("docs").join("readme.md");
    std::fs::create_dir_all(md_path.parent().unwrap()).unwrap();
    // Create the image file so canonicalize works.
    let img_dir = dir.path().join("assets");
    std::fs::create_dir_all(&img_dir).unwrap();
    std::fs::write(img_dir.join("logo.png"), b"png").unwrap();

    let source = "![logo](../assets/logo.png)";
    let (result, paths) = resolve_image_paths(source, &md_path);
    assert!(result.starts_with("![logo](file://"));
    assert!(result.contains("assets/logo.png"));
    assert!(!result.contains(".."));
    assert_eq!(paths.len(), 1);
}

#[test]
fn resolve_image_paths_preserves_http_urls() {
    let source = "![img](https://example.com/image.png)";
    let (result, _) = resolve_image_paths(source, Path::new("/tmp/test.md"));
    assert_eq!(result, source);
}

#[test]
fn resolve_image_paths_preserves_absolute_paths() {
    let source = "![img](/absolute/path/image.png)";
    let (result, _) = resolve_image_paths(source, Path::new("/tmp/test.md"));
    assert_eq!(result, source);
}

#[test]
fn resolve_image_paths_preserves_file_uris() {
    let source = "![img](file:///some/image.png)";
    let (result, _) = resolve_image_paths(source, Path::new("/tmp/test.md"));
    assert_eq!(result, source);
}

#[test]
fn resolve_image_paths_handles_no_closing_paren() {
    let source = "![alt](path/without/close";
    let (result, _) = resolve_image_paths(source, Path::new("/tmp/test.md"));
    assert!(result.contains("![alt]("));
}

#[test]
fn resolve_image_paths_handles_no_alt_close_bracket() {
    let source = "![alt without close bracket";
    let (result, _) = resolve_image_paths(source, Path::new("/tmp/test.md"));
    assert_eq!(result, source);
}

#[test]
fn resolve_image_paths_passes_through_non_image_text() {
    let source = "# Hello\n\nSome text without images.";
    let (result, _) = resolve_image_paths(source, Path::new("/tmp/test.md"));
    assert_eq!(result, source);
}

// ── flatten_list_code_blocks tests ───────────────────────────────────────

use katana_core::preview::flatten_list_code_blocks;

#[test]
fn flatten_strips_indented_code_fence_in_list() {
    let source = "1. Step one\n\n   ```bash\n   echo hello\n   ```\n\n1. Step two\n";
    let result = flatten_list_code_blocks(source);
    assert!(
        result.contains("\n```bash\n"),
        "Opening fence should be de-indented: {result:?}"
    );
    assert!(
        result.contains("echo hello\n```\n"),
        "Content and closing fence should be de-indented: {result:?}"
    );
}

#[test]
fn flatten_leaves_toplevel_code_block_unchanged() {
    let source = "# Title\n\n```rust\nfn main() {}\n```\n";
    let result = flatten_list_code_blocks(source);
    assert_eq!(result, source);
}

#[test]
fn flatten_handles_multiple_code_blocks_in_one_item() {
    let source = "1. Step\n\n   ```bash\n   cmd1\n   ```\n\n   ```bash\n   cmd2\n   ```\n";
    let result = flatten_list_code_blocks(source);
    // Both fences should be de-indented.
    let count = result.matches("\n```bash\n").count()
        + if result.starts_with("```bash\n") {
            1
        } else {
            0
        };
    assert!(
        count >= 2,
        "Expected 2 de-indented fences, got {count} in: {result:?}"
    );
}

#[test]
fn flatten_preserves_no_trailing_newline() {
    let source = "text\n   ```\n   code\n   ```";
    let result = flatten_list_code_blocks(source);
    assert!(
        !result.ends_with("\n\n"),
        "Should not add extra trailing newline"
    );
}

#[test]
fn html_blocks_remain_in_markdown_sections_for_pulldown_cmark() {
    // After the render_html_fn refactor, flush_markdown no longer splits HTML blocks.
    // HTML blocks remain inside Markdown sections and are handled by pulldown-cmark
    // via egui_commonmark's render_html_fn callback at render time.
    let src = "text\n<p align=\"center\">\n  <img src=\"a.png\" alt=\"A\">\n</p>\nmore text\n<h1 align=\"center\">Title</h1>";
    let sections = split_into_sections(src);
    // All content should be in a single Markdown section (no diagram fences present).
    assert_eq!(sections.len(), 1);
    if let PreviewSection::Markdown(ref s) = sections[0] {
        assert!(s.contains("text"), "Expected 'text' in markdown");
        assert!(
            s.contains("<p align=\"center\">"),
            "HTML blocks must stay inside Markdown section"
        );
        assert!(
            s.contains("<img src=\"a.png\""),
            "Expected img tag in markdown"
        );
        assert!(
            s.contains("<h1 align=\"center\">Title</h1>"),
            "Expected h1 tag in markdown"
        );
        assert!(s.contains("more text"), "Expected 'more text' in markdown");
    } else {
        panic!("Expected Markdown section");
    }
}

// ── resolve_html_image_paths tests ──

use katana_core::preview::resolve_html_image_paths;

#[test]
fn resolve_html_image_relative_path_to_file_uri() {
    let html = r#"<img src="logo.png" alt="Logo">"#;
    let result = resolve_html_image_paths(html, Path::new("/project/README.md"));
    assert!(
        result.contains("file://"),
        "Relative img src must be resolved to file:// URI, got: {result}"
    );
    assert!(
        result.contains("logo.png"),
        "Resolved path must contain original filename, got: {result}"
    );
}

#[test]
fn resolve_html_image_absolute_url_unchanged() {
    let html = r#"<img src="https://example.com/badge.svg" alt="badge">"#;
    let result = resolve_html_image_paths(html, Path::new("/project/README.md"));
    assert_eq!(result, html, "Absolute URL must not be modified");
}

#[test]
fn resolve_html_image_file_uri_unchanged() {
    let html = r#"<img src="file:///absolute/path/logo.png" alt="Logo">"#;
    let result = resolve_html_image_paths(html, Path::new("/project/README.md"));
    assert_eq!(result, html, "file:// URI must not be modified");
}

#[test]
fn resolve_html_image_absolute_path_unchanged() {
    let html = r#"<img src="/usr/share/image.png" alt="img">"#;
    let result = resolve_html_image_paths(html, Path::new("/project/README.md"));
    assert_eq!(result, html, "Absolute path must not be modified");
}

#[test]
fn resolve_html_image_multiple_tags() {
    let html = r#"<img src="a.png" alt="A"><img src="b.png" alt="B">"#;
    let result = resolve_html_image_paths(html, Path::new("/project/README.md"));
    // Both relative images should be resolved
    let file_uri_count = result.matches("file://").count();
    assert_eq!(
        file_uri_count, 2,
        "Both img tags should be resolved, got: {result}"
    );
}

// ── wrap_standalone_inline_html tests ──

use katana_core::preview::wrap_standalone_inline_html;

#[test]
fn wrap_standalone_a_tag() {
    let src = r#"<a href="foo"><img src="bar"></a>"#;
    let expected = format!("<div>\n{src}\n</div>");
    assert_eq!(wrap_standalone_inline_html(src), expected);
}

#[test]
fn wrap_standalone_img_tag() {
    let src = r#"<img src="https://example.com/badge.svg">"#;
    let expected = format!("<div>\n{src}\n</div>");
    assert_eq!(wrap_standalone_inline_html(src), expected);
}

#[test]
fn do_not_wrap_if_surrounded_by_text() {
    let src = r#"Here is a <a href="foo">link</a> inside text."#;
    assert_eq!(wrap_standalone_inline_html(src), src);
}

#[test]
fn wrap_standalone_with_leading_whitespace() {
    let src = "  <a href=\"foo\">link</a>  ";
    let expected = "<div>\n<a href=\"foo\">link</a>\n</div>".to_string();
    assert_eq!(wrap_standalone_inline_html(src), expected);
}

#[test]
fn verify_wrapped_html_is_parsed_as_html_block() {
    use pulldown_cmark::{Event, Parser, Tag, TagEnd};

    // Simulate what flush_markdown does before passing to egui_commonmark
    let src = r#"<a href="foo"><img src="bar"></a>"#;
    let processed = wrap_standalone_inline_html(src);

    let parser = Parser::new(&processed);
    let mut found_html_block = false;
    for event in parser {
        if let Event::Start(Tag::HtmlBlock) | Event::End(TagEnd::HtmlBlock) | Event::Html(_) = event
        {
            found_html_block = true;
        }
    }

    assert!(
        found_html_block,
        "The wrapped HTML must be recognised as an HtmlBlock by pulldown-cmark so that render_html_fn can handle it"
    );
}

/// Verifies that real README-style HTML patterns are correctly parsed by pulldown-cmark
/// as HtmlBlock events (not InlineHtml), which is the prerequisite for render_html_fn
/// to handle them. This test simulates what egui_commonmark does: accumulate all
/// Event::Html chunks within a HtmlBlock, then pass the accumulated string to render_html_fn.
#[test]
fn readme_html_patterns_parsed_as_html_blocks_for_render_html_fn() {
    use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

    // Reproduce the exact structure from README.md
    let source = r#"<p align="center">
  <img src="assets/icon.iconset/icon_128x128.png" width="128" alt="KatanA Desktop">
</p>

<h1 align="center">KatanA Desktop</h1>

<p align="center">
  A fast, lightweight Markdown workspace for macOS — built with Rust and egui.
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
  <a href="https://github.com/repo/actions"><img src="https://github.com/repo/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
</p>

<p align="center">
  English | <a href="README.ja.md">日本語</a>
</p>

---

## What is KatanA

Some text here.
"#;

    // Process through wrap_standalone_inline_html (same as flush_markdown does)
    let processed = wrap_standalone_inline_html(source);

    // Simulate egui_commonmark's HTML block accumulation
    let parser = Parser::new_ext(&processed, Options::all());
    let mut html_blocks: Vec<String> = Vec::new();
    let mut in_html_block = false;
    let mut current_html = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::HtmlBlock) => {
                in_html_block = true;
                current_html.clear();
            }
            Event::Html(text) => {
                if in_html_block {
                    current_html.push_str(&text);
                }
            }
            Event::End(TagEnd::HtmlBlock) => {
                html_blocks.push(current_html.clone());
                in_html_block = false;
            }
            Event::InlineHtml(text) => {
                panic!(
                    "HTML should NOT be parsed as InlineHtml (would bypass render_html_fn): {:?}",
                    text.as_ref()
                );
            }
            _ => {}
        }
    }

    // Verify: we expect at least 5 HTML blocks from the README header
    // (icon <p>, <h1>, description <p>, badges <p>, language <p>)
    assert!(
        html_blocks.len() >= 5,
        "Expected at least 5 HTML blocks from README header, got {}: {:?}",
        html_blocks.len(),
        html_blocks
    );

    // Verify badge block: must contain the complete <p> with all <a><img> children
    let badge_block = html_blocks
        .iter()
        .find(|b| b.contains("img.shields.io"))
        .expect("Badge HTML block must exist");
    assert!(
        badge_block.contains(r#"<p align="center">"#),
        "Badge block must start with <p align=\"center\">, got: {badge_block}"
    );
    assert!(
        badge_block.contains("</p>"),
        "Badge block must contain closing </p>, got: {badge_block}"
    );
    assert!(
        badge_block.contains(r#"<a href="LICENSE">"#),
        "Badge block must contain license link, got: {badge_block}"
    );

    // Verify icon block
    let icon_block = html_blocks
        .iter()
        .find(|b| b.contains("icon_128x128"))
        .expect("Icon HTML block must exist");
    assert!(
        icon_block.contains(r#"<p align="center">"#),
        "Icon block must be inside centered paragraph"
    );

    // Verify centered heading - find block containing <h1
    let heading_block = html_blocks
        .iter()
        .find(|b| b.contains("<h1"))
        .expect("Heading HTML block must exist");
    assert!(
        heading_block.contains("KatanA Desktop"),
        "Heading must contain title text, got: {heading_block}"
    );
}

/// Verifies that a standalone sponsor badge link (outside <p>) is correctly
/// wrapped and parsed as HtmlBlock after wrap_standalone_inline_html processing.
#[test]
fn standalone_sponsor_badge_parsed_as_html_block_after_wrapping() {
    use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

    let source = r#"Some text here.

<a href="https://github.com/sponsors/HiroyukiFuruno"><img src="https://img.shields.io/badge/Sponsor.svg" alt="Sponsor"></a>

More text.
"#;

    let processed = wrap_standalone_inline_html(source);

    let parser = Parser::new_ext(&processed, Options::all());
    let mut html_blocks: Vec<String> = Vec::new();
    let mut in_html_block = false;
    let mut current_html = String::new();
    let mut found_inline_html = false;

    for event in parser {
        match event {
            Event::Start(Tag::HtmlBlock) => {
                in_html_block = true;
                current_html.clear();
            }
            Event::Html(text) => {
                if in_html_block {
                    current_html.push_str(&text);
                }
            }
            Event::End(TagEnd::HtmlBlock) => {
                html_blocks.push(current_html.clone());
                in_html_block = false;
            }
            Event::InlineHtml(_) => {
                found_inline_html = true;
            }
            _ => {}
        }
    }

    assert!(
        !found_inline_html,
        "Standalone <a><img></a> must NOT be parsed as InlineHtml after wrapping"
    );
    assert!(
        !html_blocks.is_empty(),
        "Standalone sponsor badge must be parsed as an HTML block"
    );
    let sponsor_block = html_blocks
        .iter()
        .find(|b| b.contains("Sponsor"))
        .expect("Sponsor HTML block must exist");
    assert!(
        sponsor_block.contains("<a href="),
        "Sponsor block must contain the link tag"
    );
}

/// Tags inside <p> blocks must NOT be wrapped with <div>.
/// The <p> itself is already a block element recognised by pulldown-cmark.
/// Wrapping its children breaks inline layout (badges won't be horizontal).
#[test]
fn do_not_wrap_tags_inside_p_block() {
    let src = concat!(
        "<p align=\"center\">\n",
        "  <a href=\"LICENSE\"><img src=\"badge1.svg\" alt=\"License: MIT\"></a>\n",
        "  <a href=\"https://example.com/ci\"><img src=\"badge2.svg\" alt=\"CI\"></a>\n",
        "  <img src=\"badge3.svg\" alt=\"Platform\">\n",
        "</p>\n"
    );
    // The content inside <p> should remain unchanged
    assert_eq!(
        wrap_standalone_inline_html(src),
        src,
        "Tags inside <p> blocks must not be wrapped with <div>"
    );
}

/// Tags inside <h1> blocks must NOT be wrapped with <div>.
#[test]
fn do_not_wrap_tags_inside_heading_block() {
    let src = "<h1 align=\"center\">Title</h1>\n";
    assert_eq!(
        wrap_standalone_inline_html(src),
        src,
        "Content inside <h1> must not be modified"
    );
}
