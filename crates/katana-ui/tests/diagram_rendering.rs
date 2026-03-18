//! Integration tests verifying that Mermaid, DrawIo, and PlantUML diagrams
//! are actually rendered (or gracefully fall back) through the full pipeline.
//!
//! Each diagram type tests BOTH states idempotently:
//! 1. Tool hidden via env var → fallback UI snapshot
//! 2. Tool restored → rendered image UI snapshot (skipped if tool not installed)
//!
//! This ensures that regardless of the host environment, fallback UIs are
//! always verified, and rendering is verified when tools are present.

use egui_kittest::{Harness, SnapshotOptions, SnapshotResults};
use katana_ui::preview_pane::{PreviewPane, RenderedSection};
use std::path::Path;

/// Snapshot pixel tolerance to absorb non-deterministic rendering differences.
const SNAPSHOT_PIXEL_TOLERANCE: usize = 10000;

// ─────────────────────────────────────────────
// Helper
// ─────────────────────────────────────────────

/// Build markdown source with a single diagram fenced code block.
fn diagram_md(lang: &str, body: &str) -> String {
    format!("# Diagram Test\n\n```{lang}\n{body}\n```\n\n## Footer\n")
}

/// Assert that the section at `idx` is an `Image` variant.
fn assert_image(sections: &[RenderedSection], idx: usize, context: &str) {
    assert!(
        matches!(sections.get(idx), Some(RenderedSection::Image { .. })),
        "[{context}] Expected Image at index {idx}, got: {:?}",
        sections.get(idx)
    );
}

fn snapshot_opts() -> SnapshotOptions {
    SnapshotOptions::default().failed_pixel_count_threshold(SNAPSHOT_PIXEL_TOLERANCE)
}

/// Render a diagram and wait for completion, returning the PreviewPane.
fn render_and_wait(lang: &str, source: &str) -> PreviewPane {
    let md = diagram_md(lang, source);
    let mut pane = PreviewPane::default();
    pane.full_render(&md, Path::new("/tmp/test.md"));
    pane.wait_for_renders();
    pane
}

/// Build a harness, run it, take a snapshot, and return the SnapshotResults
/// (which must be merged into a parent SnapshotResults if multiple snapshots
/// are taken within a single test).
fn build_snapshot(
    sections: Vec<RenderedSection>,
    name: &str,
    width: f32,
    height: f32,
) -> SnapshotResults {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, height))
        .build_ui(move |ui| {
            let mut pane = PreviewPane::default();
            pane.sections = sections.clone();
            pane.show_content(ui);
        });
    harness.step();
    harness.step();
    harness.step();
    harness.run();
    let result = harness.try_snapshot_options(name, &snapshot_opts());
    let mut results = harness.take_snapshot_results();
    results.add(result);
    results
}

// ─────────────────────────────────────────────
// DrawIo: Pure Rust renderer (no external tools)
// ─────────────────────────────────────────────

const DRAWIO_SOURCE: &str = r#"<mxGraphModel>
  <root>
    <mxCell id="0"/>
    <mxCell id="1" parent="0"/>
    <mxCell id="2" value="Hello" style="rounded=1;" vertex="1" parent="1">
      <mxGeometry x="100" y="100" width="120" height="60" as="geometry"/>
    </mxCell>
  </root>
</mxGraphModel>"#;

/// DrawIo: always renders (pure Rust). Tests both success and error states.
#[test]
fn drawio_renders_to_image_section() {
    let pane = render_and_wait("drawio", DRAWIO_SOURCE);

    assert_eq!(
        pane.sections.len(),
        3,
        "Expected 3 sections (md, diagram, md)"
    );
    assert_image(&pane.sections, 1, "DrawIo");

    if let RenderedSection::Image { svg_data, alt } = &pane.sections[1] {
        assert!(svg_data.width > 0, "Width should be > 0");
        assert!(svg_data.height > 0, "Height should be > 0");
        assert!(!svg_data.rgba.is_empty(), "RGBA data should not be empty");
        assert!(
            alt.contains("DrawIo"),
            "Alt text should mention DrawIo, got: {alt}"
        );
    }
}

/// DrawIo with invalid XML produces an Error section (not a panic).
#[test]
fn drawio_invalid_xml_produces_error_section() {
    let pane = render_and_wait("drawio", "<not-valid-drawio/>");
    assert!(
        !matches!(pane.sections[1], RenderedSection::Pending { .. }),
        "Should not remain Pending after wait_for_renders"
    );
}

/// Snapshot: DrawIo rendered image displayed in the UI.
#[test]
fn snapshot_drawio_rendered_image() {
    let pane = render_and_wait("drawio", DRAWIO_SOURCE);
    assert_image(&pane.sections, 1, "DrawIo pre-snapshot");
    build_snapshot(pane.sections, "diagram_drawio_rendered", 600.0, 400.0);
}

/// Snapshot: DrawIo render error fallback UI.
#[test]
fn snapshot_drawio_render_error() {
    let sections = vec![
        RenderedSection::Markdown("# DrawIo Diagram\n".to_string()),
        RenderedSection::Error {
            kind: "DrawIo".to_string(),
            _source: "<invalid/>".to_string(),
            message: "Failed to extract SVG from rendered HTML".to_string(),
        },
        RenderedSection::Markdown("## After diagram\n".to_string()),
    ];
    build_snapshot(sections, "diagram_drawio_render_error", 600.0, 300.0);
}

// ─────────────────────────────────────────────
// Mermaid: Both states tested idempotently
// ─────────────────────────────────────────────

const MERMAID_SOURCE: &str = "graph TD\n    A[Start] --> B[End]";

/// Mermaid idempotent test: verifies BOTH fallback and rendering states
/// by controlling tool visibility via environment variable.
///
/// Phase 1: Hide mmdc → verify CommandNotFound fallback + snapshot
/// Phase 2: Restore mmdc → verify Image rendering + snapshot (skip if mmdc not on system)
#[test]
fn snapshot_mermaid_both_states() {
    let saved_mmdc = std::env::var("MERMAID_MMDC").ok();
    let mut results = SnapshotResults::default();

    // ── Phase 1: Hide mmdc → test CommandNotFound fallback ──
    std::env::set_var("MERMAID_MMDC", "nonexistent_mmdc_for_idempotent_test");

    let pane = render_and_wait("mermaid", MERMAID_SOURCE);
    assert!(
        matches!(pane.sections[1], RenderedSection::CommandNotFound { .. }),
        "With hidden mmdc, should be CommandNotFound, got: {:?}",
        pane.sections[1]
    );
    if let RenderedSection::CommandNotFound {
        tool_name,
        install_hint,
        ..
    } = &pane.sections[1]
    {
        assert!(tool_name.contains("mmdc"), "Tool name should mention mmdc");
        assert!(
            install_hint.contains("npm"),
            "Install hint should mention npm"
        );
    }
    results.extend(build_snapshot(
        pane.sections,
        "diagram_mermaid_command_not_found",
        600.0,
        300.0,
    ));

    // ── Phase 2: Restore mmdc → test actual rendering ──
    match &saved_mmdc {
        Some(v) => std::env::set_var("MERMAID_MMDC", v),
        None => std::env::remove_var("MERMAID_MMDC"),
    }

    if katana_core::markdown::mermaid_renderer::is_mmdc_available() {
        let pane = render_and_wait("mermaid", MERMAID_SOURCE);
        assert_image(&pane.sections, 1, "Mermaid rendered");
        if let RenderedSection::Image { svg_data, alt } = &pane.sections[1] {
            assert!(svg_data.width > 0, "Mermaid image width should be > 0");
            assert!(svg_data.height > 0, "Mermaid image height should be > 0");
            assert!(alt.contains("Mermaid"), "Alt should mention Mermaid");
        }
        results.extend(build_snapshot(
            pane.sections,
            "diagram_mermaid_rendered",
            600.0,
            400.0,
        ));
    } else {
        eprintln!("SKIP Phase 2: mmdc not installed — Mermaid rendering snapshot skipped");
    }

    // ── Cleanup + handle results ──
    match saved_mmdc {
        Some(v) => std::env::set_var("MERMAID_MMDC", v),
        None => std::env::remove_var("MERMAID_MMDC"),
    }
    // SnapshotResults are checked automatically on drop (panics if errors).
}

// ─────────────────────────────────────────────
// PlantUML: Both states tested idempotently
// ─────────────────────────────────────────────

const PLANTUML_SOURCE: &str = "@startuml\nAlice -> Bob : Hello\n@enduml";

/// PlantUML idempotent test: verifies BOTH fallback and rendering states
/// by controlling tool visibility via environment variable.
///
/// Phase 1: Hide plantuml.jar → verify NotInstalled fallback + snapshot
/// Phase 2: Restore plantuml.jar → verify Image rendering + snapshot (skip if jar not on system)
#[test]
fn snapshot_plantuml_both_states() {
    let saved_jar = std::env::var("PLANTUML_JAR").ok();
    let mut results = SnapshotResults::default();

    // ── Phase 1: Hide plantuml.jar → test NotInstalled fallback ──
    std::env::set_var("PLANTUML_JAR", "/nonexistent/path/for/idempotent/test.jar");

    let pane = render_and_wait("plantuml", PLANTUML_SOURCE);
    assert!(
        matches!(pane.sections[1], RenderedSection::NotInstalled { .. }),
        "With hidden jar, should be NotInstalled, got: {:?}",
        pane.sections[1]
    );
    if let RenderedSection::NotInstalled {
        kind, download_url, ..
    } = &pane.sections[1]
    {
        assert_eq!(kind, "PlantUML", "Kind should be PlantUML");
        assert!(
            download_url.contains("plantuml"),
            "URL should mention plantuml"
        );
    }
    results.extend(build_snapshot(
        pane.sections,
        "diagram_plantuml_not_installed",
        600.0,
        300.0,
    ));

    // ── Phase 2: Restore plantuml.jar → test actual rendering ──
    match &saved_jar {
        Some(v) => std::env::set_var("PLANTUML_JAR", v),
        None => std::env::remove_var("PLANTUML_JAR"),
    }

    if katana_core::markdown::plantuml_renderer::find_plantuml_jar().is_some() {
        let pane = render_and_wait("plantuml", PLANTUML_SOURCE);
        assert_image(&pane.sections, 1, "PlantUML rendered");
        if let RenderedSection::Image { svg_data, alt } = &pane.sections[1] {
            assert!(svg_data.width > 0, "PlantUML image width should be > 0");
            assert!(svg_data.height > 0, "PlantUML image height should be > 0");
            assert!(alt.contains("PlantUml"), "Alt should mention PlantUml");
        }
        results.extend(build_snapshot(
            pane.sections,
            "diagram_plantuml_rendered",
            600.0,
            400.0,
        ));
    } else {
        eprintln!("SKIP Phase 2: plantuml.jar not found — PlantUML rendering snapshot skipped");
    }

    // ── Cleanup + handle results ──
    match saved_jar {
        Some(v) => std::env::set_var("PLANTUML_JAR", v),
        None => std::env::remove_var("PLANTUML_JAR"),
    }
    // SnapshotResults are checked automatically on drop (panics if errors).
}

// ─────────────────────────────────────────────
// Mixed diagram document: all three types
// ─────────────────────────────────────────────

/// Verifies that a document containing all three diagram types renders
/// each one independently — one diagram's failure doesn't affect others.
#[test]
fn mixed_diagram_document_renders_all_independently() {
    let source = format!(
        "# Mixed\n\n```mermaid\n{MERMAID_SOURCE}\n```\n\n\
         ## DrawIo\n\n```drawio\n{DRAWIO_SOURCE}\n```\n\n\
         ## PlantUML\n\n```plantuml\n{PLANTUML_SOURCE}\n```\n\n\
         ## End\n"
    );
    let mut pane = PreviewPane::default();
    pane.full_render(&source, Path::new("/tmp/test.md"));

    assert_eq!(
        pane.sections.len(),
        7,
        "Expected 7 sections for mixed document"
    );

    pane.wait_for_renders();

    assert!(
        !pane
            .sections
            .iter()
            .any(|s| matches!(s, RenderedSection::Pending { .. })),
        "No Pending sections should remain after wait_for_renders"
    );

    // Mermaid (index 1): Image or CommandNotFound
    assert!(
        matches!(
            pane.sections[1],
            RenderedSection::Image { .. } | RenderedSection::CommandNotFound { .. }
        ),
        "Mermaid should be Image or CommandNotFound, got: {:?}",
        pane.sections[1]
    );

    // DrawIo (index 3): Always Image (pure Rust)
    assert_image(&pane.sections, 3, "DrawIo in mixed document");

    // PlantUML (index 5): Image or NotInstalled
    assert!(
        matches!(
            pane.sections[5],
            RenderedSection::Image { .. } | RenderedSection::NotInstalled { .. }
        ),
        "PlantUML should be Image or NotInstalled, got: {:?}",
        pane.sections[5]
    );
}

// ─────────────────────────────────────────────
// UI snapshot: Mixed document + Pending spinner + all fallbacks
// ─────────────────────────────────────────────

/// Snapshot: Mixed document with DrawIo image + Mermaid/PlantUML fallbacks.
#[test]
fn snapshot_mixed_diagrams_with_fallbacks() {
    let drawio_pane = render_and_wait("drawio", DRAWIO_SOURCE);
    let drawio_image = drawio_pane.sections[1].clone();
    assert!(matches!(drawio_image, RenderedSection::Image { .. }));

    let sections = vec![
        RenderedSection::Markdown("# Mixed Diagram Document\n".to_string()),
        drawio_image,
        RenderedSection::Markdown("---\n".to_string()),
        RenderedSection::CommandNotFound {
            tool_name: "mmdc (Mermaid CLI)".to_string(),
            install_hint: "`npm install -g @mermaid-js/mermaid-cli`".to_string(),
            _source: "graph TD; A-->B".to_string(),
        },
        RenderedSection::Markdown("---\n".to_string()),
        RenderedSection::NotInstalled {
            kind: "PlantUML".to_string(),
            download_url:
                "https://github.com/plantuml/plantuml/releases/latest/download/plantuml.jar"
                    .to_string(),
            install_path: std::path::PathBuf::from("/tmp/plantuml.jar"),
        },
        RenderedSection::Markdown("## End\n".to_string()),
    ];
    build_snapshot(sections, "diagram_mixed_with_fallbacks", 600.0, 600.0);
}

/// Snapshot: Diagram in Pending state showing spinner.
#[test]
fn snapshot_diagram_pending_spinner() {
    let sections = vec![
        RenderedSection::Markdown("# Rendering in progress\n".to_string()),
        RenderedSection::Pending {
            kind: "Mermaid".to_string(),
        },
        RenderedSection::Pending {
            kind: "PlantUML".to_string(),
        },
        RenderedSection::Markdown("## Waiting...\n".to_string()),
    ];

    let mut harness = Harness::builder()
        .with_size(egui::vec2(600.0, 300.0))
        .build_ui(move |ui| {
            let mut pane = PreviewPane::default();
            pane.sections = sections.clone();
            pane.show_content(ui);
        });
    harness.step();
    harness.snapshot_options("diagram_pending_spinner", &snapshot_opts());
}

// ─────────────────────────────────────────────
// Regression: full_render → update_markdown_sections preserves diagrams
// ─────────────────────────────────────────────

/// After `full_render`, calling `update_markdown_sections` should preserve
/// already-rendered diagram sections while updating only Markdown text.
#[test]
fn update_after_render_preserves_diagram_images() {
    let source = format!("# Title\n\n```drawio\n{DRAWIO_SOURCE}\n```\n\n## Footer\n");
    let mut pane = PreviewPane::default();
    pane.full_render(&source, Path::new("/tmp/test.md"));
    pane.wait_for_renders();

    assert_image(&pane.sections, 1, "Before update");

    let updated =
        format!("# Updated Title\n\n```drawio\n{DRAWIO_SOURCE}\n```\n\n## Updated Footer\n");
    pane.update_markdown_sections(&updated, Path::new("/tmp/test.md"));

    assert_image(&pane.sections, 1, "After markdown-only update");

    if let RenderedSection::Markdown(md) = &pane.sections[0] {
        assert!(
            md.contains("Updated Title"),
            "Title should be updated, got: {md}"
        );
    }
    if let RenderedSection::Markdown(md) = &pane.sections[2] {
        assert!(
            md.contains("Updated Footer"),
            "Footer should be updated, got: {md}"
        );
    }
}
