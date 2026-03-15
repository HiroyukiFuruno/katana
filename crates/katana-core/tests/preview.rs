use katana_core::markdown::diagram::DiagramKind;
use katana_core::preview::*;

#[test]
fn プレーンmarkdownはひとつのセクションになる() {
    let src = "# Hello\n\nWorld";
    let sections = split_into_sections(src);
    assert_eq!(sections.len(), 1);
    assert!(matches!(sections[0], PreviewSection::Markdown(_)));
}

#[test]
fn mermaidフェンスはdiagramセクションに分割される() {
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
fn 不明なフェンスはmarkdownとして残る() {
    let src = "intro\n```rust\nfn main() {}\n```\nfin";
    let sections = split_into_sections(src);
    // rust フェンスはダイアグラムではないのですべて Markdown セクションに含まれる。
    assert!(sections
        .iter()
        .all(|s| matches!(s, PreviewSection::Markdown(_))));
}

#[test]
fn 複数ダイアグラムが正しく分割される() {
    let src = "A\n```mermaid\ngraph TD; A-->B\n```\nB\n```drawio\n<mxGraphModel/>\n```\nC";
    let sections = split_into_sections(src);
    let diagram_count = sections
        .iter()
        .filter(|s| matches!(s, PreviewSection::Diagram { .. }))
        .count();
    assert_eq!(diagram_count, 2);
}
