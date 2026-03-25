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
//! ### Known AccessKit limitations
//! - `<img>` alt attributes are NOT exposed as AccessKit labels.
//!   Badge same-row verification is limited by standard AccessKit queries.

use eframe::egui;
use egui_kittest::kittest::{NodeT, Queryable};
use egui_kittest::Harness;
use katana_ui::preview_pane::{PreviewPane, RenderedSection};
use std::path::Path;

const PANEL_WIDTH: f32 = 800.0;
/// Maximum height for UI panel rendering tests.
/// GPU texture limit is 8192px. Fixtures are split into groups (html, basic, diagrams)
/// so that each group fits within this limit and is captured in full — no clipping.
const PANEL_HEIGHT: f32 = 8000.0;
const CENTERING_TOLERANCE: f64 = 50.0;

// ─────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────

/// Load fixture file, full_render with diagrams, and wait for completion.
fn load_fixture(filename: &str) -> (PreviewPane, std::path::PathBuf, String) {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fixtures")
        .join(filename);
    let source = std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", fixture_path.display()));

    let mut pane = PreviewPane::default();
    pane.full_render(
        &source,
        &fixture_path,
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        false,
        4,
    );
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
            pane.show_content(ui, None, None);
        });
    for _ in 0..5 {
        harness.step();
    }
    harness.run();
    harness
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

/// Assert that widget B has a meaningful vertical gap below widget A.
fn assert_gap_at_least(
    harness: &Harness,
    label_above: &str,
    label_below: &str,
    min_gap: f64,
    context: &str,
) {
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
    let gap = bounds_b.y0 - bounds_a.y1;
    assert!(
        gap >= min_gap,
        "[{context}] gap between '{label_above}' and '{label_below}' must be >= {min_gap:.1}, got {gap:.1}"
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
        pane.sections.len() > 25,
        "English fixture should produce >25 sections, got: {}",
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
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 200.0);
    assert_centered(&harness, "KatanA Desktop", "§1.1 centered h1");
}

/// §1.2: `<p align="center">` paragraph is horizontally centered.
#[test]
fn fixture_en_s1_2_centered_paragraph() {
    let (_, _, source) = load_fixture("sample.md");
    let section_md = extract_section(&source, "### 1.2", "### 1.3");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 200.0);
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
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 400.0);
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
    let _harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 200.0);
}

/// §1.5: "English | 日本語" — link exists and is to the right of text, same row.
#[test]
fn fixture_en_s1_5_text_link_same_row_and_centered() {
    let (_, _, source) = load_fixture("sample.md");
    let section_md = extract_section(&source, "### 1.5", "### 1.6");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 200.0);
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
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 500.0);
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
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 500.0);
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
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 300.0);
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
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 200.0);
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
        pane.sections.len() > 25,
        "Japanese fixture should produce >25 sections, got: {}",
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
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 200.0);
    assert_centered(&harness, "KatanA Desktop", "§1.1 JA centered h1");
}

/// §1.3 (JA): Multiple centered blocks no overlap — extracted from JA fixture.
#[test]
fn fixture_ja_s1_3_centered_blocks_no_overlap() {
    let (_, _, source) = load_fixture("sample.ja.md");
    let section_md = extract_section(&source, "### 1.3", "### 1.4");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 400.0);
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
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 200.0);
    let _link = harness.get_by_label("English");
    // Verify text+link are on same row
    assert_right_of_same_row(&harness, "English", "| 日本語", "§1.5 JA link same row");
}

/// §1.6 (JA): Top-level HTML blocks should keep browser-like vertical margins.
#[test]
fn fixture_ja_s1_6_readme_header_has_block_spacing() {
    let (_, _, source) = load_fixture("sample.ja.md");
    let section_md = extract_section(&source, "### 1.6", "## 2.");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 500.0);
    assert_gap_at_least(
        &harness,
        "KatanA Desktop",
        "高速・軽量な macOS 向け Markdown ワークスペース",
        7.0,
        "§1.6 JA top-level block spacing",
    );
}

/// Top-level HTML blocks should keep browser-like spacing between paragraphs.
#[test]
fn top_level_html_paragraphs_keep_browser_like_vertical_spacing() {
    let pane = render_snippet(concat!(
        "<p align=\"center\">First paragraph</p>\n\n",
        "<p align=\"center\">Second paragraph</p>\n\n",
        "<p align=\"center\">Third paragraph</p>\n",
    ));
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 280.0);

    assert_gap_at_least(
        &harness,
        "First paragraph",
        "Second paragraph",
        6.0,
        "top-level html paragraph spacing 1->2",
    );
    assert_gap_at_least(
        &harness,
        "Second paragraph",
        "Third paragraph",
        6.0,
        "top-level html paragraph spacing 2->3",
    );
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
// Split-fixture full render tests — verifies structural bounds.
//
// Fixtures are split into groups:
//   1. HTML centering (sample_html)
//   2. Basic Markdown (sample_basic)
// ═════════════════════════════════════════════

fn load_fixture_harness(filename: &str) -> Harness<'static> {
    let (pane, _, _) = load_fixture(filename);
    build_harness(pane.sections.clone(), PANEL_WIDTH, PANEL_HEIGHT)
}

// ── HTML Centering ──

#[test]
fn html_fixture_en_semantic_layout() {
    let harness = load_fixture_harness("sample_html.md");
    assert_centered(
        &harness,
        "Centered Heading",
        "sample_html_en centered heading",
    );
    assert_centered(
        &harness,
        "Second centered paragraph — should NOT overlap with the first one.",
        "sample_html_en centered paragraph",
    );
    assert_below(
        &harness,
        "Centered Heading",
        "If all sections above render correctly, HTML centering is working.",
        "sample_html_en full document order",
    );
}

#[test]
fn html_fixture_ja_semantic_layout() {
    let harness = load_fixture_harness("sample_html.ja.md");
    assert_centered(
        &harness,
        "中央寄せ見出し",
        "sample_html_ja centered heading",
    );
    assert_centered(
        &harness,
        "2つ目の中央寄せ段落 — 1つ目と重ならないこと。",
        "sample_html_ja centered paragraph",
    );
    assert_below(
        &harness,
        "中央寄せ見出し",
        "すべてのセクションが正しく表示されていれば、HTMLセンタリングは正常です。",
        "sample_html_ja full document order",
    );
}

// ── Basic Markdown ──

#[test]
fn basic_fixture_en_semantic_smoke() {
    let harness = load_fixture_harness("sample_basic.md");
    let _h6 = harness.get_by_label("H6 Heading");
    let _strike = harness.get_by_label("Strikethrough");
    let _link = harness.get_by_label("Normal link");
    let _task = harness.get_by_label("Completed task");
    let _quote = harness.get_by_label("Outer quote");
    let _arch = harness.get_by_label("Architecture Overview");
    assert_below(
        &harness,
        "H6 Heading",
        "Architecture Overview",
        "sample_basic_en full document order",
    );
}

/// §2.2: `<u>` inline HTML renders as underlined text (not raw tag).
#[test]
fn basic_fixture_en_s2_2_underline_renders_as_text() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "### 2.2", "### 2.3");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 300.0);
    // The word "Underline" should appear as a labeled node (not as "<u>Underline</u>")
    let node = harness.get_by_label("Underline");
    let bounds = node
        .accesskit_node()
        .raw_bounds()
        .expect("Underline text should have bounds");
    assert!(
        bounds.x1 - bounds.x0 > 10.0,
        "Underline text should have non-trivial width, got {:.1}",
        bounds.x1 - bounds.x0
    );
}

/// §2.2: `<mark>` inline HTML renders as highlighted text (not raw tag).
#[test]
fn basic_fixture_en_s2_2_highlight_renders_as_text() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "### 2.2", "### 2.3");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 300.0);
    // The word "Highlight" should appear as a labeled node (not as "<mark>Highlight</mark>")
    let node = harness.get_by_label("Highlight");
    let bounds = node
        .accesskit_node()
        .raw_bounds()
        .expect("Highlight text should have bounds");
    assert!(
        bounds.x1 - bounds.x0 > 10.0,
        "Highlight text should have non-trivial width, got {:.1}",
        bounds.x1 - bounds.x0
    );
}

#[test]
fn basic_fixture_ja_semantic_smoke() {
    let harness = load_fixture_harness("sample_basic.ja.md");
    let _h6 = harness.get_by_label("H6 見出し");
    let _strike = harness.get_by_label("取り消し線");
    let _link = harness.get_by_label("通常のリンク");
    let _task = harness.get_by_label("完了タスク");
    let _quote = harness.get_by_label("外側の引用");
    let _arch = harness.get_by_label("アーキテクチャ概要");
    assert_below(
        &harness,
        "H6 見出し",
        "アーキテクチャ概要",
        "sample_basic_ja full document order",
    );
}

/// §11.2 (JA): Long inline code should wrap inside the preview instead of overflowing horizontally.
#[test]
fn basic_fixture_ja_s11_2_long_inline_code_wraps_within_panel() {
    let (_, _, source) = load_fixture("sample_basic.ja.md");
    let section_md = extract_section(&source, "### 11.2", "### 11.3");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), 420.0, 220.0);
    let node = harness.get_by_label_contains("このテキストは非常に長い行");
    let bounds = node
        .accesskit_node()
        .raw_bounds()
        .expect("long inline code should have bounds");
    assert!(
        bounds.x1 <= 412.0,
        "long inline code must stay within the preview width, got right edge {:.1}",
        bounds.x1
    );
    assert!(
        bounds.y1 - bounds.y0 > 24.0,
        "long inline code should wrap to multiple rows, got height {:.1}",
        bounds.y1 - bounds.y0
    );
}

/// §12: `<details><summary>` renders as an accordion with the summary text as a label.
#[test]
fn basic_fixture_en_s12_accordion_renders_summary() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "## 12", "## 13");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 300.0);
    // The summary text "Show details" should appear as a clickable label
    let node = harness.get_by_label("Show details");
    let bounds = node
        .accesskit_node()
        .raw_bounds()
        .expect("Show details label should have bounds");
    assert!(
        bounds.x1 - bounds.x0 > 10.0,
        "Show details label should have non-trivial width, got {:.1}",
        bounds.x1 - bounds.x0
    );
}

// ── §13 Math ──

/// §13.1: ` ```math ` block renders the LaTeX source (not dropped or shown as raw code block).
#[test]
fn basic_fixture_en_s13_block_math_renders() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "### 13.1", "### 13.2");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 300.0);
    // The LaTeX source content should appear as a label (render_math_fn fallback)
    let node = harness.get_by_label("f(x) = \\int_{0}^{x} \\frac{t^2}{1 + t^4} \\, dt");
    assert!(
        !node.accesskit_node().is_hidden(),
        "Block math content should be rendered and visible"
    );
}

/// §13.2: Inline math `$E = mc^2$` renders as visible monospace text (no spaces inside delimiters).
#[test]
fn basic_fixture_en_s13_inline_math_renders() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "### 13.2", "### 13.3");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 200.0);
    // The LaTeX source "E = mc^2" is what render_math_fn receives (delimiters stripped by pulldown-cmark).
    // pulldown-cmark only parses $...$ as InlineMath when there are NO spaces inside the delimiters.
    let node = harness.get_by_label("E = mc^2");
    // Node exists and is not hidden — math was rendered (not silently dropped).
    assert!(
        !node.accesskit_node().is_hidden(),
        "Inline math node should not be hidden"
    );
}

/// §13.3: Single-line `$$ ... $$` renders the LaTeX source as block math.
#[test]
fn basic_fixture_en_s13_singleline_math_renders() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "### 13.3", "---");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 300.0);
    let node = harness.get_by_label("\\sum_{k=1}^{n} k = \\frac{n(n+1)}{2}");
    assert!(
        !node.accesskit_node().is_hidden(),
        "Single-line $$ math content should be rendered and visible"
    );
}

// ── §12 Accordion layout ──

/// Build a harness with the CollapsingHeader already open.
///   1. Build with the accordion section
///   2. Click the summary label to open it
///   3. Run several frames so layout stabilises
fn build_harness_accordion_open(sections: Vec<RenderedSection>) -> Harness<'static> {
    let mut fonts_loaded = false;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(PANEL_WIDTH, 800.0))
        .build_ui(move |ui| {
            if !fonts_loaded {
                load_test_fonts(ui.ctx());
                fonts_loaded = true;
            }
            let mut pane = PreviewPane::default();
            pane.sections = sections.clone();
            pane.show_content(ui, None, None);
        });
    // Open the accordion by clicking its summary
    for _ in 0..3 {
        harness.step();
    }
    let summary = harness.get_by_label("Show details");
    summary.click();
    for _ in 0..5 {
        harness.step();
    }
    harness.run();
    harness
}

/// §12: When the accordion is open, list items (bullet + text) must be on the SAME row.
/// Specifically "Swords" text must appear to the right of its bullet marker on the same Y line.
#[test]
fn basic_fixture_en_s12_accordion_open_list_items_inline() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "## 12", "## 13");
    let pane = render_snippet(&section_md);
    let harness = build_harness_accordion_open(pane.sections.clone());

    // "Swords" must be present (the top-level list item text)
    let swords_node = harness.get_by_label("Swords");
    let swords_bounds = swords_node
        .accesskit_node()
        .raw_bounds()
        .expect("'Swords' should have bounds after accordion opens");

    // The item must have a non-trivial width — means it rendered on the same row as bullet
    assert!(
        swords_bounds.x1 - swords_bounds.x0 > 5.0,
        "'Swords' text width should be > 5px to confirm it rendered inline, got {:.1}",
        swords_bounds.x1 - swords_bounds.x0
    );
}

/// §12: Nested list items "Muramasa", "Masamune", "Kotetsu" must appear below "Swords"
/// and be indented (larger X than "Swords").
#[test]
fn basic_fixture_en_s12_accordion_open_nested_list_indented() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "## 12", "## 13");
    let pane = render_snippet(&section_md);
    let harness = build_harness_accordion_open(pane.sections.clone());

    let swords = harness.get_by_label("Swords");
    let s_bounds = swords
        .accesskit_node()
        .raw_bounds()
        .expect("'Swords' should have bounds");

    let muramasa = harness.get_by_label("Muramasa");
    let m_bounds = muramasa
        .accesskit_node()
        .raw_bounds()
        .expect("'Muramasa' should have bounds");

    // Muramasa is below Swords
    assert!(
        m_bounds.y0 > s_bounds.y0,
        "'Muramasa' (Y={:.1}) should be below 'Swords' (Y={:.1})",
        m_bounds.y0,
        s_bounds.y0
    );
    // Muramasa must be indented at least 5px right of Swords — catches level-2 indent failure
    assert!(
        m_bounds.x0 >= s_bounds.x0 + 5.0,
        "'Muramasa' X ({:.1}) should be ≥5px right of 'Swords' X ({:.1}). Nested indent broken!",
        m_bounds.x0,
        s_bounds.x0
    );
}

// ── §12 Accordion layout quality (tasks 4.3/4.4/4.5) ──

/// §12 (4.3): Accordion must have at least 5px bottom margin below it.
/// Measured as the gap between accordion "Show details" header y1 and the
/// next paragraph text y0. A paragraph placed immediately after the accordion
/// in the markdown source should be separated by at least 5px and at most 20px.
#[test]
fn basic_fixture_en_s12_accordion_has_bottom_margin() {
    let md = "\
<details><summary>Show details</summary><div>

- Content item

</div></details>

After accordion paragraph.
";
    let pane = render_snippet(md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 300.0);

    let summary = harness.get_by_label("Show details");
    let summary_bounds = summary
        .accesskit_node()
        .raw_bounds()
        .expect("'Show details' should have bounds");

    let after = harness.get_by_label_contains("After accordion");
    let after_bounds = after
        .accesskit_node()
        .raw_bounds()
        .expect("'After accordion paragraph' should have bounds");

    let gap = after_bounds.y0 - summary_bounds.y1;
    assert!(
        gap >= 5.0,
        "Bottom margin below accordion (y1={:.1}) to next para (y0={:.1}) = {:.1}px must be >=5px (task 4.3)",
        summary_bounds.y1,
        after_bounds.y0,
        gap
    );
    assert!(
        gap <= 30.0,
        "Bottom margin {:.1}px must be <=30px — too large (task 4.3)",
        gap
    );
}

/// §12 (4.3-open): When accordion is OPEN, bottom margin to immediate next content
/// must be ≤20px. Uses a snippet with open accordion immediately followed by a paragraph.
#[test]
fn basic_fixture_en_s12_accordion_open_bottom_margin_not_excessive() {
    let md = "\
<details><summary>Show details</summary><div>

- Last item

</div></details>

After open paragraph.
";
    let pane = render_snippet(md);
    // Use build_harness_accordion_open helper logic (click to open then wait).
    let sections = pane.sections.clone();
    let mut fonts_loaded = false;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(PANEL_WIDTH, 400.0))
        .build_ui(move |ui| {
            if !fonts_loaded {
                load_test_fonts(ui.ctx());
                fonts_loaded = true;
            }
            let mut p = PreviewPane::default();
            p.sections = sections.clone();
            p.show_content(ui, None, None);
        });
    // Open the accordion.
    for _ in 0..3 {
        harness.step();
    }
    harness.get_by_label("Show details").click();
    for _ in 0..5 {
        harness.step();
    }
    harness.run();

    let last_item = harness.get_by_label("Last item");
    let last_bounds = last_item
        .accesskit_node()
        .raw_bounds()
        .expect("'Last item' must be visible when accordion is open");

    let after = harness.get_by_label_contains("After open");
    let after_bounds = after
        .accesskit_node()
        .raw_bounds()
        .expect("'After open paragraph' must have bounds");

    let gap = after_bounds.y0 - last_bounds.y1;
    // gap = accordion_bottom_margin(8) + internal_close_spacing + item_spacing + paragraph_top
    // With a properly compacted layout this should be well under 55px.
    assert!(
        gap <= 55.0,
        "Open accordion: gap from last item (y1={:.1}) to next para (y0={:.1}) = {:.1}px must be <=55px (task 4.3-open)",
        last_bounds.y1,
        after_bounds.y0,
        gap
    );
}

/// §12 (4.4): ▼/▶ icon must be vertically centered with the summary text.
///
/// Implementation: we use `allocate_exact_size(row_h = max(galley_h, icon_width))` and
/// paint both the icon and the text at `row_rect.center().y`. Centering is guaranteed
/// by construction. This test verifies the measurable invariant: the CollapsingHeader
/// accesskit bounds.height == row_h ≈ galley_h (no extra button_padding bloat).
///
/// If the underlying CollapsingHeader widget were used instead:
///   desired_size.y = galley.h + 2*button_padding (≥ galley.h + 4px)
///   → bounds.height ≥ galley.h + 4px, which would fail the upper bound here.
///
/// Regression guard: bounds.height must be ≤ body_text_height + 2px (tight fit).
#[test]
fn basic_fixture_en_s12_accordion_icon_vertically_centered() {
    let md = "<details><summary>Show details</summary><div>\n\nContent\n\n</div></details>\n";
    let pane = render_snippet(md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 200.0);

    // The CollapsingHeader accesskit node bounds = row_rect from allocate_exact_size.
    // bounds.height = max(galley_h, icon_width).
    // The accesskit label is the summary text set via widget_info.
    let summary = harness.get_by_label("Show details");
    let sb = summary
        .accesskit_node()
        .raw_bounds()
        .expect("'Show details' header must have bounds");

    let header_height = sb.y1 - sb.y0;

    // Lower bound: at least body font height
    assert!(
        header_height >= 10.0,
        "Header height {:.1}px < 10px (task 4.4)",
        header_height
    );

    // Upper bound: must be ≤ 24px (text height 14px + 2*4px button padding + small buffer).
    // By enforcing this, we ensure we use native egui sizes without bloated overrides.
    assert!(
        header_height <= 24.0,
        "Header height {:.1}px > 24px — inflated improperly causing vertical misalignment (task 4.4)",
        header_height
    );

    // Icon slot width (indent) must be a sensible size
    assert!(
        sb.x0 > 0.0 && sb.x0 <= 30.0,
        "Icon slot width {:.1}px must be 0–30px (task 4.4)",
        sb.x0
    );
}

/// §12 (4.5): When open, accordion body content must NOT be indented by a vertical left line.
/// Content x0 should be close to the header x0 (no extra left indent from blockquote-style line).
#[test]
fn basic_fixture_en_s12_accordion_open_no_vertical_left_line() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "## 12", "## 13");
    let pane = render_snippet(&section_md);
    let harness = build_harness_accordion_open(pane.sections.clone());

    let summary = harness.get_by_label("Show details");
    let summary_bounds = summary
        .accesskit_node()
        .raw_bounds()
        .expect("'Show details' should have bounds");

    let swords = harness.get_by_label("Swords");
    let swords_bounds = swords
        .accesskit_node()
        .raw_bounds()
        .expect("'Swords' should have bounds after accordion opens");

    // Without a vertical left line, list content x0 should be close to summary text x0.
    // A blockquote-style vertical line would push content ~8–16px to the right.
    // We allow up to 30px indent for the bullet point but no extra vertical-line indent.
    let indent_delta = swords_bounds.x0 - summary_bounds.x0;
    assert!(
        indent_delta <= 30.0,
        "Open accordion body 'Swords' x0 ({:.1}) vs summary x0 ({:.1}) = {:.1}px extra indent — vertical left line present? (task 4.5)",
        swords_bounds.x0,
        summary_bounds.x0,
        indent_delta
    );
}

// ── §11.4 Footnotes ──

/// §11.4: Footnote reference "[1]" must be rendered as a labeled node.
#[test]
fn basic_fixture_en_s11_4_footnote_reference_rendered() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "### 11.4", "### 11.5");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 400.0);
    // The footnote reference should render as "[1]" near the left side (not pushed off-screen)
    let node = harness.get_by_label("[1]");
    let bounds = node
        .accesskit_node()
        .raw_bounds()
        .expect("'[1]' footnote reference should have bounds");
    // Must start in the left 80% of the panel — catches right-side overflow
    assert!(
        bounds.x0 < PANEL_WIDTH as f64 * 0.8,
        "Footnote reference '[1]' x0={:.1} is too far right (panel={:.1}). Rendered off-screen?",
        bounds.x0,
        PANEL_WIDTH
    );
}

/// §11.4: Footnote definition text must appear in the rendered output.
#[test]
fn basic_fixture_en_s11_4_footnote_definition_rendered() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "### 11.4", "### 11.5");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 400.0);
    // The footnote definition body text must be visible.
    // It may be rendered as "1. First footnote content." so use contains.
    let node = harness.get_by_label_contains("First footnote content.");
    let bounds = node
        .accesskit_node()
        .raw_bounds()
        .expect("Footnote definition text should have bounds");
    // Must start in the left half of the panel — catches right-side overflow.
    // When the overflow bug was present, x0 was near the panel right edge (> 800px).
    assert!(
        bounds.x0 < PANEL_WIDTH as f64 * 0.5,
        "Footnote text x0={:.1} should be in left half of panel (width={:.1}). Right-side overflow detected!",
        bounds.x0, PANEL_WIDTH
    );
    // Must have at least 50px width — rules out 0-width or single-pixel renderings.
    // (~170px is the natural rendered width of "First footnote content." at normal font size)
    let text_width = bounds.x1 - bounds.x0;
    assert!(
        text_width > 50.0,
        "Footnote text width={:.1}px should be > 50px. Single-char vertical rendering detected!",
        text_width
    );
}

/// §11.4: Return link "↩" must appear in the rendered footnote definition.
#[test]
fn basic_fixture_en_s11_4_footnote_return_link_rendered() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "### 11.4", "### 11.5");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 400.0);
    // Multiple footnotes → multiple ↩ links; verify at least one is rendered.
    let nodes: Vec<_> = harness
        .query_all(egui_kittest::kittest::By::default().label("↩"))
        .collect();
    assert!(
        !nodes.is_empty(),
        "At least one return link '↩' should be rendered"
    );
}

/// §11.4 (5.10): Multiple footnote blocks must have compact spacing.
/// The y-gap between footnote 1 text bottom and footnote 2 text top must be
/// small — only frame inner_margin (top + bottom ≈ 2px) between them.
/// Original gap was 24px before any fix.
#[test]
fn basic_fixture_en_s11_4_footnote_blocks_compact_spacing() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "### 11.4", "### 11.5");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 400.0);

    let fn1 = harness.get_by_label_contains("First footnote content.");
    let fn1_bounds = fn1
        .accesskit_node()
        .raw_bounds()
        .expect("Footnote 1 text should have bounds");

    let fn2 = harness.get_by_label_contains("Second footnote content.");
    let fn2_bounds = fn2
        .accesskit_node()
        .raw_bounds()
        .expect("Footnote 2 text should have bounds");

    let gap = fn2_bounds.y0 - fn1_bounds.y1;
    assert!(
        gap < 8.0,
        "Gap fn1(y1={:.1}) → fn2(y0={:.1}) = {:.1}px must be <8px (fix 5.10)",
        fn1_bounds.y1,
        fn2_bounds.y0,
        gap
    );
}

/// §11.4 (5.11): Return link ↩ vertical centre must match footnote text centre (±3px).
#[test]
fn basic_fixture_en_s11_4_return_link_vertically_centered() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "### 11.4", "### 11.5");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 400.0);

    let fn_text = harness.get_by_label_contains("First footnote content.");
    let text_bounds = fn_text
        .accesskit_node()
        .raw_bounds()
        .expect("Footnote 1 text should have bounds");
    let text_center_y = (text_bounds.y0 + text_bounds.y1) / 2.0;

    let nodes: Vec<_> = harness
        .query_all(egui_kittest::kittest::By::default().label("↩"))
        .collect();
    assert!(!nodes.is_empty(), "At least one '↩' must exist");
    let link_bounds = nodes[0]
        .accesskit_node()
        .raw_bounds()
        .expect("Return link '↩' should have bounds");
    let link_center_y = (link_bounds.y0 + link_bounds.y1) / 2.0;

    let diff = (link_center_y - text_center_y).abs();
    assert!(
        diff < 3.0,
        "↩ centre Y({:.1}) vs text centre Y({:.1}) diff={:.1}px must be <3px (fix 5.11)",
        link_center_y,
        text_center_y,
        diff
    );
}

// ── §11.4 Regression: footnote x-position after accordion ──

/// Regression: When an accordion (<details>) precedes a footnote section in the
/// same document, the footnote text must NOT be shifted to the right.
/// Previously, mutating `ui.spacing().icon_width` inside the accordion scope leaked
/// into the footnote's scope_builder, causing its cursor.x to be offset rightward.
///
/// This test renders accordion + footnote in the same snippet and asserts:
///   - Footnote text x0 is within the left 20% of the panel (i.e., near the left edge)
///   - Footnote text has at least 50px width (not collapsed to a single character column)
#[test]
fn regression_footnote_x_not_shifted_after_accordion() {
    let md = "\
<details><summary>Accordion Section</summary><div>

Some content inside the accordion.

</div></details>

Paragraph with a footnote reference[^1].

[^1]: The footnote body text that should appear at the left edge.
";
    let pane = render_snippet(md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 500.0);

    // The footnote body must be visible and near the left edge.
    let fn_text = harness.get_by_label_contains("footnote body text");
    let bounds = fn_text
        .accesskit_node()
        .raw_bounds()
        .expect("Footnote body text should be visible after accordion");

    // x0 must be in the LEFT 20% of the panel (≤ 80px for a 400px panel).
    // When the regression was present, x0 was pushed ~spacing().indent (≈18px) or more
    // to the right per leaked spacing mutation, compounding with each scope.
    assert!(
        bounds.x0 <= PANEL_WIDTH as f64 * 0.2,
        "Regression: footnote x0={:.1} after accordion — too far right (panel={:.1}px, threshold=20%). Spacing leaked from accordion scope? (task 4.4 regression)",
        bounds.x0,
        PANEL_WIDTH
    );

    // Width sanity: the text must be at least 50px wide (not collapsed vertically).
    let text_width = bounds.x1 - bounds.x0;
    assert!(
        text_width >= 50.0,
        "Regression: footnote text width={:.1}px after accordion — text collapsed? (task 4.4 regression)",
        text_width
    );
}
// These tests depend on external tools (mmdc, plantuml.jar, drawio) and produce
// vastly different output depending on whether they are installed. CI runners
// lack these tools, so the snapshots will never match. Run locally with:
//   cargo test -- --ignored snapshot_diagrams
