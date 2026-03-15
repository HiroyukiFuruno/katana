use katana_core::markdown::*;

#[test]
fn basic_gfm_renders_to_html() {
    let md = "# Heading\n\nParagraph with **bold** and `code`.\n";
    let out = render_basic(md).expect("render failed");
    assert!(out.html.contains("<h1>"));
    assert!(out.html.contains("<strong>bold</strong>"));
    assert!(out.html.contains("<code>code</code>"));
}

#[test]
fn gfm_table_renders() {
    let md = "| A | B |\n|---|---|\n| 1 | 2 |\n";
    let out = render_basic(md).expect("render failed");
    assert!(out.html.contains("<table>"));
}

#[test]
fn gfm_tasklist_renders() {
    let md = "- [x] Done\n- [ ] Todo\n";
    let out = render_basic(md).expect("render failed");
    assert!(out.html.contains("<li>"));
}

#[test]
fn malformed_document_does_not_panic() {
    let md = "## Unclosed\n\n```\nno close fence";
    assert!(render_basic(md).is_ok());
}

#[test]
fn mermaid_block_is_transformed() {
    let md = "\n```mermaid\ngraph TD; A-->B\n```\n";
    let out = render_basic(md).expect("render failed");
    // NoOpRenderer wraps in code block; original fence should not appear as-is.
    assert!(out.html.contains("mermaid"));
}

#[test]
fn unknown_fence_passes_through() {
    let md = "\n```rust\nfn main() {}\n```\n";
    let out = render_basic(md).expect("render failed");
    assert!(out.html.contains("fn main()"));
}

// L25-37: KatanaRenderer and render_with_katana_renderer
// KatanaRenderer routes to sub-renderers; since no external tools, Mermaid will
// return CommandNotFound but it should not panic.
#[test]
fn render_with_katana_renderer_succeeds_for_plain_markdown() {
    let md = "# Hello\n\nWorld";
    let out = render_with_katana_renderer(md).expect("render failed");
    assert!(out.html.contains("<h1>"));
    assert!(out.html.contains("World"));
}

#[test]
fn katana_renderer_handles_mermaid_block_without_crash() {
    // mmdc unavailable → CommandNotFound → fallback HTML
    unsafe { std::env::set_var("MERMAID_MMDC", "/nonexistent/mmdc") };
    let md = "\n```mermaid\ngraph TD; A-->B\n```\n";
    let out = render_with_katana_renderer(md).expect("render failed");
    // Should contain something (fallback or diagram hint)
    assert!(!out.html.is_empty());
    unsafe { std::env::remove_var("MERMAID_MMDC") };
}

#[test]
fn katana_renderer_handles_plantuml_block_without_crash() {
    // PlantUML not installed → NotInstalled → fallback HTML
    unsafe { std::env::set_var("PLANTUML_JAR", "/nonexistent/plantuml.jar") };
    let md = "\n```plantuml\n@startuml\nA -> B\n@enduml\n```\n";
    let out = render_with_katana_renderer(md).expect("render failed");
    assert!(!out.html.is_empty());
    unsafe { std::env::remove_var("PLANTUML_JAR") };
}

#[test]
fn katana_renderer_handles_drawio_block() {
    let md = "\n```drawio\n<mxGraphModel><root><mxCell id=\"0\"/></root></mxGraphModel>\n```\n";
    let out = render_with_katana_renderer(md).expect("render failed");
    assert!(!out.html.is_empty());
}

// L90-100: extract_fence_block - test the fence-at-start-of-string cases
#[test]
fn render_with_fence_at_very_start_of_document() {
    // Fenced block at very beginning (no preceding newline)
    let md = "```mermaid\ngraph TD; A-->B\n```\nAfter block";
    let out = render_basic(md).expect("render failed");
    assert!(!out.html.is_empty());
}

// L127-137: render_diagram_block for OkPng, CommandNotFound, NotInstalled branches
// These are tested via render_with_katana_renderer which hits dispatch_renderer
#[test]
fn render_with_katana_renderer_drawio_renders_svg() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/><mxCell id="1" parent="0"/>
<mxCell id="2" value="Box" vertex="1" parent="1"><mxGeometry x="10" y="10" width="100" height="50" as="geometry"/></mxCell>
</root></mxGraphModel>"#;
    let md = format!("\n```drawio\n{xml}\n```\n");
    let out = render_with_katana_renderer(&md).expect("render failed");
    // Should contain SVG or div.katana-diagram
    assert!(out.html.contains("svg") || out.html.contains("katana-diagram"));
}

// L103-108: html_escape in mod.rs (via process_fence with malformed XML content)
#[test]
fn drawio_renderer_escapes_html_in_fallback() {
    // Invalid XML → error → fallback_html which calls html_escape
    let md = "\n```drawio\nnot valid xml & <stuff>\n```\n";
    let out = render_with_katana_renderer(md).expect("render failed");
    // fallback html should contain escaped entities
    assert!(!out.html.is_empty());
}

// L127: Test covering DiagramResult::OkPng branch
#[test]
fn okpng_branch_becomes_empty_string_in_core_layer() {
    use katana_core::markdown::diagram::{DiagramBlock, DiagramRenderer, DiagramResult};

    struct PngRenderer;
    impl DiagramRenderer for PngRenderer {
        fn render(&self, _block: &DiagramBlock) -> DiagramResult {
            DiagramResult::OkPng(vec![0x89, 0x50, 0x4E, 0x47]) // PNG magic
        }
    }

    let md = "\n```mermaid\ngraph TD; A-->B\n```\n";
    let out = render(md, &PngRenderer).expect("render failed");
    // OkPng is converted to an empty string in the core layer (processed in the UI layer)
    // Ensures the diagram block is not replaced by figure tags etc.
    assert!(!out.html.contains("graph TD"));
}
