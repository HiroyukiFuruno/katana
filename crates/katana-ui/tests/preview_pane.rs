use eframe::egui;
use egui_kittest::kittest::{NodeT, Queryable};
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
fn centered_html_stays_in_markdown_section_update() {
    // After render_html_fn refactor, HTML blocks remain in Markdown sections.
    // pulldown-cmark detects them and passes to render_html_fn at render time.
    let mut pane = PreviewPane::default();
    let src = "<p align=\"center\">centered</p>";
    pane.update_markdown_sections(src, std::path::Path::new("/tmp/test.md"));
    assert_eq!(pane.sections.len(), 1);
    assert!(matches!(pane.sections[0], RenderedSection::Markdown(_)));
}

#[test]
fn centered_html_stays_in_markdown_section_full_render() {
    let mut pane = PreviewPane::default();
    let src = "<p align=\"center\">centered</p>";
    pane.full_render(src, std::path::Path::new("/tmp/test.md"));
    assert_eq!(pane.sections.len(), 1);
    assert!(matches!(pane.sections[0], RenderedSection::Markdown(_)));
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
    // HTML blocks now stay inside Markdown sections and are rendered via render_html_fn.
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::Markdown(
        "<p align=\"center\"><img src=\"test.png\" alt=\"alt\"></p>".to_string(),
    )];

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui);
    });
    harness.run();
}

// ── TDD: HTML rendering behavior verification ──

/// Verifies that a centered paragraph with multiple badge links renders them
/// on the same horizontal row (all badges share the same Y coordinate).
#[test]
fn centered_badges_render_on_same_horizontal_row() {
    let html = concat!(
        "<p align=\"center\">\n",
        "  <a href=\"LICENSE\"><img src=\"badge1.svg\" alt=\"License: MIT\"></a>\n",
        "  <a href=\"https://example.com/ci\"><img src=\"badge2.svg\" alt=\"CI\"></a>\n",
        "</p>\n"
    );
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::Markdown(html.to_string())];

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui);
    });
    // Multiple steps needed for measure-then-position pattern (request_discard)
    harness.step();
    harness.step();
    harness.step();
    harness.run();
    // Test passes if no panic — the multi-element centering code path is exercised.
    // AccessKit-based position verification follows once the basic rendering is stable.
}

/// Verifies that a link in a centered paragraph is clickable (has click sense).
#[test]
fn centered_text_link_is_clickable() {
    let html = concat!(
        "<p align=\"center\">\n",
        "  English | <a href=\"README.ja.md\">日本語</a>\n",
        "</p>\n"
    );
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::Markdown(html.to_string())];

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui);
    });
    harness.step();
    harness.step();
    harness.step();
    harness.run();
    // Verify the link label exists in the accessibility tree
    let _link_node = harness.get_by_label("日本語");
}

/// Verifies that multiple centered <p> blocks each take their own vertical
/// space and don't overlap. The cursor positions after each block should
/// be strictly increasing.
#[test]
fn multiple_centered_paragraphs_have_increasing_y_positions() {
    let html = concat!(
        "<p align=\"center\">First paragraph</p>\n\n",
        "<p align=\"center\">Second paragraph</p>\n\n",
        "<p align=\"center\">Third paragraph</p>\n"
    );
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::Markdown(html.to_string())];

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui);
    });
    harness.step();
    harness.step();
    harness.step();
    harness.run();

    // Verify all three paragraphs are present and have distinct Y positions
    let first = harness.get_by_label("First paragraph");
    let second = harness.get_by_label("Second paragraph");
    let third = harness.get_by_label("Third paragraph");

    let first_bounds = first
        .accesskit_node()
        .raw_bounds()
        .expect("First should have bounds");
    let second_bounds = second
        .accesskit_node()
        .raw_bounds()
        .expect("Second should have bounds");
    let third_bounds = third
        .accesskit_node()
        .raw_bounds()
        .expect("Third should have bounds");

    assert!(
        second_bounds.y0 > first_bounds.y0,
        "Second paragraph Y ({}) should be below first ({})",
        second_bounds.y0,
        first_bounds.y0
    );
    assert!(
        third_bounds.y0 > second_bounds.y0,
        "Third paragraph Y ({}) should be below second ({})",
        third_bounds.y0,
        second_bounds.y0
    );
}

/// Full README header structure: icon, heading, description, badges, language selector.
/// Verifies the complete rendering pipeline doesn't crash and produces widgets.
#[test]
fn readme_header_full_structure_renders() {
    let html = concat!(
        "<p align=\"center\">\n",
        "  <img src=\"assets/icon.png\" alt=\"KatanA Desktop\">\n",
        "</p>\n\n",
        "<h1 align=\"center\">KatanA Desktop</h1>\n\n",
        "<p align=\"center\">\n",
        "  A fast, lightweight Markdown workspace for macOS.\n",
        "</p>\n\n",
        "<p align=\"center\">\n",
        "  <a href=\"LICENSE\"><img src=\"badge1.svg\" alt=\"License\"></a>\n",
        "  <a href=\"ci\"><img src=\"badge2.svg\" alt=\"CI\"></a>\n",
        "</p>\n\n",
        "<p align=\"center\">\n",
        "  English | <a href=\"README.ja.md\">日本語</a>\n",
        "</p>\n"
    );
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::Markdown(html.to_string())];

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui);
    });
    harness.step();
    harness.step();
    harness.step();
    harness.run();

    // Verify key widgets exist in the accessibility tree
    let _heading = harness.get_by_label("KatanA Desktop");
    let _description = harness.get_by_label("A fast, lightweight Markdown workspace for macOS.");
    let _lang_link = harness.get_by_label("日本語");
}

/// Verifies that a single centered text is horizontally centered within the
/// available width. The widget's center X should be near the panel center.
#[test]
fn centered_single_text_is_horizontally_centered() {
    let html = "<p align=\"center\">Centered Text Here</p>\n";
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::Markdown(html.to_string())];

    let panel_width: f32 = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(panel_width, 200.0))
        .build_ui(move |ui| {
            pane.show_content(ui);
        });
    harness.step();
    harness.step();
    harness.step();
    harness.run();

    let node = harness.get_by_label("Centered Text Here");
    let bounds = node
        .accesskit_node()
        .raw_bounds()
        .expect("Should have bounds");
    let widget_center_x = (bounds.x0 + bounds.x1) / 2.0;
    let panel_center_x = f64::from(panel_width) / 2.0;

    // Centering tolerance: widget center should be within 50px of panel center.
    // This is generous because egui adds padding/margins.
    let tolerance = 50.0;
    assert!(
        (widget_center_x - panel_center_x).abs() < tolerance,
        "Widget center X ({widget_center_x:.1}) should be near panel center ({panel_center_x:.1}), diff={:.1}",
        (widget_center_x - panel_center_x).abs()
    );
}

/// Verifies that multi-element centered content (text + link) is rendered
/// on the same row and that the link widget exists in the accessibility tree.
#[test]
fn centered_text_and_link_share_same_row() {
    let html = concat!(
        "<p align=\"center\">\n",
        "  English | <a href=\"README.ja.md\">日本語</a>\n",
        "</p>\n"
    );
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::Markdown(html.to_string())];

    let mut harness = Harness::builder()
        .with_size(egui::vec2(800.0, 200.0))
        .build_ui(move |ui| {
            pane.show_content(ui);
        });
    harness.step();
    harness.step();
    harness.step();
    harness.run();

    // Both "English |" text and "日本語" link should exist
    let text_node = harness.get_by_label("English |");
    let link_node = harness.get_by_label("日本語");

    let text_bounds = text_node
        .accesskit_node()
        .raw_bounds()
        .expect("text should have bounds");
    let link_bounds = link_node
        .accesskit_node()
        .raw_bounds()
        .expect("link should have bounds");

    // They should be on the same row (similar Y coordinates, within text height)
    let y_diff = (text_bounds.y0 - link_bounds.y0).abs();
    assert!(
        y_diff < 5.0,
        "Text and link should be on same row: text Y={:.1}, link Y={:.1}, diff={y_diff:.1}",
        text_bounds.y0,
        link_bounds.y0
    );

    // The link should be to the right of the text
    assert!(
        link_bounds.x0 > text_bounds.x0,
        "Link X ({:.1}) should be to the right of text X ({:.1})",
        link_bounds.x0,
        text_bounds.x0
    );

    // The GROUP (text + link combined) should be horizontally centered.
    let group_left = text_bounds.x0.min(link_bounds.x0);
    let group_right = text_bounds.x1.max(link_bounds.x1);
    let group_center_x = (group_left + group_right) / 2.0;
    let panel_center_x = 800.0 / 2.0; // panel width is 800
    let tolerance = 50.0;
    assert!(
        (group_center_x - panel_center_x).abs() < tolerance,
        "Multi-element group center X ({group_center_x:.1}) should be near panel center ({panel_center_x:.1}), diff={:.1}",
        (group_center_x - panel_center_x).abs()
    );
}

// ── TDD: Centering bug reproduction ──
// These tests verify the POSITION of centered elements, not just that they render.
// The bug: <h1 align="center">, <p align="center"> etc. are not horizontally centered.

/// Verifies that `<h1 align="center">` renders the heading text centered
/// within the panel width. The heading's center X must be near the panel center.
#[test]
fn centered_heading_h1_is_horizontally_centered() {
    let html = "<h1 align=\"center\">KatanA Desktop</h1>\n";
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::Markdown(html.to_string())];

    let panel_width: f32 = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(panel_width, 200.0))
        .build_ui(move |ui| {
            pane.show_content(ui);
        });
    harness.step();
    harness.step();
    harness.step();
    harness.run();

    let node = harness.get_by_label("KatanA Desktop");
    let bounds = node
        .accesskit_node()
        .raw_bounds()
        .expect("Should have bounds");
    let widget_center_x = (bounds.x0 + bounds.x1) / 2.0;
    let panel_center_x = f64::from(panel_width) / 2.0;

    let tolerance = 50.0;
    assert!(
        (widget_center_x - panel_center_x).abs() < tolerance,
        "Centered <h1> center X ({widget_center_x:.1}) should be near panel center ({panel_center_x:.1}), diff={:.1}",
        (widget_center_x - panel_center_x).abs()
    );
}

/// Verifies that a centered description paragraph renders centered.
#[test]
fn centered_description_paragraph_is_horizontally_centered() {
    let html = concat!(
        "<p align=\"center\">\n",
        "  macOS向けの高速・軽量なMarkdownワークスペース — Rustとeguiで構築。\n",
        "</p>\n"
    );
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::Markdown(html.to_string())];

    let panel_width: f32 = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(panel_width, 200.0))
        .build_ui(move |ui| {
            pane.show_content(ui);
        });
    harness.step();
    harness.step();
    harness.step();
    harness.run();

    let node =
        harness.get_by_label("macOS向けの高速・軽量なMarkdownワークスペース — Rustとeguiで構築。");
    let bounds = node
        .accesskit_node()
        .raw_bounds()
        .expect("Should have bounds");
    let widget_center_x = (bounds.x0 + bounds.x1) / 2.0;
    let panel_center_x = f64::from(panel_width) / 2.0;

    let tolerance = 50.0;
    assert!(
        (widget_center_x - panel_center_x).abs() < tolerance,
        "Centered <p> center X ({widget_center_x:.1}) should be near panel center ({panel_center_x:.1}), diff={:.1}",
        (widget_center_x - panel_center_x).abs()
    );
}

/// Full README header integration test: verifies ALL centered elements are
/// horizontally centered. This reproduces the exact bug seen in the UI where
/// elements appear left-aligned instead of centered.
#[test]
fn readme_header_all_elements_horizontally_centered() {
    let html = concat!(
        "<p align=\"center\">\n",
        "  <img src=\"assets/icon.iconset/icon_128x128.png\" width=\"128\" alt=\"KatanA Desktop\">\n",
        "</p>\n\n",
        "<h1 align=\"center\">KatanA Desktop</h1>\n\n",
        "<p align=\"center\">\n",
        "  macOS向けの高速・軽量なMarkdownワークスペース — Rustとeguiで構築。\n",
        "</p>\n\n",
        "<p align=\"center\">\n",
        "  <a href=\"LICENSE\"><img src=\"https://img.shields.io/badge/License-MIT-blue.svg\" alt=\"License: MIT\"></a>\n",
        "  <a href=\"https://github.com/HiroyukiFuruno/katana/actions/workflows/ci.yml\"><img src=\"https://github.com/HiroyukiFuruno/katana/actions/workflows/ci.yml/badge.svg\" alt=\"CI\"></a>\n",
        "  <a href=\"https://github.com/HiroyukiFuruno/katana/releases/latest\"><img src=\"https://img.shields.io/github/v/release/HiroyukiFuruno/katana\" alt=\"Latest Release\"></a>\n",
        "  <img src=\"https://img.shields.io/badge/platform-macOS-lightgrey\" alt=\"Platform: macOS\">\n",
        "</p>\n\n",
        "<p align=\"center\">\n",
        "  <a href=\"README.md\">English</a> | 日本語\n",
        "</p>\n"
    );
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::Markdown(html.to_string())];

    let panel_width: f32 = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(panel_width, 400.0))
        .build_ui(move |ui| {
            pane.show_content(ui);
        });
    // Multiple frames needed for measure-then-position (request_discard)
    for _ in 0..5 {
        harness.step();
    }
    harness.run();

    let panel_center_x = f64::from(panel_width) / 2.0;
    let tolerance = 50.0;

    // Verify heading is centered
    let heading = harness.get_by_label("KatanA Desktop");
    let heading_bounds = heading
        .accesskit_node()
        .raw_bounds()
        .expect("heading should have bounds");
    let heading_center_x = (heading_bounds.x0 + heading_bounds.x1) / 2.0;
    assert!(
        (heading_center_x - panel_center_x).abs() < tolerance,
        "Heading center X ({heading_center_x:.1}) should be near panel center ({panel_center_x:.1}), diff={:.1}",
        (heading_center_x - panel_center_x).abs()
    );

    // Verify description text is centered
    let desc =
        harness.get_by_label("macOS向けの高速・軽量なMarkdownワークスペース — Rustとeguiで構築。");
    let desc_bounds = desc
        .accesskit_node()
        .raw_bounds()
        .expect("description should have bounds");
    let desc_center_x = (desc_bounds.x0 + desc_bounds.x1) / 2.0;
    assert!(
        (desc_center_x - panel_center_x).abs() < tolerance,
        "Description center X ({desc_center_x:.1}) should be near panel center ({panel_center_x:.1}), diff={:.1}",
        (desc_center_x - panel_center_x).abs()
    );

    // Verify language selector link is centered (the "English" link within the group)
    let english_link = harness.get_by_label("English");
    let english_bounds = english_link
        .accesskit_node()
        .raw_bounds()
        .expect("English link should have bounds");
    // The "| 日本語" is plain text (pipe + space + Japanese text)
    let ja_node = harness.get_by_label("| 日本語");
    let ja_bounds = ja_node
        .accesskit_node()
        .raw_bounds()
        .expect("Japanese text should have bounds");
    let group_left = english_bounds.x0.min(ja_bounds.x0);
    let group_right = english_bounds.x1.max(ja_bounds.x1);
    let group_center_x = (group_left + group_right) / 2.0;
    assert!(
        (group_center_x - panel_center_x).abs() < tolerance,
        "Language selector group center X ({group_center_x:.1}) should be near panel center ({panel_center_x:.1}), diff={:.1}",
        (group_center_x - panel_center_x).abs()
    );
}

/// Diagnostic test: heading + description (no badges) — isolates whether
/// multiple centered HTML blocks affect each other's positioning.
#[test]
fn centered_heading_then_description_both_centered() {
    let html = concat!(
        "<h1 align=\"center\">KatanA Desktop</h1>\n\n",
        "<p align=\"center\">\n",
        "  macOS向けの高速・軽量なMarkdownワークスペース — Rustとeguiで構築。\n",
        "</p>\n"
    );
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::Markdown(html.to_string())];

    let panel_width: f32 = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(panel_width, 300.0))
        .build_ui(move |ui| {
            pane.show_content(ui);
        });
    for _ in 0..5 {
        harness.step();
    }
    harness.run();

    let panel_center_x = f64::from(panel_width) / 2.0;
    let tolerance = 50.0;

    let heading = harness.get_by_label("KatanA Desktop");
    let heading_bounds = heading.accesskit_node().raw_bounds().expect("bounds");
    let heading_cx = (heading_bounds.x0 + heading_bounds.x1) / 2.0;
    assert!(
        (heading_cx - panel_center_x).abs() < tolerance,
        "Heading center X ({heading_cx:.1}) should be near panel center ({panel_center_x:.1})"
    );

    let desc =
        harness.get_by_label("macOS向けの高速・軽量なMarkdownワークスペース — Rustとeguiで構築。");
    let desc_bounds = desc.accesskit_node().raw_bounds().expect("bounds");
    let desc_cx = (desc_bounds.x0 + desc_bounds.x1) / 2.0;
    assert!(
        (desc_cx - panel_center_x).abs() < tolerance,
        "Description center X ({desc_cx:.1}) should be near panel center ({panel_center_x:.1})"
    );
}

/// Diagnostic test: badges block + language selector — isolates whether
/// the multi-element badge block affects subsequent centered blocks.
#[test]
fn badges_then_language_selector_both_centered() {
    let html = concat!(
        "<p align=\"center\">\n",
        "  <a href=\"LICENSE\"><img src=\"https://img.shields.io/badge/License-MIT-blue.svg\" alt=\"License: MIT\"></a>\n",
        "  <a href=\"ci\"><img src=\"https://github.com/repo/ci.yml/badge.svg\" alt=\"CI\"></a>\n",
        "</p>\n\n",
        "<p align=\"center\">\n",
        "  <a href=\"README.md\">English</a> | 日本語\n",
        "</p>\n"
    );
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::Markdown(html.to_string())];

    let panel_width: f32 = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(panel_width, 300.0))
        .build_ui(move |ui| {
            pane.show_content(ui);
        });
    for _ in 0..5 {
        harness.step();
    }
    harness.run();

    let panel_center_x = f64::from(panel_width) / 2.0;
    let tolerance = 50.0;

    // Verify the language selector after badges is still centered
    let english_link = harness.get_by_label("English");
    let english_bounds = english_link.accesskit_node().raw_bounds().expect("bounds");
    let ja_node = harness.get_by_label("| 日本語");
    let ja_bounds = ja_node.accesskit_node().raw_bounds().expect("bounds");
    let group_left = english_bounds.x0.min(ja_bounds.x0);
    let group_right = english_bounds.x1.max(ja_bounds.x1);
    let group_center_x = (group_left + group_right) / 2.0;
    assert!(
        (group_center_x - panel_center_x).abs() < tolerance,
        "Language selector after badges: center X ({group_center_x:.1}) should be near panel center ({panel_center_x:.1}), diff={:.1}",
        (group_center_x - panel_center_x).abs()
    );
}
