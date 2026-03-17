use egui_kittest::Harness;
use katana_core::markdown::svg_rasterize::RasterizedSvg;
use katana_ui::preview_pane::{decode_png_rgba, extract_svg, PreviewPane, RenderedSection};
use std::path::PathBuf;

/// Helper: Extract Markdown text from RenderedSection.
fn markdown_texts(sections: &[RenderedSection]) -> Vec<&str> {
    sections
        .iter()
        .filter_map(|s| match s {
            RenderedSection::Markdown(md) => Some(md.as_str()),
            _ => None,
        })
        .collect()
}

// ── 3.2 Preview Synchronization: Immediate preview update from unsaved buffer ──

#[test]
fn unsaved_buffer_changes_are_reflected_in_preview() {
    let mut pane = PreviewPane::default();

    // Build preview with initial content
    pane.update_markdown_sections("# Hello", std::path::Path::new("/tmp/test.md"));
    assert_eq!(pane.sections.len(), 1);
    let texts = markdown_texts(&pane.sections);
    assert!(texts[0].contains("# Hello"));

    // Update buffer without saving to file -> Should be reflected in preview
    pane.update_markdown_sections(
        "# Hello World\n\nNew paragraph",
        std::path::Path::new("/tmp/test.md"),
    );
    let texts = markdown_texts(&pane.sections);
    assert!(texts[0].contains("# Hello World"));
    assert!(texts[0].contains("New paragraph"));
}

#[test]
fn consecutive_edits_are_immediately_reflected_in_preview() {
    let mut pane = PreviewPane::default();

    // Multiple consecutive edits are all reflected
    let edits = vec![
        "# Draft 1",
        "# Draft 2\n\n- item A",
        "# Draft 3\n\n- item A\n- item B\n- item C",
    ];

    for edit in &edits {
        pane.update_markdown_sections(edit, std::path::Path::new("/tmp/test.md"));
        let texts = markdown_texts(&pane.sections);
        assert!(
            texts[0].contains(edit),
            "Edit not reflected in preview: {edit}"
        );
    }
}

#[test]
fn empty_buffer_does_not_crash_preview() {
    let mut pane = PreviewPane::default();

    // Input content
    pane.update_markdown_sections("# Hello", std::path::Path::new("/tmp/test.md"));
    assert_eq!(pane.sections.len(), 1);

    // Revert to empty -> Section count becomes 0 (empty string is not flushed)
    pane.update_markdown_sections("", std::path::Path::new("/tmp/test.md"));
    assert_eq!(pane.sections.len(), 0);
}

#[test]
fn buffer_with_diagrams_immediately_updates_markdown_portion_only() {
    let mut pane = PreviewPane::default();

    // Initial content with diagram
    let source = "# Title\n```mermaid\ngraph TD; A-->B\n```\n## Footer";
    pane.full_render(source, std::path::Path::new("/tmp/test.md"));

    // Diagram is in Pending state
    assert!(pane.sections.len() >= 3);
    assert!(matches!(pane.sections[1], RenderedSection::Pending { .. }));

    // Update only the Markdown portion (diagram is retained)
    let modified = "# Updated Title\n```mermaid\ngraph TD; A-->B\n```\n## Updated Footer";
    pane.update_markdown_sections(modified, std::path::Path::new("/tmp/test.md"));

    // Verify Markdown text is updated
    let texts = markdown_texts(&pane.sections);
    assert!(texts.iter().any(|t| t.contains("Updated Title")));
    assert!(texts.iter().any(|t| t.contains("Updated Footer")));
}

#[test]
fn full_render_splits_sections_correctly() {
    let mut pane = PreviewPane::default();

    let source = "Before\n```mermaid\ngraph TD; A-->B\n```\nAfter";
    pane.full_render(source, std::path::Path::new("/tmp/test.md"));

    // 3 sections: Markdown, Diagram(Pending), Markdown
    assert_eq!(pane.sections.len(), 3);
    assert!(matches!(pane.sections[0], RenderedSection::Markdown(_)));
    assert!(matches!(pane.sections[1], RenderedSection::Pending { .. }));
    assert!(matches!(pane.sections[2], RenderedSection::Markdown(_)));
}

#[test]
fn buffer_without_diagrams_does_not_generate_pending_sections() {
    let mut pane = PreviewPane::default();

    pane.full_render(
        "# Pure Markdown\n\nNo diagrams here.",
        std::path::Path::new("/tmp/test.md"),
    );

    assert!(pane
        .sections
        .iter()
        .all(|s| matches!(s, RenderedSection::Markdown(_))));
    assert!(!pane
        .sections
        .iter()
        .any(|s| matches!(s, RenderedSection::Pending { .. })));
}

#[test]
fn verification_that_preview_updates_do_not_depend_on_file_saves() {
    // Integration test for Document + PreviewPane:
    // Verify that updating the document's buffer (is_dirty = true) without calling save
    // correctly reflects the latest buffer in the preview.
    use katana_core::document::Document;

    let mut doc = Document::new("/workspace/spec.md", "# Original");
    let mut pane = PreviewPane::default();

    // Initial preview
    pane.update_markdown_sections(&doc.buffer, std::path::Path::new("/tmp/test.md"));
    let texts = markdown_texts(&pane.sections);
    assert!(texts[0].contains("# Original"));

    // Edit document (unsaved state)
    doc.update_buffer("# Modified by user\n\nThis is not saved yet.");
    assert!(doc.is_dirty, "Document must be dirty");

    // Update preview with unsaved buffer
    pane.update_markdown_sections(&doc.buffer, std::path::Path::new("/tmp/test.md"));
    let texts = markdown_texts(&pane.sections);
    assert!(
        texts[0].contains("Modified by user"),
        "Unsaved edits are not reflected in preview"
    );
    assert!(
        texts[0].contains("This is not saved yet"),
        "Unsaved edits are not reflected in preview"
    );

    // Document should still be dirty (not saved)
    assert!(doc.is_dirty, "Document should not have been saved");
}

// ── extract_svg tests ──

#[test]
fn valid_svg_is_extracted() {
    let html = r#"<div><svg width="100" height="100"><rect/></svg></div>"#;
    let svg = extract_svg(html).unwrap();
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
}

#[test]
fn returns_none_if_no_svg_is_present() {
    assert!(extract_svg("<div>hello</div>").is_none());
    assert!(extract_svg("").is_none());
}

#[test]
fn covers_from_start_to_end_if_multiple_svgs_are_present() {
    let html = r#"<svg>first</svg><p>mid</p><svg>second</svg>"#;
    let svg = extract_svg(html).unwrap();
    // Includes up to the last closing tag using rfind("</svg>")
    assert!(svg.contains("first"));
    assert!(svg.contains("second"));
}

// ── decode_png_rgba tests ──

#[test]
fn valid_png_is_decoded() {
    // Generate minimal byte sequence for a 1x1 white PNG
    let mut buf = Vec::new();
    {
        use image::{ImageBuffer, Rgba};
        let img = ImageBuffer::from_pixel(1, 1, Rgba([255u8, 255, 255, 255]));
        let mut cursor = std::io::Cursor::new(&mut buf);
        img.write_to(&mut cursor, image::ImageFormat::Png).unwrap();
    }
    let result = decode_png_rgba(&buf);
    assert!(result.is_ok());
    let rasterized = result.unwrap();
    assert_eq!(rasterized.width, 1);
    assert_eq!(rasterized.height, 1);
    assert_eq!(rasterized.rgba.len(), 4); // 1x1 RGBA = 4 bytes
}

#[test]
fn invalid_data_returns_error() {
    let result = decode_png_rgba(b"not a png");
    assert!(result.is_err());
}

// ── Additional update_markdown_sections tests ──

#[test]
fn markdown_only_input_is_sectioned_correctly() {
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(
        "# Title\n\nParagraph 1\n\n## Subtitle\n\nParagraph 2",
        std::path::Path::new("/tmp/test.md"),
    );
    assert_eq!(pane.sections.len(), 1);
    assert!(matches!(pane.sections[0], RenderedSection::Markdown(_)));
}

#[test]
fn mixed_diagram_input_is_split_into_sections() {
    let mut pane = PreviewPane::default();
    let src =
        "Before\n```mermaid\ngraph TD; A-->B\n```\nMiddle\n```drawio\n<mxGraphModel/>\n```\nAfter";
    pane.update_markdown_sections(src, std::path::Path::new("/tmp/test.md"));
    // Markdown + Pending + Markdown + Pending + Markdown = 5 sections
    assert!(pane.sections.len() >= 3);
}

#[test]
fn empty_input_returns_empty_section_list() {
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections("", std::path::Path::new("/tmp/test.md"));
    assert!(pane.sections.is_empty());
}

#[test]
fn centered_markdown_is_processed_in_update_markdown_sections() {
    let mut pane = PreviewPane::default();
    let src = "<p align=\"center\">centered</p>";
    pane.update_markdown_sections(src, std::path::Path::new("/tmp/test.md"));
    assert_eq!(pane.sections.len(), 1);
    assert!(matches!(pane.sections[0], RenderedSection::CenteredMarkdown(_)));
}

#[test]
fn centered_markdown_is_processed_in_full_render() {
    let mut pane = PreviewPane::default();
    let src = "<p align=\"center\">centered</p>";
    pane.full_render(src, std::path::Path::new("/tmp/test.md"));
    assert_eq!(pane.sections.len(), 1);
    assert!(matches!(pane.sections[0], RenderedSection::CenteredMarkdown(_)));
}

// ── Cover each variant of show_section using egui_kittest ──

/// Covers rendering of the Markdown variant in show_section
#[test]
fn show_section_markdown_variant_renders() {
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(
        "# Hello from egui test",
        std::path::Path::new("/tmp/test.md"),
    );

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui);
    });
    harness.run();
    // OK if it doesn't crash
}

/// Covers rendering of the Error variant in show_section (L267-275)
#[test]
fn show_section_error_variant_renders() {
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::Error {
        kind: "DrawIo".to_string(),
        _source: "<mxCell/>".to_string(),
        message: "invalid XML".to_string(),
    }];

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui);
    });
    harness.run();
}

/// Covers rendering of the CommandNotFound variant in show_section (L277-291)
#[test]
fn show_section_command_not_found_variant_renders() {
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::CommandNotFound {
        tool_name: "mmdc".to_string(),
        install_hint: "npm install -g @mermaid-js/mermaid-cli".to_string(),
        _source: "graph TD; A-->B".to_string(),
    }];

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui);
    });
    harness.run();
}

/// Covers rendering of the NotInstalled variant in show_section (L292-296, L310-341)
#[test]
fn show_section_not_installed_variant_renders() {
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::NotInstalled {
        kind: "PlantUML".to_string(),
        download_url: "https://example.com/plantuml.jar".to_string(),
        install_path: PathBuf::from("/tmp/plantuml.jar"),
    }];

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui);
    });
    harness.run();
}

/// Covers rendering of the Pending variant in show_section (L297-305)
#[test]
fn show_section_pending_variant_renders() {
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::Pending {
        kind: "Mermaid".to_string(),
    }];

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui);
    });
    // Spinner paints constantly so execute just 1 frame with step()
    harness.step();
}

/// Covers rendering of the Image variant in show_section (L258-261, L344-358)
#[test]
fn show_section_image_variant_renders() {
    let mut pane = PreviewPane::default();
    // 1x1 RGBA dummy image
    pane.sections = vec![RenderedSection::Image {
        svg_data: RasterizedSvg {
            width: 1,
            height: 1,
            rgba: vec![255, 255, 255, 255],
        },
        alt: "test diagram".to_string(),
    }];

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui);
    });
    harness.run();
}

/// Covers show() method rendering (L156-167): has allow(dead_code) but covered by egui_kittest
#[test]
fn show_method_renders_without_crash() {
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections("# Show method test", std::path::Path::new("/tmp/test.md"));

    let mut harness = Harness::new_ui(move |ui| {
        pane.show(ui);
    });
    harness.run();
}

/// Covers the empty section branch in render_sections (L189-191)
#[test]
fn render_sections_empty_shows_no_preview_label() {
    let mut pane = PreviewPane::default();
    // sections remains empty

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui);
    });
    harness.run();
}

/// poll_renders: repaint_after is called when still pending (L214-215)
#[test]
fn poll_renders_with_pending_does_not_crash() {
    let mut pane = PreviewPane::default();
    // background thread started with full_render
    pane.full_render(
        "# Title\n```mermaid\ngraph TD; A-->B\n```\nAfter",
        std::path::Path::new("/tmp/test.md"),
    );

    // execute poll_renders with egui context
    let mut harness = Harness::new_ui(move |ui| {
        // show_content calls poll_renders internally
        pane.show_content(ui);
    });
    // Repaints continue via spinner due to Pending section.
    // Verify no crash occurs by executing just 1 frame using step().
    harness.step();
}

/// Renders show_not_installed UI for NotInstalled state (L310-341)
#[test]
fn show_section_not_installed_download_button_returns_request() {
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::NotInstalled {
        kind: "PlantUML".to_string(),
        download_url: "https://example.com/plantuml.jar".to_string(),
        install_path: PathBuf::from("/tmp/plantuml_test.jar"),
    }];

    let mut harness = Harness::new_ui(move |ui| {
        // Covers show_not_installed rendering (L316-341)
        let _req = pane.show_content(ui);
    });
    harness.run();
    // OK if no crash occurs (button rendering and label rendering executed)
}

/// show_rasterized: Covers the actual texture drawing path (L344-358)
#[test]
fn show_section_image_full_render_with_texture() {
    let mut pane = PreviewPane::default();
    // 4x4 valid RGBA image
    let rgba: Vec<u8> = (0..64).map(|i| (i * 4) as u8).collect();
    pane.sections = vec![RenderedSection::Image {
        svg_data: RasterizedSvg {
            width: 4,
            height: 4,
            rgba,
        },
        alt: "Test texture".to_string(),
    }];

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui);
    });
    harness.run();
}

// ── Image Path Resolution Integration Tests ──

#[test]
fn image_path_resolved_in_rendered_markdown_section() {
    let dir = tempfile::tempdir().unwrap();
    // Create: project/docs/readme.md referencing project/assets/logo.png
    let docs_dir = dir.path().join("docs");
    let assets_dir = dir.path().join("assets");
    std::fs::create_dir_all(&docs_dir).unwrap();
    std::fs::create_dir_all(&assets_dir).unwrap();
    std::fs::write(assets_dir.join("logo.png"), b"fake-png").unwrap();

    let md_path = docs_dir.join("readme.md");
    let source = "# Title\n\n![logo](../assets/logo.png)\n\nAfter image.";

    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(source, &md_path);

    let texts = markdown_texts(&pane.sections);
    assert_eq!(texts.len(), 1);
    // The relative path should be resolved to an absolute file:// URI.
    assert!(
        texts[0].contains("file://"),
        "Image path should be resolved to file:// URI, got: {}",
        texts[0]
    );
    assert!(
        !texts[0].contains("../"),
        "Relative path '..' should be resolved, got: {}",
        texts[0]
    );
}

#[test]
fn http_image_url_preserved_in_rendered_markdown_section() {
    let source = "# Title\n\n![logo](https://example.com/logo.png)\n";
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(source, std::path::Path::new("/tmp/test.md"));

    let texts = markdown_texts(&pane.sections);
    assert!(
        texts[0].contains("https://example.com/logo.png"),
        "HTTP URL should be preserved unchanged, got: {}",
        texts[0]
    );
}

#[test]
fn multiple_images_all_resolved_in_single_section() {
    let dir = tempfile::tempdir().unwrap();
    let img_dir = dir.path().join("img");
    std::fs::create_dir_all(&img_dir).unwrap();
    std::fs::write(img_dir.join("a.png"), b"png-a").unwrap();
    std::fs::write(img_dir.join("b.png"), b"png-b").unwrap();

    let md_path = dir.path().join("doc.md");
    let source = "![A](img/a.png)\n\n![B](img/b.png)\n";

    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(source, &md_path);

    let texts = markdown_texts(&pane.sections);
    // Both images should be resolved.
    let resolved_count = texts[0].matches("file://").count();
    assert_eq!(
        resolved_count, 2,
        "Both images should be resolved, found {resolved_count} file:// URIs"
    );
}

// ── Preset Color Application via egui_kittest ──

#[test]
fn preset_colors_applied_without_crash_in_harness() {
    // Verifies that the full preset wiring (preview_text color + syntax themes)
    // works through the real rendering pipeline without panicking.
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(
        "# Heading\n\nBody text.\n\n```rust\nfn main() {}\n```\n",
        std::path::Path::new("/tmp/test.md"),
    );

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui);
    });
    harness.run();
}

#[test]
fn syntax_highlighted_code_block_renders_in_harness() {
    // Specifically tests that the syntax theme names from the preset are
    // valid syntect theme identifiers that don't cause panics.
    let mut pane = PreviewPane::default();
    let source = concat!(
        "# Code\n\n",
        "```python\n",
        "def hello():\n",
        "    print('world')\n",
        "```\n\n",
        "```json\n",
        "{\"key\": \"value\"}\n",
        "```\n",
    );
    pane.update_markdown_sections(source, std::path::Path::new("/tmp/test.md"));

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui);
    });
    harness.run();
}

#[test]
fn show_section_centered_markdown_variant_renders() {
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::CenteredMarkdown(
        "# Centered Title\n![alt](test.png)".to_string(),
    )];

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui);
    });
    harness.run();
}
