//! Integration tests verifying that Mermaid, DrawIo, and PlantUML diagrams
//! are actually rendered (or gracefully fall back) through the full pipeline.
//!
//! Each diagram type tests BOTH states idempotently:
//! 1. Tool hidden via env var → fallback UI snapshot
//! 2. Tool restored → rendered image UI snapshot (skipped if tool not installed)
//!
//! This ensures that regardless of the host environment, fallback UIs are
//! always verified, and rendering is verified when tools are present.

use egui_kittest::{kittest::Queryable, Harness};
use katana_ui::preview_pane::{PreviewPane, RenderedSection};
use std::path::Path;

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

/// Render a diagram and wait for completion, returning the PreviewPane.
fn render_and_wait(lang: &str, source: &str) -> PreviewPane {
    let md = diagram_md(lang, source);
    let mut pane = PreviewPane::default();
    pane.full_render(
        &md,
        Path::new("/tmp/test.md"),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        false,
        4,
    );
    pane.wait_for_renders();
    pane
}

/// Build a harness and render the provided sections for semantic UI assertions.
fn build_harness(sections: Vec<RenderedSection>, width: f32, height: f32) -> Harness<'static> {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, height))
        .build_ui(move |ui| {
            let mut pane = PreviewPane::default();
            pane.sections = sections.clone();
            pane.show_content(ui);
        });
    for _ in 0..5 {
        harness.step();
    }
    harness.run();
    harness
}

fn assert_standard_diagram_markdown_visible(harness: &Harness) {
    let _heading = harness.get_by_label("Diagram Test");
    let _footer = harness.get_by_label("Footer");
}

/// Build a harness, run it, take a snapshot, and return the SnapshotResults
/// (which must be merged into a parent SnapshotResults if multiple snapshots
/// are taken within a single test).
/// Fallback: DrawIo render error fallback UI.
#[test]
fn drawio_render_error_ui() {
    let sections = vec![
        RenderedSection::Markdown("# DrawIo Diagram\n".to_string()),
        RenderedSection::Error {
            kind: "DrawIo".to_string(),
            _source: "<invalid/>".to_string(),
            message: "Failed to extract SVG from rendered HTML".to_string(),
        },
        RenderedSection::Markdown("## After diagram\n".to_string()),
    ];
    let harness = build_harness(sections, 600.0, 300.0);
    // Verify fallback UI text using i18n
    let expected_error = katana_ui::i18n::tf(
        &katana_ui::i18n::get().error.render_error,
        &[
            ("kind", "DrawIo"),
            ("message", "Failed to extract SVG from rendered HTML"),
        ],
    );
    // Verify it doesn't panic and error text exists
    let _ = harness.get_by_label(&expected_error);
}

// ─────────────────────────────────────────────
// Mermaid: Both states tested idempotently
// ─────────────────────────────────────────────

const MERMAID_SOURCE: &str = "graph TD\n    A[Start] --> B[End]";

/// Mermaid idempotent test: verifies BOTH fallback and rendering states
/// by controlling tool visibility via environment variable.
#[test]
fn mermaid_both_states_render_semantically() {
    let saved_mmdc = std::env::var("MERMAID_MMDC").ok();

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
        let expected = katana_ui::i18n::get()
            .error
            .missing_dependency
            .replace("{tool_name}", tool_name)
            .replace("{install_hint}", install_hint);
        let harness = build_harness(pane.sections.clone(), 600.0, 300.0);
        assert_standard_diagram_markdown_visible(&harness);
        let _fallback = harness.get_by_label(&expected);
    }

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
        let harness = build_harness(pane.sections, 600.0, 400.0);
        assert_standard_diagram_markdown_visible(&harness);
    } else {
        eprintln!("SKIP Phase 2: mmdc not installed — Mermaid rendering semantic UI check skipped");
    }

    // ── Cleanup + handle results ──
    match saved_mmdc {
        Some(v) => std::env::set_var("MERMAID_MMDC", v),
        None => std::env::remove_var("MERMAID_MMDC"),
    }
}

// ─────────────────────────────────────────────
// PlantUML: Both states tested idempotently
// ─────────────────────────────────────────────

const PLANTUML_SOURCE: &str = "@startuml\nAlice -> Bob : Hello\n@enduml";

/// PlantUML idempotent test: verifies BOTH fallback and rendering states
/// by controlling tool visibility via environment variable.
#[test]
fn plantuml_both_states_render_semantically() {
    let saved_jar = std::env::var("PLANTUML_JAR").ok();

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
        let tool_msg = katana_ui::i18n::tf(
            &katana_ui::i18n::get().tool.not_installed,
            &[("tool", kind)],
        );
        let harness = build_harness(pane.sections.clone(), 600.0, 300.0);
        assert_standard_diagram_markdown_visible(&harness);
        let _fallback = harness.get_by_label(&tool_msg);
    }

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
        let harness = build_harness(pane.sections, 600.0, 400.0);
        assert_standard_diagram_markdown_visible(&harness);
    } else {
        eprintln!(
            "SKIP Phase 2: plantuml.jar not found — PlantUML rendering semantic UI check skipped"
        );
    }

    // ── Cleanup + handle results ──
    match saved_jar {
        Some(v) => std::env::set_var("PLANTUML_JAR", v),
        None => std::env::remove_var("PLANTUML_JAR"),
    }
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
    pane.full_render(
        &source,
        Path::new("/tmp/test.md"),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        false,
        4,
    );

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
// UI semantic check: Mixed document + Pending spinner + all fallbacks
// ─────────────────────────────────────────────

/// Mixed document fallback UI remains semantically correct without snapshots.
#[test]
fn mixed_diagrams_with_fallbacks_render_semantically() {
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
    let expected_missing = katana_ui::i18n::get()
        .error
        .missing_dependency
        .replace("{tool_name}", "mmdc (Mermaid CLI)")
        .replace("{install_hint}", "`npm install -g @mermaid-js/mermaid-cli`");
    let expected_not_installed = katana_ui::i18n::tf(
        &katana_ui::i18n::get().tool.not_installed,
        &[("tool", "PlantUML")],
    );
    let harness = build_harness(sections, 600.0, 600.0);
    let _heading = harness.get_by_label("Mixed Diagram Document");
    let _footer = harness.get_by_label("End");
    let _missing = harness.get_by_label(&expected_missing);
    let _not_installed = harness.get_by_label(&expected_not_installed);
}

/// Fallback: Diagram in Pending state showing spinner.
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
    // Use run_steps instead of run() because spin animations loop forever.
    harness.run_steps(5);
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
    pane.full_render(
        &source,
        Path::new("/tmp/test.md"),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        false,
        4,
    );
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
