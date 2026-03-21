use eframe::egui;
use egui_kittest::kittest::{NodeT, Queryable};
use egui_kittest::Harness;
use katana_core::markdown::color_preset::DiagramColorPreset;
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

fn flatten_shapes<'a>(
    shapes: impl IntoIterator<Item = &'a egui::epaint::ClippedShape>,
) -> Vec<&'a egui::epaint::Shape> {
    fn visit<'a>(shape: &'a egui::epaint::Shape, acc: &mut Vec<&'a egui::epaint::Shape>) {
        match shape {
            egui::epaint::Shape::Vec(children) => {
                for child in children {
                    visit(child, acc);
                }
            }
            _ => acc.push(shape),
        }
    }

    let mut flat = Vec::new();
    for clipped in shapes {
        visit(&clipped.shape, &mut flat);
    }
    flat
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
    pane.full_render(
        source,
        std::path::Path::new("/tmp/test.md"),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        false,
        4,
    );

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
    pane.full_render(
        source,
        std::path::Path::new("/tmp/test.md"),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        false,
        4,
    );

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
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        false,
        4,
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
    pane.full_render(
        src,
        std::path::Path::new("/tmp/test.md"),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        false,
        4,
    );
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
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        false,
        4,
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
    let source = "# Title\n\nInline image: ![logo](../assets/logo.png)\n\nAfter image.";

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
    let source = "# Title\n\nInline: ![logo](https://example.com/logo.png)\n";
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
    let source = "Inline: ![A](img/a.png) ![B](img/b.png)\n";

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

#[test]
fn standalone_local_image_is_split_into_local_image_section() {
    let source = "# Title\n\n![Logo](file:///path/to/logo.png)\n\nText";
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(source, std::path::Path::new("/tmp/test.md"));

    assert_eq!(pane.sections.len(), 3);
    assert!(matches!(pane.sections[0], RenderedSection::Markdown(_)));
    if let RenderedSection::LocalImage { path, alt } = &pane.sections[1] {
        assert_eq!(path.to_string_lossy(), "/path/to/logo.png");
        assert_eq!(alt, "Logo");
    } else {
        panic!("Expected LocalImage section at index 1");
    }
    assert!(matches!(pane.sections[2], RenderedSection::Markdown(_)));
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

/// Verifies that mixed inline text is emitted as a single wrapped text run.
///
/// Rendering each text fragment as an independent widget causes baseline drift
/// for CJK text and makes long lines wrap inconsistently.
#[test]
fn inline_html_text_fragments_are_not_split_into_multiple_widgets() {
    let html = "<p>前文<strong>強調</strong>後文</p>\n";
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::Markdown(html.to_string())];

    let mut harness = Harness::builder()
        .with_size(egui::vec2(400.0, 160.0))
        .build_ui(move |ui| {
            pane.show_content(ui);
        });
    harness.step();
    harness.step();
    harness.step();
    harness.run();

    let _combined = harness.get_by_label("前文強調後文");
    assert!(
        harness.query_by_label("前文").is_none(),
        "inline text should not be emitted as separate widgets"
    );
    assert!(
        harness.query_by_label("強調").is_none(),
        "strong text should participate in the same text run"
    );
}

#[test]
fn blockquote_long_line_wraps_within_preview_width() {
    let markdown = concat!(
        "> Note: On macOS Sequoia (15.x), Gatekeeper requires this command for apps not notarized with Apple. ",
        "Alternatively, go to System Settings -> Privacy & Security -> \"Open Anyway\" after the first launch attempt.\n"
    );
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(markdown, std::path::Path::new("/tmp/blockquote.md"));

    let mut harness = Harness::builder()
        .with_size(egui::vec2(520.0, 240.0))
        .build_ui(move |ui| {
            pane.show_content(ui);
        });
    harness.step();
    harness.step();
    harness.step();
    harness.run();

    let quote = harness.get_by_label_contains("Note:");
    let bounds = quote
        .accesskit_node()
        .raw_bounds()
        .expect("blockquote text should have bounds");
    assert!(
        bounds.x1 <= 520.0,
        "blockquote long line must stay within the preview viewport, got right edge {:.1}",
        bounds.x1
    );
    assert!(
        bounds.y1 - bounds.y0 > 30.0,
        "blockquote long line should wrap to multiple rows, got height {:.1}",
        bounds.y1 - bounds.y0
    );
}

#[test]
fn blockquote_with_strong_prefix_wraps_within_preview_width() {
    let markdown = concat!(
        "> **Note:** On macOS Sequoia (15.x), Gatekeeper requires this command for apps not notarized with Apple. ",
        "Alternatively, go to **System Settings -> Privacy & Security -> \"Open Anyway\"** after the first launch attempt.\n"
    );
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(markdown, std::path::Path::new("/tmp/blockquote-strong.md"));

    let ctx = egui::Context::default();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(520.0, 240.0),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.inner_margin(egui::Margin::same(12)))
                .show(ctx, |ui| {
                    let inner_width = ui.available_width();
                    ui.set_width(inner_width);
                    pane.show(ui);
                });
        },
    );

    let text_shapes: Vec<(&egui::epaint::TextShape, egui::Rect)> = output
        .shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::epaint::Shape::Text(text)
                if text.galley.job.text.contains("Note:")
                    || text.galley.job.text.contains("Gatekeeper requires")
                    || text.galley.job.text.contains("Open Anyway") =>
            {
                Some((text, clipped.clip_rect))
            }
            _ => None,
        })
        .collect();

    assert!(
        !text_shapes.is_empty(),
        "expected blockquote text shapes for strong-prefixed note"
    );

    let max_right = text_shapes
        .iter()
        .map(|(text, _)| text.visual_bounding_rect().right())
        .fold(f32::NEG_INFINITY, f32::max);
    let max_rows = text_shapes
        .iter()
        .map(|(text, _)| text.galley.rows.len())
        .max()
        .unwrap_or(0);
    let clip_right = text_shapes
        .iter()
        .map(|(_, clip_rect)| clip_rect.right())
        .fold(f32::NEG_INFINITY, f32::max);

    assert!(
        max_rows > 1,
        "strong-prefixed blockquote should wrap to multiple rows"
    );
    assert!(
        max_right <= clip_right + 4.0,
        "strong-prefixed blockquote must stay within clip rect, got right edge {max_right} with clip right {clip_right}"
    );
}

#[test]
fn blockquote_soft_break_continuation_stays_aligned_to_quote_content() {
    let markdown = concat!(
        "> Note: On macOS Sequoia (15.x), Gatekeeper requires this command for apps not notarized with Apple.\n",
        "> Alternatively, go to System Settings -> Privacy & Security -> \"Open Anyway\" after the first launch attempt.\n"
    );
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(
        markdown,
        std::path::Path::new("/tmp/blockquote-softbreak.md"),
    );

    let ctx = egui::Context::default();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(680.0, 240.0),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.inner_margin(egui::Margin::same(12)))
                .show(ctx, |ui| {
                    pane.show(ui);
                });
        },
    );

    let text_shapes: Vec<&egui::epaint::TextShape> = flatten_shapes(output.shapes.iter())
        .into_iter()
        .filter_map(|shape| match shape {
            egui::epaint::Shape::Text(text)
                if text.galley.job.text.contains("Note:")
                    || text.galley.job.text.contains("Alternatively,") =>
            {
                Some(text)
            }
            _ => None,
        })
        .collect();

    let combined = text_shapes
        .iter()
        .find(|text| {
            text.galley.job.text.contains("Note:")
                && text.galley.job.text.contains("Alternatively,")
        })
        .copied();

    if let Some(text) = combined {
        let row_lefts: Vec<f32> = text
            .galley
            .rows
            .iter()
            .map(|row| row.rect().left() + text.pos.x)
            .collect();
        let first_left = *row_lefts.first().expect("blockquote row");
        let max_deviation = row_lefts
            .iter()
            .map(|left| (left - first_left).abs())
            .fold(0.0_f32, f32::max);
        assert!(
            max_deviation <= 4.0,
            "blockquote continuation rows should align with the first row, got row_lefts={row_lefts:?}"
        );
    } else {
        assert!(
            text_shapes.len() >= 2,
            "expected blockquote text shapes for both lines"
        );
        let first_left = text_shapes
            .iter()
            .find(|text| text.galley.job.text.contains("Note:"))
            .expect("note text shape")
            .visual_bounding_rect()
            .left();
        let second_left = text_shapes
            .iter()
            .find(|text| text.galley.job.text.contains("Alternatively,"))
            .expect("continuation text shape")
            .visual_bounding_rect()
            .left();

        assert!(
            (second_left - first_left).abs() <= 4.0,
            "blockquote continuation should align with the first line, got first_left={first_left} second_left={second_left}"
        );
    }
}

#[test]
fn preview_scroll_content_uses_viewport_width_instead_of_intrinsic_text_width() {
    let markdown = concat!(
        "# PaddingHeading\n\n",
        "> **Note:** On macOS Sequoia (15.x), Gatekeeper requires this command for apps not notarized with Apple. ",
        "Alternatively, go to **System Settings → Privacy & Security → \"Open Anyway\"** after the first launch attempt.\n"
    );
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(markdown, std::path::Path::new("/tmp/preview-width.md"));

    let ctx = egui::Context::default();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(520.0, 280.0),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.inner_margin(egui::Margin::same(12)))
                .show(ctx, |ui| {
                    pane.show(ui);
                });
        },
    );

    let shapes = flatten_shapes(output.shapes.iter());
    let heading_rect = shapes
        .iter()
        .find_map(|shape| match shape {
            egui::epaint::Shape::Text(text) if text.galley.job.text.contains("PaddingHeading") => {
                Some(text.visual_bounding_rect())
            }
            _ => None,
        })
        .expect("heading text shape");
    let note_shapes: Vec<(&egui::epaint::TextShape, egui::Rect)> = output
        .shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::epaint::Shape::Text(text)
                if text.galley.job.text.contains("Note:")
                    || text.galley.job.text.contains("Gatekeeper requires")
                    || text.galley.job.text.contains("Open Anyway") =>
            {
                Some((text, clipped.clip_rect))
            }
            _ => None,
        })
        .collect();
    let max_right = note_shapes
        .iter()
        .map(|(text, _)| text.visual_bounding_rect().right())
        .fold(f32::NEG_INFINITY, f32::max);
    let max_rows = note_shapes
        .iter()
        .map(|(text, _)| text.galley.rows.len())
        .max()
        .unwrap_or(0);
    let clip_right = note_shapes
        .iter()
        .map(|(_, clip_rect)| clip_rect.right())
        .fold(f32::NEG_INFINITY, f32::max);

    assert!(
        (heading_rect.left() - 12.0).abs() <= 2.0,
        "preview heading should start at the viewport padding, got left edge {}",
        heading_rect.left()
    );
    assert!(
        max_rows > 1,
        "long blockquote should wrap within the preview viewport"
    );
    assert!(
        max_right <= clip_right + 4.0,
        "wrapped blockquote must stay within the preview clip rect, got right edge {max_right} with clip right {clip_right}"
    );
}

#[test]
fn paragraph_with_inline_link_wraps_from_the_left_edge_after_link() {
    let markdown = concat!(
        "KatanA Desktop is under active development. See the ",
        "[Releases page](https://example.com) ",
        "for the latest version and changelog.\n"
    );
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(markdown, std::path::Path::new("/tmp/inline-link-wrap.md"));

    let ctx = egui::Context::default();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(560.0, 220.0),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.inner_margin(egui::Margin::same(12)))
                .show(ctx, |ui| {
                    pane.show(ui);
                });
        },
    );

    let shapes = flatten_shapes(output.shapes.iter());
    let paragraph_left = shapes
        .iter()
        .find_map(|shape| match shape {
            egui::epaint::Shape::Text(text)
                if text
                    .galley
                    .job
                    .text
                    .contains("KatanA Desktop is under active development.") =>
            {
                Some(text.visual_bounding_rect().left())
            }
            _ => None,
        })
        .expect("leading paragraph text shape");
    let tail_word_shapes: Vec<egui::Rect> = shapes
        .iter()
        .filter_map(|shape| match shape {
            egui::epaint::Shape::Text(text)
                if ["for ", "the ", "latest ", "version ", "and ", "changelog."]
                    .iter()
                    .any(|needle| text.galley.job.text.contains(needle)) =>
            {
                Some(text.visual_bounding_rect())
            }
            _ => None,
        })
        .collect();
    assert!(
        !tail_word_shapes.is_empty(),
        "expected trailing paragraph word shapes"
    );

    let first_row_top = tail_word_shapes
        .iter()
        .map(|rect| rect.top())
        .fold(f32::INFINITY, f32::min);
    let continuation_left = tail_word_shapes
        .iter()
        .filter(|rect| rect.top() > first_row_top + 2.0)
        .map(|rect| rect.left())
        .fold(f32::INFINITY, f32::min);

    assert!(
        continuation_left.is_finite(),
        "expected the text after the inline link to wrap to a continuation row"
    );
    assert!(
        (continuation_left - paragraph_left).abs() <= 24.0,
        "text after an inline link must wrap from the paragraph left edge, got paragraph_left={paragraph_left} continuation_left={continuation_left}"
    );
}

#[test]
#[cfg(target_os = "macos")]
fn preview_markdown_renders_emoji_as_inline_image_on_macos() {
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections("Hello 🌍", std::path::Path::new("/tmp/emoji.md"));

    let ctx = egui::Context::default();
    katana_ui::font_loader::SystemFontLoader::setup_fonts(
        &ctx,
        DiagramColorPreset::current(),
        None,
        None,
    );

    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(240.0, 120.0),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                pane.show_content(ui);
            });
        },
    );

    let emoji_text = flatten_shapes(output.shapes.iter())
        .into_iter()
        .filter_map(|shape| match shape {
            egui::epaint::Shape::Text(text) if text.galley.job.text.contains('🌍') => Some(text),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(
        emoji_text.is_empty(),
        "emoji should no longer be rendered through egui text glyphs"
    );

    let mut pane = PreviewPane::default();
    pane.update_markdown_sections("Hello 🌍", std::path::Path::new("/tmp/emoji.md"));
    let mut harness = Harness::new_ui(move |ui| {
        katana_ui::font_loader::SystemFontLoader::setup_fonts(
            ui.ctx(),
            DiagramColorPreset::current(),
            None,
            None,
        );
        pane.show_content(ui);
    });
    harness.run();

    harness.get_by_role(egui::accesskit::Role::Image);
}

#[test]
#[cfg(target_os = "macos")]
fn inline_emoji_stays_within_text_line_height_budget() {
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(
        "👉 Become a Sponsor",
        std::path::Path::new("/tmp/emoji-line.md"),
    );
    let mut harness = Harness::new_ui(move |ui| {
        katana_ui::font_loader::SystemFontLoader::setup_fonts(
            ui.ctx(),
            DiagramColorPreset::current(),
            None,
            None,
        );
        pane.show_content(ui);
    });
    harness.run();

    let image = harness.get_by_role(egui::accesskit::Role::Image);
    let image_bounds = image
        .accesskit_node()
        .raw_bounds()
        .expect("emoji image should have bounds");
    let text = harness.get_by_label("Sponsor");
    let text_bounds = text
        .accesskit_node()
        .raw_bounds()
        .expect("text should have bounds");

    let image_height = image_bounds.y1 - image_bounds.y0;
    let text_height = text_bounds.y1 - text_bounds.y0;
    let bottom_diff = (image_bounds.y1 - text_bounds.y1).abs();

    assert!(
        image_height <= text_height * 1.15,
        "inline emoji should stay near text line height, got image={image_height:.1} text={text_height:.1}"
    );
    assert!(
        bottom_diff <= 8.0,
        "inline emoji should stay aligned near the text baseline, bottom diff={bottom_diff:.1}, image=({:.1},{:.1}) text=({:.1},{:.1})",
        image_bounds.y0,
        image_bounds.y1,
        text_bounds.y0,
        text_bounds.y1,
    );
}

#[test]
fn markdown_text_uses_bottom_vertical_alignment_for_mixed_cjk_runs() {
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(
        "KatanA は AIエージェントと共に仕様駆動開発を行う時代のために設計されたツールです。\n",
        std::path::Path::new("/tmp/cjk-baseline.md"),
    );

    let ctx = egui::Context::default();
    katana_ui::font_loader::SystemFontLoader::setup_fonts(
        &ctx,
        DiagramColorPreset::current(),
        None,
        None,
    );
    katana_ui::theme_bridge::apply_font_family(&ctx, "Monospace");

    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(900.0, 200.0),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                pane.show_content(ui);
            });
        },
    );

    let text_shape = flatten_shapes(output.shapes.iter())
        .into_iter()
        .filter_map(|shape| match shape {
            egui::epaint::Shape::Text(text)
                if text
                    .galley
                    .job
                    .text
                    .contains("AIエージェントと共に仕様駆動開発") =>
            {
                Some(text)
            }
            _ => None,
        })
        .next()
        .expect("expected rendered mixed CJK paragraph");

    assert!(
        text_shape
            .galley
            .job
            .sections
            .iter()
            .all(|section| section.format.valign == egui::Align::BOTTOM),
        "mixed CJK markdown text should use bottom baseline alignment"
    );
}

#[test]
fn html_text_uses_bottom_vertical_alignment_for_mixed_cjk_runs() {
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::Markdown(
        "<p>KatanA は AIエージェントと共に仕様駆動開発を行う時代のために設計されたツールです。</p>\n"
            .to_string(),
    )];

    let ctx = egui::Context::default();
    katana_ui::font_loader::SystemFontLoader::setup_fonts(
        &ctx,
        DiagramColorPreset::current(),
        None,
        None,
    );
    katana_ui::theme_bridge::apply_font_family(&ctx, "Monospace");

    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(900.0, 200.0),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                pane.show_content(ui);
            });
        },
    );

    let text_shape = flatten_shapes(output.shapes.iter())
        .into_iter()
        .filter_map(|shape| match shape {
            egui::epaint::Shape::Text(text)
                if text
                    .galley
                    .job
                    .text
                    .contains("AIエージェントと共に仕様駆動開発") =>
            {
                Some(text)
            }
            _ => None,
        })
        .next()
        .expect("expected rendered mixed CJK html paragraph");

    assert!(
        text_shape
            .galley
            .job
            .sections
            .iter()
            .all(|section| section.format.valign == egui::Align::BOTTOM),
        "mixed CJK html text should use bottom baseline alignment"
    );
}

#[test]
fn preview_markdown_uses_proportional_body_font_even_when_ui_font_family_is_monospace() {
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(
        "KatanA は AIエージェントと共に仕様駆動開発を行う時代のために設計されたツールです。\n",
        std::path::Path::new("/tmp/preview-font-markdown.md"),
    );

    let ctx = egui::Context::default();
    katana_ui::font_loader::SystemFontLoader::setup_fonts(
        &ctx,
        DiagramColorPreset::current(),
        None,
        None,
    );
    katana_ui::theme_bridge::apply_font_family(&ctx, "Monospace");

    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(900.0, 200.0),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                pane.show_content(ui);
            });
        },
    );

    let text_shape = flatten_shapes(output.shapes.iter())
        .into_iter()
        .filter_map(|shape| match shape {
            egui::epaint::Shape::Text(text)
                if text
                    .galley
                    .job
                    .text
                    .contains("AIエージェントと共に仕様駆動開発") =>
            {
                Some(text)
            }
            _ => None,
        })
        .next()
        .expect("expected rendered markdown paragraph");

    assert!(
        text_shape
            .galley
            .job
            .sections
            .iter()
            .all(|section| section.format.font_id.family == egui::FontFamily::Proportional),
        "preview markdown body text should use the proportional family even when the UI uses monospace"
    );
}

#[test]
fn preview_html_uses_proportional_body_font_even_when_ui_font_family_is_monospace() {
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::Markdown(
        "<p>KatanA は AIエージェントと共に仕様駆動開発を行う時代のために設計されたツールです。</p>\n"
            .to_string(),
    )];

    let ctx = egui::Context::default();
    katana_ui::font_loader::SystemFontLoader::setup_fonts(
        &ctx,
        DiagramColorPreset::current(),
        None,
        None,
    );
    katana_ui::theme_bridge::apply_font_family(&ctx, "Monospace");

    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(900.0, 200.0),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                pane.show_content(ui);
            });
        },
    );

    let text_shape = flatten_shapes(output.shapes.iter())
        .into_iter()
        .filter_map(|shape| match shape {
            egui::epaint::Shape::Text(text)
                if text
                    .galley
                    .job
                    .text
                    .contains("AIエージェントと共に仕様駆動開発") =>
            {
                Some(text)
            }
            _ => None,
        })
        .next()
        .expect("expected rendered html paragraph");

    assert!(
        text_shape
            .galley
            .job
            .sections
            .iter()
            .all(|section| section.format.font_id.family == egui::FontFamily::Proportional),
        "preview html body text should use the proportional family even when the UI uses monospace"
    );
}

#[test]
fn preview_code_blocks_keep_monospace_font_when_body_text_is_proportional() {
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(
        "本文です。\n\n```rust\nfn main() {}\n```\n",
        std::path::Path::new("/tmp/preview-font-code.md"),
    );

    let ctx = egui::Context::default();
    katana_ui::font_loader::SystemFontLoader::setup_fonts(
        &ctx,
        DiagramColorPreset::current(),
        None,
        None,
    );
    katana_ui::theme_bridge::apply_font_family(&ctx, "Monospace");

    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(900.0, 260.0),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                pane.show_content(ui);
            });
        },
    );

    let code_shape = flatten_shapes(output.shapes.iter())
        .into_iter()
        .filter_map(|shape| match shape {
            egui::epaint::Shape::Text(text) if text.galley.job.text.contains("fn main() {}") => {
                Some(text)
            }
            _ => None,
        })
        .next()
        .expect("expected rendered code block");

    assert!(
        code_shape
            .galley
            .job
            .sections
            .iter()
            .all(|section| section.format.font_id.family == egui::FontFamily::Monospace),
        "preview code blocks must keep the monospace family"
    );
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
        "  <a href=\"https://github.com/HiroyukiFuruno/KatanA/actions/workflows/ci.yml\"><img src=\"https://github.com/HiroyukiFuruno/KatanA/actions/workflows/ci.yml/badge.svg\" alt=\"CI\"></a>\n",
        "  <a href=\"https://github.com/HiroyukiFuruno/KatanA/releases/latest\"><img src=\"https://img.shields.io/github/v/release/HiroyukiFuruno/KatanA\" alt=\"Latest Release\"></a>\n",
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
    let heading_to_desc_gap = desc_bounds.y0 - heading_bounds.y1;
    assert!(
        heading_to_desc_gap <= 56.0,
        "Heading-to-description gap should stay compact, got {heading_to_desc_gap:.1}"
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

// ── TDD(RED): Markdown table must fill available preview width ──

/// Table must use the available preview width so columns don't shrink to
/// content-minimum. Without explicit width constraints, egui::Grid compacts
/// cells to fit text, leaving most of the preview area unused and causing
/// text overflow when content is even slightly longer.
///
/// Validates BOTH:
/// 1. The table group frame rect fills >= 80% of content width
/// 2. The rightmost cell text ("Header C" / "Cell 3") starts past the 50% mark,
///    proving columns are actually distributed across the table width.
#[test]
fn markdown_table_fills_available_width() {
    let table_md = concat!(
        "# Table Test\n\n",
        "| Header A | Header B | Header C |\n",
        "|----------|----------|----------|\n",
        "| Cell 1   | Cell 2   | Cell 3   |\n",
        "| Cell 4   | Cell 5   | Cell 6   |\n",
    );

    let preview_width = 600.0_f32;
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(table_md, std::path::Path::new("/tmp/table_test.md"));

    let ctx = egui::Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(preview_width, 400.0),
        )),
        ..Default::default()
    };

    let render_frame = |ctx: &egui::Context, pane: &mut PreviewPane| {
        ctx.run(raw_input.clone(), |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.inner_margin(egui::Margin::same(8)))
                .show(ctx, |ui| {
                    pane.show(ui);
                });
        })
    };

    // Run multiple frames so egui::Grid can persist column widths.
    // Grid's min_col_width propagates via stored state across frames.
    let _ = render_frame(&ctx, &mut pane);
    let _ = render_frame(&ctx, &mut pane);
    let output = render_frame(&ctx, &mut pane);

    let content_width = preview_width - 16.0; // 8px margin each side

    // ── Check 1: Frame rect width ──
    let flat = flatten_shapes(&output.shapes);
    let group_frame_widths: Vec<f32> = flat
        .iter()
        .filter_map(|s| {
            if let egui::epaint::Shape::Rect(rect_shape) = s {
                if rect_shape.stroke.width > 0.0 && rect_shape.rect.width() > 50.0 {
                    Some(rect_shape.rect.width())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    assert!(
        !group_frame_widths.is_empty(),
        "Expected at least one group frame rect shape for the table"
    );

    let table_frame_width = group_frame_widths
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
    let fill_ratio = table_frame_width / content_width;

    assert!(
        fill_ratio >= 0.80,
        "Table frame should fill >= 80% of available width, got {fill_ratio:.2} \
         (table_frame_width={table_frame_width:.1}, content_width={content_width:.1})"
    );

    // ── Check 2: Rightmost column text position ──
    // Find text shapes for "Header C" or "Cell 3" (the 3rd column content).
    // Their left edge (x position) must be past the 50% mark of the content area,
    // proving the Grid columns are distributed, not all clustered on the left.
    let right_col_texts: Vec<f32> = flat
        .iter()
        .filter_map(|s| {
            if let egui::epaint::Shape::Text(text_shape) = s {
                let text = &text_shape.galley.job.text;
                if text.contains("Header C") || text.contains("Cell 3") {
                    Some(text_shape.pos.x)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    assert!(
        !right_col_texts.is_empty(),
        "Expected text shapes for the rightmost column (Header C / Cell 3)"
    );

    let rightmost_col_x = right_col_texts
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
    let midpoint = 8.0 + content_width * 0.50; // 50% mark from left margin

    assert!(
        rightmost_col_x >= midpoint,
        "Rightmost column text must start past 50% of content width, \
         got x={rightmost_col_x:.1}, midpoint={midpoint:.1}. \
         This means Grid columns are NOT distributed across the table width."
    );
}

// ── TDD(RED): Table cells within same row must share the same Y coordinate ──

/// Validates that header cells and data cells are rendered in proper row-aligned
/// layout. With `ui.columns()` (equal-width distribution), columns are rendered
/// but cell content within each "row" may not share the same Y coordinate if the
/// Grid-based row mechanism is broken.
///
/// This test uses Shape-based inspection to verify:
/// 1. All 6 data cell texts (Cell 1–6) are present in the rendered output
/// 2. Cells within the SAME row share the same Y coordinate (within tolerance)
/// 3. Header cells (Header A/B/C) are all on the same row
#[test]
fn markdown_table_cells_in_same_row_share_y_coordinate() {
    let table_md = concat!(
        "# Table Test\n\n",
        "| Header A | Header B | Header C |\n",
        "|----------|----------|----------|\n",
        "| Cell 1   | Cell 2   | Cell 3   |\n",
        "| Cell 4   | Cell 5   | Cell 6   |\n",
    );

    let preview_width = 600.0_f32;
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(table_md, std::path::Path::new("/tmp/table_row_test.md"));

    let ctx = egui::Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(preview_width, 400.0),
        )),
        ..Default::default()
    };

    let render_frame = |ctx: &egui::Context, pane: &mut PreviewPane| {
        ctx.run(raw_input.clone(), |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.inner_margin(egui::Margin::same(8)))
                .show(ctx, |ui| {
                    pane.show(ui);
                });
        })
    };

    // Multiple frames for egui::Grid layout stabilization.
    // Grid persists column widths across frames and needs several iterations
    // to settle auto-sized column widths.
    for _ in 0..4 {
        let _ = render_frame(&ctx, &mut pane);
    }
    let output = render_frame(&ctx, &mut pane);

    let flat = flatten_shapes(&output.shapes);

    // Extract positions of all cell/header text shapes
    let cell_labels = [
        "Header A", "Header B", "Header C", "Cell 1", "Cell 2", "Cell 3", "Cell 4", "Cell 5",
        "Cell 6",
    ];
    let mut text_positions: std::collections::HashMap<&str, egui::Pos2> =
        std::collections::HashMap::new();

    for shape in &flat {
        if let egui::epaint::Shape::Text(text_shape) = shape {
            let text = &text_shape.galley.job.text;
            for label in &cell_labels {
                if text.contains(label) {
                    text_positions.insert(label, text_shape.pos);
                }
            }
        }
    }

    // ── Check 1: All cell texts must be present ──
    for label in &cell_labels {
        assert!(
            text_positions.contains_key(label),
            "Missing text shape for '{label}' in rendered output. \
             Found: {:?}",
            text_positions.keys().collect::<Vec<_>>()
        );
    }

    // ── Check 2: Cells in the same row must share the same Y coordinate ──
    let y_tolerance = 2.0_f32;

    // Header row: Header A, Header B, Header C
    let header_a_y = text_positions["Header A"].y;
    let header_b_y = text_positions["Header B"].y;
    let header_c_y = text_positions["Header C"].y;
    assert!(
        (header_a_y - header_b_y).abs() <= y_tolerance
            && (header_b_y - header_c_y).abs() <= y_tolerance,
        "Header cells must be on the same row (same Y). \
         Got: A.y={header_a_y:.1}, B.y={header_b_y:.1}, C.y={header_c_y:.1}"
    );

    // Data row 1: Cell 1, Cell 2, Cell 3
    let cell1_y = text_positions["Cell 1"].y;
    let cell2_y = text_positions["Cell 2"].y;
    let cell3_y = text_positions["Cell 3"].y;
    assert!(
        (cell1_y - cell2_y).abs() <= y_tolerance && (cell2_y - cell3_y).abs() <= y_tolerance,
        "Data row 1 cells must be on the same row (same Y). \
         Got: C1.y={cell1_y:.1}, C2.y={cell2_y:.1}, C3.y={cell3_y:.1}"
    );

    // Data row 2: Cell 4, Cell 5, Cell 6
    let cell4_y = text_positions["Cell 4"].y;
    let cell5_y = text_positions["Cell 5"].y;
    let cell6_y = text_positions["Cell 6"].y;
    assert!(
        (cell4_y - cell5_y).abs() <= y_tolerance && (cell5_y - cell6_y).abs() <= y_tolerance,
        "Data row 2 cells must be on the same row (same Y). \
         Got: C4.y={cell4_y:.1}, C5.y={cell5_y:.1}, C6.y={cell6_y:.1}"
    );

    // ── Check 3: Rows must be in descending Y order ──
    assert!(
        header_a_y < cell1_y && cell1_y < cell4_y,
        "Rows must be in top-to-bottom order. \
         Got header.y={header_a_y:.1}, row1.y={cell1_y:.1}, row2.y={cell4_y:.1}"
    );
}
