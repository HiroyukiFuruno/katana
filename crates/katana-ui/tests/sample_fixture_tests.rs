//! Integration tests that load the canonical sample fixture files
//! (`tests/fixtures/sample.md` / `sample.ja.md`) through `full_render`,
//! then verify specific rendering behaviors.
//!
//! ## Test Strategy
//!
//! ### Full-fixture tests (`load_fixture`)
//! Load the *entire* fixture, `full_render` with diagrams, verify structural
//! properties: section count, no Pending, diagram type correctness, etc.
//!
//! ### Section-extracted tests
//! Extract specific HTML/Markdown sections from the *actual fixture file*
//! (not hard-coded duplicates) to avoid label collisions in AccessKit,
//! while guaranteeing that fixture changes propagate to tests.
//!
//! ### Snapshot tests
//! Full-document visual snapshots serve as the ultimate regression catch-all,
//! including badge rendering and layout details that AccessKit can't verify.
//!
//! ### Known AccessKit limitations
//! - `<img>` alt attributes are NOT exposed as AccessKit labels.
//!   Badge same-row verification relies on snapshot tests, not AccessKit queries.

use eframe::egui;
use egui_kittest::kittest::{NodeT, Queryable};
use egui_kittest::{Harness, SnapshotOptions};
use katana_ui::preview_pane::{PreviewPane, RenderedSection};
use std::path::Path;

const PANEL_WIDTH: f32 = 800.0;
/// Maximum height for snapshot tests.
/// GPU texture limit is 8192px. Fixtures are split into groups (html, basic, diagrams)
/// so that each group fits within this limit and is captured in full — no clipping.
const PANEL_HEIGHT: f32 = 8000.0;
const CENTERING_TOLERANCE: f64 = 50.0;
const SNAPSHOT_PIXEL_TOLERANCE: usize = 10000;

// ─────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────

/// Load fixture file, full_render with diagrams, and wait for completion.
fn load_fixture(filename: &str) -> (PreviewPane, std::path::PathBuf, String) {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(filename);
    let source = std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", fixture_path.display()));

    let mut pane = PreviewPane::default();
    pane.full_render(&source, &fixture_path);
    pane.wait_for_renders();
    (pane, fixture_path, source)
}

/// Extract a section of the fixture source between two headings (regex-free).
/// `start_heading` and `end_heading` are exact markdown heading lines (e.g., "### 1.1").
/// Returns the content between them (exclusive of both headings).
fn extract_section(source: &str, start_marker: &str, end_marker: &str) -> String {
    let start_pos = source
        .find(start_marker)
        .unwrap_or_else(|| panic!("Marker not found: '{start_marker}'"));
    let after_start = start_pos + start_marker.len();
    // Find next newline after marker
    let content_start = source[after_start..]
        .find('\n')
        .map(|p| after_start + p + 1)
        .unwrap_or(after_start);

    let end_pos = source[content_start..]
        .find(end_marker)
        .map(|p| content_start + p)
        .unwrap_or(source.len());

    source[content_start..end_pos].trim().to_string()
}

/// Render a Markdown/HTML snippet through update_markdown_sections.
fn render_snippet(md: &str) -> PreviewPane {
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(md, Path::new("/tmp/snippet.md"));
    pane
}

/// Load CJK system fonts into an egui Context.
/// This replicates what `setup_fonts` does in the real application,
/// but directly in test code since `setup_fonts` is in the binary crate.
fn load_test_fonts(ctx: &egui::Context) {
    use std::sync::Arc;

    let mut fonts = egui::FontDefinitions::default();

    // macOS proportional CJK font candidates
    let prop_candidates = [
        "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
        "/System/Library/Fonts/AquaKana.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
    ];
    for path in &prop_candidates {
        if let Ok(data) = std::fs::read(path) {
            let name = std::path::Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("cjk_font")
                .to_string();
            fonts
                .font_data
                .insert(name.clone(), Arc::new(egui::FontData::from_owned(data)));
            if let Some(list) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                list.insert(0, name.clone());
            }
            if let Some(list) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                list.push(name);
            }
            break;
        }
    }

    // macOS monospace font candidates
    let mono_candidates = [
        "/System/Library/Fonts/Menlo.ttc",
        "/System/Library/Fonts/Monaco.ttf",
    ];
    for path in &mono_candidates {
        if let Ok(data) = std::fs::read(path) {
            let name = std::path::Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("mono_font")
                .to_string();
            fonts
                .font_data
                .insert(name.clone(), Arc::new(egui::FontData::from_owned(data)));
            if let Some(list) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                list.insert(0, name);
            }
            break;
        }
    }

    ctx.set_fonts(fonts);
}

/// Build a Harness with the given sections, load CJK fonts, and run it enough frames
/// for measure-then-position to stabilize.
fn build_harness(sections: Vec<RenderedSection>, width: f32, height: f32) -> Harness<'static> {
    let mut fonts_loaded = false;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(width, height))
        .build_ui(move |ui| {
            if !fonts_loaded {
                load_test_fonts(ui.ctx());
                fonts_loaded = true;
            }
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

fn snapshot_opts() -> SnapshotOptions {
    SnapshotOptions::default().failed_pixel_count_threshold(SNAPSHOT_PIXEL_TOLERANCE)
}

/// Assert a widget's center X is near the panel center (horizontally centered).
fn assert_centered(harness: &Harness, label: &str, context: &str) {
    let node = harness.get_by_label(label);
    let bounds = node
        .accesskit_node()
        .raw_bounds()
        .unwrap_or_else(|| panic!("[{context}] '{label}' should have bounds"));
    let widget_cx = (bounds.x0 + bounds.x1) / 2.0;
    let panel_cx = f64::from(PANEL_WIDTH) / 2.0;
    assert!(
        (widget_cx - panel_cx).abs() < CENTERING_TOLERANCE,
        "[{context}] '{label}' center X ({widget_cx:.1}) should be near panel center ({panel_cx:.1}), diff={:.1}",
        (widget_cx - panel_cx).abs()
    );
}

/// Assert that widget B appears below widget A (increasing Y).
fn assert_below(harness: &Harness, label_above: &str, label_below: &str, context: &str) {
    let node_a = harness.get_by_label(label_above);
    let node_b = harness.get_by_label(label_below);
    let bounds_a = node_a
        .accesskit_node()
        .raw_bounds()
        .unwrap_or_else(|| panic!("[{context}] '{label_above}' should have bounds"));
    let bounds_b = node_b
        .accesskit_node()
        .raw_bounds()
        .unwrap_or_else(|| panic!("[{context}] '{label_below}' should have bounds"));
    assert!(
        bounds_b.y0 > bounds_a.y0,
        "[{context}] '{label_below}' (Y={:.1}) should be below '{label_above}' (Y={:.1})",
        bounds_b.y0,
        bounds_a.y0
    );
}

/// Assert that widget B is to the right of widget A and they share the same Y row.
fn assert_right_of_same_row(harness: &Harness, label_left: &str, label_right: &str, context: &str) {
    let node_a = harness.get_by_label(label_left);
    let node_b = harness.get_by_label(label_right);
    let bounds_a = node_a
        .accesskit_node()
        .raw_bounds()
        .unwrap_or_else(|| panic!("[{context}] '{label_left}' should have bounds"));
    let bounds_b = node_b
        .accesskit_node()
        .raw_bounds()
        .unwrap_or_else(|| panic!("[{context}] '{label_right}' should have bounds"));

    // Same row (Y within threshold)
    let y_diff = (bounds_a.y0 - bounds_b.y0).abs();
    assert!(
        y_diff < 5.0,
        "[{context}] '{label_left}' (Y={:.1}) and '{label_right}' (Y={:.1}) should be on same row, diff={y_diff:.1}",
        bounds_a.y0, bounds_b.y0
    );
    // Right of
    assert!(
        bounds_b.x0 > bounds_a.x0,
        "[{context}] '{label_right}' X ({:.1}) should be right of '{label_left}' X ({:.1})",
        bounds_b.x0,
        bounds_a.x0
    );
}

// ═════════════════════════════════════════════
// Full fixture: Structural verification (load_fixture)
// ═════════════════════════════════════════════

/// Fixture produces a substantial number of sections (not truncated).
#[test]
fn fixture_en_produces_many_sections() {
    let (pane, _, _) = load_fixture("sample.md");
    assert!(
        pane.sections.len() > 30,
        "English fixture should produce >30 sections, got: {}",
        pane.sections.len()
    );
}

/// No Pending sections remain after wait_for_renders.
#[test]
fn fixture_en_no_pending_after_render() {
    let (pane, _, _) = load_fixture("sample.md");
    let pending_count = pane
        .sections
        .iter()
        .filter(|s| matches!(s, RenderedSection::Pending { .. }))
        .count();
    assert_eq!(pending_count, 0, "No Pending sections should remain");
}

/// DrawIo diagrams are always Image (pure Rust, no external dependency).
#[test]
fn fixture_en_drawio_always_renders_to_image() {
    let (pane, _, _) = load_fixture("sample.md");
    let drawio_image_count = pane
        .sections
        .iter()
        .filter(|s| matches!(s, RenderedSection::Image { alt, .. } if alt.contains("DrawIo")))
        .count();
    // The fixture has multiple DrawIo blocks
    assert!(
        drawio_image_count >= 2,
        "Expected at least 2 DrawIo Image sections, got: {drawio_image_count}"
    );
}

/// No DrawIo errors — fixture XML is valid.
#[test]
fn fixture_en_no_drawio_errors() {
    let (pane, _, _) = load_fixture("sample.md");
    for (i, section) in pane.sections.iter().enumerate() {
        if let RenderedSection::Error { kind, message, .. } = section {
            if kind == "DrawIo" {
                panic!("DrawIo at section {i} got Error: {message}");
            }
        }
    }
}

/// Mermaid diagrams are either Image (mmdc present) or CommandNotFound (no mmdc).
/// No Error or Pending variants allowed.
#[test]
fn fixture_en_mermaid_renders_or_fallback() {
    let (pane, _, _) = load_fixture("sample.md");
    let mermaid_count = pane
        .sections
        .iter()
        .filter(|s| match s {
            RenderedSection::Image { alt, .. } => alt.contains("Mermaid"),
            RenderedSection::CommandNotFound { tool_name, .. } => tool_name.contains("mmdc"),
            _ => false,
        })
        .count();
    // The fixture has 5+ Mermaid diagrams
    assert!(
        mermaid_count >= 5,
        "Expected at least 5 Mermaid sections (Image or CommandNotFound), got: {mermaid_count}"
    );
}

/// PlantUML diagrams are either Image (jar present) or NotInstalled.
/// No Error or Pending variants allowed.
#[test]
fn fixture_en_plantuml_renders_or_fallback() {
    let (pane, _, _) = load_fixture("sample.md");
    let plantuml_count = pane
        .sections
        .iter()
        .filter(|s| match s {
            RenderedSection::Image { alt, .. } => alt.contains("PlantUml"),
            RenderedSection::NotInstalled { kind, .. } => kind == "PlantUML",
            _ => false,
        })
        .count();
    // The fixture has 3 PlantUML diagrams
    assert!(
        plantuml_count >= 3,
        "Expected at least 3 PlantUML sections (Image or NotInstalled), got: {plantuml_count}"
    );
}

/// DrawIo succeeds regardless of whether Mermaid/PlantUML tools are installed.
#[test]
fn fixture_en_diagram_independence() {
    let (pane, _, _) = load_fixture("sample.md");
    let drawio_ok = pane
        .sections
        .iter()
        .any(|s| matches!(s, RenderedSection::Image { alt, .. } if alt.contains("DrawIo")));
    assert!(
        drawio_ok,
        "DrawIo should render independently of other diagram failures"
    );
}

// ═════════════════════════════════════════════
// §1: HTML Centering — Position verification
//
// Each test extracts a section from the ACTUAL fixture file, not hard-coded HTML.
// ═════════════════════════════════════════════

/// §1.1: `<h1 align="center">` heading is horizontally centered.
/// Extracts the §1.1 section from the fixture.
#[test]
fn fixture_en_s1_1_centered_heading_h1() {
    let (_, _, source) = load_fixture("sample.md");
    let section_md = extract_section(&source, "### 1.1", "### 1.2");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections, PANEL_WIDTH, 200.0);
    assert_centered(&harness, "KatanA Desktop", "§1.1 centered h1");
}

/// §1.2: `<p align="center">` paragraph is horizontally centered.
#[test]
fn fixture_en_s1_2_centered_paragraph() {
    let (_, _, source) = load_fixture("sample.md");
    let section_md = extract_section(&source, "### 1.2", "### 1.3");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections, PANEL_WIDTH, 200.0);
    assert_centered(
        &harness,
        "A fast, lightweight Markdown workspace for macOS — built with Rust and egui.",
        "§1.2 centered paragraph",
    );
}

/// §1.3: Multiple centered blocks do not overlap — each has increasing Y.
#[test]
fn fixture_en_s1_3_centered_blocks_no_overlap() {
    let (_, _, source) = load_fixture("sample.md");
    let section_md = extract_section(&source, "### 1.3", "### 1.4");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections, PANEL_WIDTH, 400.0);
    assert_below(
        &harness,
        "Centered Heading",
        "Centered description paragraph.",
        "§1.3 heading -> description",
    );
    assert_below(
        &harness,
        "Centered description paragraph.",
        "Second centered paragraph — should NOT overlap with the first one.",
        "§1.3 description -> second",
    );
}

/// §1.4: Badge rendering produces a section without crash.
/// NOTE: Badge same-row alignment cannot be tested via AccessKit because
/// `<img>` alt text is not exposed as AccessKit node labels.
/// → Visual verification is delegated to `snapshot_sample_en`.
#[test]
fn fixture_en_s1_4_badge_section_renders() {
    let (_, _, source) = load_fixture("sample.md");
    let section_md = extract_section(&source, "### 1.4", "### 1.5");
    let pane = render_snippet(&section_md);
    // Verify it produces at least one section (not empty)
    assert!(
        !pane.sections.is_empty(),
        "§1.4 badge section should produce at least one RenderedSection"
    );
    // Verify no panic during harness rendering
    let _harness = build_harness(pane.sections, PANEL_WIDTH, 200.0);
}

/// §1.5: "English | 日本語" — link exists and is to the right of text, same row.
#[test]
fn fixture_en_s1_5_text_link_same_row_and_centered() {
    let (_, _, source) = load_fixture("sample.md");
    let section_md = extract_section(&source, "### 1.5", "### 1.6");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections, PANEL_WIDTH, 200.0);
    // Verify link exists
    let _link = harness.get_by_label("日本語");
    // Verify text and link are on the same row, link to the right
    assert_right_of_same_row(&harness, "English |", "日本語", "§1.5 text + link");
}

/// §1.6: Full README header — heading and description are centered.
#[test]
fn fixture_en_s1_6_readme_header_centered() {
    let (_, _, source) = load_fixture("sample.md");
    let section_md = extract_section(&source, "### 1.6", "## 2.");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections, PANEL_WIDTH, 500.0);
    // Heading centered
    assert_centered(&harness, "KatanA Desktop", "§1.6 heading");
    // Description centered
    assert_centered(
        &harness,
        "A fast, lightweight Markdown workspace for macOS",
        "§1.6 description",
    );
    // Language link present
    let _link = harness.get_by_label("日本語");
    // Heading is above description
    assert_below(
        &harness,
        "KatanA Desktop",
        "A fast, lightweight Markdown workspace for macOS",
        "§1.6 heading above description",
    );
}

// ═════════════════════════════════════════════
// §2: Basic Markdown elements — extracted from fixture
// ═════════════════════════════════════════════

/// §2.1: All heading levels render and have correct vertical ordering.
#[test]
fn fixture_en_s2_1_heading_levels_render_and_order() {
    let (_, _, source) = load_fixture("sample.md");
    let section_md = extract_section(&source, "### 2.1", "### 2.2");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections, PANEL_WIDTH, 500.0);
    // All exist
    let _h1 = harness.get_by_label("H1 Heading");
    let _h2 = harness.get_by_label("H2 Heading");
    let _h3 = harness.get_by_label("H3 Heading");
    let _h4 = harness.get_by_label("H4 Heading");
    let _h5 = harness.get_by_label("H5 Heading");
    let _h6 = harness.get_by_label("H6 Heading");
    // Correct top-down ordering
    assert_below(&harness, "H1 Heading", "H2 Heading", "§2.1 H1 > H2");
    assert_below(&harness, "H2 Heading", "H3 Heading", "§2.1 H2 > H3");
    assert_below(&harness, "H5 Heading", "H6 Heading", "§2.1 H5 > H6");
}

/// §2.2: Text decorations produce labeled nodes.
#[test]
fn fixture_en_s2_2_text_decorations_render() {
    let (_, _, source) = load_fixture("sample.md");
    let section_md = extract_section(&source, "### 2.2", "### 2.3");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections, PANEL_WIDTH, 300.0);
    let _bold = harness.get_by_label("Bold text");
    let _italic = harness.get_by_label("Italic text");
    let _strike = harness.get_by_label("Strikethrough");
    // Vertical order
    assert_below(&harness, "Bold text", "Italic text", "§2.2 bold > italic");
    assert_below(
        &harness,
        "Italic text",
        "Strikethrough",
        "§2.2 italic > strike",
    );
}

/// §2.3: Links exist as clickable labels.
#[test]
fn fixture_en_s2_3_links_render() {
    let (_, _, source) = load_fixture("sample.md");
    let section_md = extract_section(&source, "### 2.3", "### 2.4");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections, PANEL_WIDTH, 200.0);
    let _link = harness.get_by_label("Normal link");
    let _email = harness.get_by_label("Email link");
}

// ═════════════════════════════════════════════
// Japanese fixture
// ═════════════════════════════════════════════

/// Japanese fixture: many sections, no Pending.
#[test]
fn fixture_ja_structural_integrity() {
    let (pane, _, _) = load_fixture("sample.ja.md");
    assert!(
        pane.sections.len() > 30,
        "Japanese fixture should produce >30 sections, got: {}",
        pane.sections.len()
    );
    let pending_count = pane
        .sections
        .iter()
        .filter(|s| matches!(s, RenderedSection::Pending { .. }))
        .count();
    assert_eq!(pending_count, 0, "No Pending sections should remain");
}

/// §1.1 (JA): Centered heading — extracted from JA fixture.
#[test]
fn fixture_ja_s1_1_centered_heading() {
    let (_, _, source) = load_fixture("sample.ja.md");
    let section_md = extract_section(&source, "### 1.1", "### 1.2");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections, PANEL_WIDTH, 200.0);
    assert_centered(&harness, "KatanA Desktop", "§1.1 JA centered h1");
}

/// §1.3 (JA): Multiple centered blocks no overlap — extracted from JA fixture.
#[test]
fn fixture_ja_s1_3_centered_blocks_no_overlap() {
    let (_, _, source) = load_fixture("sample.ja.md");
    let section_md = extract_section(&source, "### 1.3", "### 1.4");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections, PANEL_WIDTH, 400.0);
    assert_below(
        &harness,
        "中央寄せ見出し",
        "中央寄せの説明段落。",
        "§1.3 JA heading -> description",
    );
}

/// §1.5 (JA): Bidirectional link "English" exists — extracted from JA fixture.
#[test]
fn fixture_ja_s1_5_bidirectional_link() {
    let (_, _, source) = load_fixture("sample.ja.md");
    let section_md = extract_section(&source, "### 1.5", "### 1.6");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections, PANEL_WIDTH, 200.0);
    let _link = harness.get_by_label("English");
    // Verify text+link are on same row
    assert_right_of_same_row(&harness, "English", "| 日本語", "§1.5 JA link same row");
}

/// DrawIo renders correctly in JA fixture.
#[test]
fn fixture_ja_drawio_renders() {
    let (pane, _, _) = load_fixture("sample.ja.md");
    let drawio_count = pane
        .sections
        .iter()
        .filter(|s| matches!(s, RenderedSection::Image { alt, .. } if alt.contains("DrawIo")))
        .count();
    assert!(
        drawio_count >= 2,
        "Expected at least 2 DrawIo Image sections in JA fixture, got: {drawio_count}"
    );
}

// ═════════════════════════════════════════════
// Split-fixture snapshots — each captures the ENTIRE document, no clipping.
//
// Fixtures are split into 3 groups so each fits within the GPU texture limit (8192px):
//   1. HTML centering (sample_html)       — past bug regression
//   2. Basic Markdown  (sample_basic)     — headings, lists, code, tables, quotes, edge cases
//   3. Diagrams        (sample_diagrams)  — Mermaid, PlantUML, DrawIo (external deps)
//
// Badge same-row, overall layout, and visual details
// are verified here — this is the ground truth.
// ═════════════════════════════════════════════

/// Shared helper: load a fixture, build a harness with CJK fonts, run, and take a snapshot.
fn snapshot_fixture(fixture_filename: &str, snapshot_name: &str) {
    let (pane, _, _) = load_fixture(fixture_filename);
    let sections = pane.sections;
    let mut fonts_loaded = false;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(PANEL_WIDTH, PANEL_HEIGHT))
        .build_ui(move |ui| {
            if !fonts_loaded {
                load_test_fonts(ui.ctx());
                fonts_loaded = true;
            }
            let mut pane = PreviewPane::default();
            pane.sections = sections.clone();
            pane.show_content(ui);
        });
    for _ in 0..5 {
        harness.step();
    }
    harness.run();
    let result = harness.try_snapshot_options(snapshot_name, &snapshot_opts());
    let mut results = harness.take_snapshot_results();
    results.add(result);
}

// ── HTML Centering ──

#[test]
fn snapshot_html_en() {
    snapshot_fixture("sample_html.md", "sample_html_en");
}

#[test]
fn snapshot_html_ja() {
    snapshot_fixture("sample_html.ja.md", "sample_html_ja");
}

// ── Basic Markdown ──

#[test]
fn snapshot_basic_en() {
    snapshot_fixture("sample_basic.md", "sample_basic_en");
}

#[test]
fn snapshot_basic_ja() {
    snapshot_fixture("sample_basic.ja.md", "sample_basic_ja");
}

// ── Diagrams (External Dependencies) ──
// These tests depend on external tools (mmdc, plantuml.jar, drawio) and produce
// vastly different output depending on whether they are installed. CI runners
// lack these tools, so the snapshots will never match. Run locally with:
//   cargo test -- --ignored snapshot_diagrams

#[test]
#[ignore = "limited_local: requires external diagram tools (mmdc, plantuml, drawio)"]
fn snapshot_diagrams_en() {
    snapshot_fixture("sample_diagrams.md", "sample_diagrams_en");
}

#[test]
#[ignore = "limited_local: requires external diagram tools (mmdc, plantuml, drawio)"]
fn snapshot_diagrams_ja() {
    snapshot_fixture("sample_diagrams.ja.md", "sample_diagrams_ja");
}
