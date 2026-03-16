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
    let result = resolve_image_paths(source, &md_path);
    assert!(result.starts_with("![logo](file://"));
    assert!(result.contains("assets/logo.png"));
    assert!(!result.contains(".."));
}

#[test]
fn resolve_image_paths_preserves_http_urls() {
    let source = "![img](https://example.com/image.png)";
    let result = resolve_image_paths(source, Path::new("/tmp/test.md"));
    assert_eq!(result, source);
}

#[test]
fn resolve_image_paths_preserves_absolute_paths() {
    let source = "![img](/absolute/path/image.png)";
    let result = resolve_image_paths(source, Path::new("/tmp/test.md"));
    assert_eq!(result, source);
}

#[test]
fn resolve_image_paths_preserves_file_uris() {
    let source = "![img](file:///some/image.png)";
    let result = resolve_image_paths(source, Path::new("/tmp/test.md"));
    assert_eq!(result, source);
}

#[test]
fn resolve_image_paths_handles_no_closing_paren() {
    let source = "![alt](path/without/close";
    let result = resolve_image_paths(source, Path::new("/tmp/test.md"));
    assert!(result.contains("![alt]("));
}

#[test]
fn resolve_image_paths_handles_no_alt_close_bracket() {
    let source = "![alt without close bracket";
    let result = resolve_image_paths(source, Path::new("/tmp/test.md"));
    assert_eq!(result, source);
}

#[test]
fn resolve_image_paths_passes_through_non_image_text() {
    let source = "# Hello\n\nSome text without images.";
    let result = resolve_image_paths(source, Path::new("/tmp/test.md"));
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
