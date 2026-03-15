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
