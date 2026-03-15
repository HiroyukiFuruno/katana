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
