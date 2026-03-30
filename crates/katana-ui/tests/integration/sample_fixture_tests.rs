use eframe::egui;
use egui_kittest::kittest::{NodeT, Queryable};
use egui_kittest::Harness;
use katana_ui::preview_pane::{PreviewPane, RenderedSection};
use std::path::Path;

const PANEL_WIDTH: f32 = 800.0;
const PANEL_HEIGHT: f32 = 8000.0;
const CENTERING_TOLERANCE: f64 = 50.0;

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

fn extract_section(source: &str, start_marker: &str, end_marker: &str) -> String {
    let start_pos = source
        .find(start_marker)
        .unwrap_or_else(|| panic!("Marker not found: '{start_marker}'"));
    let after_start = start_pos + start_marker.len();
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

fn render_snippet(md: &str) -> PreviewPane {
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(md, Path::new("/tmp/snippet.md"));
    pane
}

fn load_test_fonts(ctx: &egui::Context) {
    use std::sync::Arc;

    let mut fonts = egui::FontDefinitions::default();

    let prop_candidates = [
        "/System/Library/Fonts/\u{30d2}\u{30e9}\u{30ae}\u{30ce}\u{89d2}\u{30b4}\u{30b7}\u{30c3}\u{30af} W3.ttc",
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

    let y_diff = (bounds_a.y0 - bounds_b.y0).abs();
    assert!(
        y_diff < 5.0,
        "[{context}] '{label_left}' (Y={:.1}) and '{label_right}' (Y={:.1}) should be on same row, diff={y_diff:.1}",
        bounds_a.y0, bounds_b.y0
    );
    assert!(
        bounds_b.x0 > bounds_a.x0,
        "[{context}] '{label_right}' X ({:.1}) should be right of '{label_left}' X ({:.1})",
        bounds_b.x0,
        bounds_a.x0
    );
}

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

#[test]
fn fixture_en_produces_many_sections() {
    let (pane, _, _) = load_fixture("sample.md");
    assert!(
        pane.sections.len() > 25,
        "English fixture should produce >25 sections, got: {}",
        pane.sections.len()
    );
}

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

#[test]
fn fixture_en_drawio_always_renders_to_image() {
    let (pane, _, _) = load_fixture("sample.md");
    let drawio_image_count = pane
        .sections
        .iter()
        .filter(|s| matches!(s, RenderedSection::Image { alt, .. } if alt.contains("DrawIo")))
        .count();
    assert!(
        drawio_image_count >= 2,
        "Expected at least 2 DrawIo Image sections, got: {drawio_image_count}"
    );
}

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
    assert!(
        mermaid_count >= 5,
        "Expected at least 5 Mermaid sections (Image or CommandNotFound), got: {mermaid_count}"
    );
}

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
    assert!(
        plantuml_count >= 3,
        "Expected at least 3 PlantUML sections (Image or NotInstalled), got: {plantuml_count}"
    );
}

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

#[test]
fn fixture_en_s1_1_centered_heading_h1() {
    let (_, _, source) = load_fixture("sample.md");
    let section_md = extract_section(&source, "### 1.1", "### 1.2");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 200.0);
    assert_centered(&harness, "KatanA Desktop", "§1.1 centered h1");
}

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

#[test]
fn fixture_en_s1_4_badge_section_renders() {
    let (_, _, source) = load_fixture("sample.md");
    let section_md = extract_section(&source, "### 1.4", "### 1.5");
    let pane = render_snippet(&section_md);
    assert!(
        !pane.sections.is_empty(),
        "§1.4 badge section should produce at least one RenderedSection"
    );
    let _harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 200.0);
}

#[test]
fn fixture_en_s1_5_text_link_same_row_and_centered() {
    let (_, _, source) = load_fixture("sample.md");
    let section_md = extract_section(&source, "### 1.5", "### 1.6");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 200.0);
    let _link = harness.get_by_label("\u{65e5}\u{672c}\u{8a9e}");
    assert_right_of_same_row(
        &harness,
        "English |",
        "\u{65e5}\u{672c}\u{8a9e}",
        "§1.5 text + link",
    );
}

#[test]
fn fixture_en_s1_6_readme_header_centered() {
    let (_, _, source) = load_fixture("sample.md");
    let section_md = extract_section(&source, "### 1.6", "## 2.");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 500.0);
    assert_centered(&harness, "KatanA Desktop", "§1.6 heading");
    assert_centered(
        &harness,
        "A fast, lightweight Markdown workspace for macOS",
        "§1.6 description",
    );
    let _link = harness.get_by_label("\u{65e5}\u{672c}\u{8a9e}");
    assert_below(
        &harness,
        "KatanA Desktop",
        "A fast, lightweight Markdown workspace for macOS",
        "§1.6 heading above description",
    );
}

#[test]
fn fixture_en_s2_1_heading_levels_render_and_order() {
    let (_, _, source) = load_fixture("sample.md");
    let section_md = extract_section(&source, "### 2.1", "### 2.2");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 500.0);
    let _h1 = harness.get_by_label("H1 Heading");
    let _h2 = harness.get_by_label("H2 Heading");
    let _h3 = harness.get_by_label("H3 Heading");
    let _h4 = harness.get_by_label("H4 Heading");
    let _h5 = harness.get_by_label("H5 Heading");
    let _h6 = harness.get_by_label("H6 Heading");
    assert_below(&harness, "H1 Heading", "H2 Heading", "§2.1 H1 > H2");
    assert_below(&harness, "H2 Heading", "H3 Heading", "§2.1 H2 > H3");
    assert_below(&harness, "H5 Heading", "H6 Heading", "§2.1 H5 > H6");
}

#[test]
fn fixture_en_s2_2_text_decorations_render() {
    let (_, _, source) = load_fixture("sample.md");
    let section_md = extract_section(&source, "### 2.2", "### 2.3");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 300.0);
    let _bold = harness.get_by_label("Bold text");
    let _italic = harness.get_by_label("Italic text");
    let _strike = harness.get_by_label("Strikethrough");
    assert_below(&harness, "Bold text", "Italic text", "§2.2 bold > italic");
    assert_below(
        &harness,
        "Italic text",
        "Strikethrough",
        "§2.2 italic > strike",
    );
}

#[test]
fn fixture_en_s2_3_links_render() {
    let (_, _, source) = load_fixture("sample.md");
    let section_md = extract_section(&source, "### 2.3", "### 2.4");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 200.0);
    let _link = harness.get_by_label("Normal link");
    let _email = harness.get_by_label("Email link");
}

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

#[test]
fn fixture_ja_s1_1_centered_heading() {
    let (_, _, source) = load_fixture("sample.ja.md");
    let section_md = extract_section(&source, "### 1.1", "### 1.2");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 200.0);
    assert_centered(&harness, "KatanA Desktop", "§1.1 JA centered h1");
}

#[test]
fn fixture_ja_s1_3_centered_blocks_no_overlap() {
    let (_, _, source) = load_fixture("sample.ja.md");
    let section_md = extract_section(&source, "### 1.3", "### 1.4");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 400.0);
    assert_below(
        &harness,
        "\u{4e2d}\u{592e}\u{5bc4}\u{305b}\u{898b}\u{51fa}\u{3057}",
        "\u{4e2d}\u{592e}\u{5bc4}\u{305b}\u{306e}\u{8aac}\u{660e}\u{6bb5}\u{843d}\u{3002}",
        "§1.3 JA heading -> description",
    );
}

#[test]
fn fixture_ja_s1_5_bidirectional_link() {
    let (_, _, source) = load_fixture("sample.ja.md");
    let section_md = extract_section(&source, "### 1.5", "### 1.6");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 200.0);
    let _link = harness.get_by_label("English");
    assert_right_of_same_row(
        &harness,
        "English",
        "| \u{65e5}\u{672c}\u{8a9e}",
        "§1.5 JA link same row",
    );
}

#[test]
fn fixture_ja_s1_6_readme_header_has_block_spacing() {
    let (_, _, source) = load_fixture("sample.ja.md");
    let section_md = extract_section(&source, "### 1.6", "## 2.");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 500.0);
    assert_gap_at_least(
        &harness,
        "KatanA Desktop",
        "\u{9ad8}\u{901f}\u{30fb}\u{8efd}\u{91cf}\u{306a} macOS \u{5411}\u{3051} Markdown \u{30ef}\u{30fc}\u{30af}\u{30b9}\u{30da}\u{30fc}\u{30b9}",
        7.0,
        "§1.6 JA top-level block spacing",
    );
}

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

fn load_fixture_harness(filename: &str) -> Harness<'static> {
    let (pane, _, _) = load_fixture(filename);
    build_harness(pane.sections.clone(), PANEL_WIDTH, PANEL_HEIGHT)
}

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
        "\u{4e2d}\u{592e}\u{5bc4}\u{305b}\u{898b}\u{51fa}\u{3057}",
        "sample_html_ja centered heading",
    );
    assert_centered(
        &harness,
        "2\u{3064}\u{76ee}\u{306e}\u{4e2d}\u{592e}\u{5bc4}\u{305b}\u{6bb5}\u{843d} — 1\u{3064}\u{76ee}\u{3068}\u{91cd}\u{306a}\u{3089}\u{306a}\u{3044}\u{3053}\u{3068}\u{3002}",
        "sample_html_ja centered paragraph",
    );
    assert_below(
        &harness,
        "\u{4e2d}\u{592e}\u{5bc4}\u{305b}\u{898b}\u{51fa}\u{3057}",
        "\u{3059}\u{3079}\u{3066}\u{306e}\u{30bb}\u{30af}\u{30b7}\u{30e7}\u{30f3}\u{304c}\u{6b63}\u{3057}\u{304f}\u{8868}\u{793a}\u{3055}\u{308c}\u{3066}\u{3044}\u{308c}\u{3070}\u{3001}HTML\u{30bb}\u{30f3}\u{30bf}\u{30ea}\u{30f3}\u{30b0}\u{306f}\u{6b63}\u{5e38}\u{3067}\u{3059}\u{3002}",
        "sample_html_ja full document order",
    );
}

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

#[test]
fn basic_fixture_en_s2_2_underline_renders_as_text() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "### 2.2", "### 2.3");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 300.0);
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

#[test]
fn basic_fixture_en_s2_2_highlight_renders_as_text() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "### 2.2", "### 2.3");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 300.0);
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
    let _h6 = harness.get_by_label("H6 \u{898b}\u{51fa}\u{3057}");
    let _strike = harness.get_by_label("\u{53d6}\u{308a}\u{6d88}\u{3057}\u{7dda}");
    let _link = harness.get_by_label("\u{901a}\u{5e38}\u{306e}\u{30ea}\u{30f3}\u{30af}");
    let _task = harness.get_by_label("\u{5b8c}\u{4e86}\u{30bf}\u{30b9}\u{30af}");
    let _quote = harness.get_by_label("\u{5916}\u{5074}\u{306e}\u{5f15}\u{7528}");
    let _arch = harness
        .get_by_label("\u{30a2}\u{30fc}\u{30ad}\u{30c6}\u{30af}\u{30c1}\u{30e3}\u{6982}\u{8981}");
    assert_below(
        &harness,
        "H6 \u{898b}\u{51fa}\u{3057}",
        "\u{30a2}\u{30fc}\u{30ad}\u{30c6}\u{30af}\u{30c1}\u{30e3}\u{6982}\u{8981}",
        "sample_basic_ja full document order",
    );
}

#[test]
fn basic_fixture_ja_s11_2_long_inline_code_wraps_within_panel() {
    let (_, _, source) = load_fixture("sample_basic.ja.md");
    let section_md = extract_section(&source, "### 13.2", "### 13.3");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), 420.0, 220.0);
    let node = harness.get_by_label_contains("\u{3053}\u{306e}\u{30c6}\u{30ad}\u{30b9}\u{30c8}\u{306f}\u{975e}\u{5e38}\u{306b}\u{9577}\u{3044}\u{884c}");
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

#[test]
fn basic_fixture_en_s12_accordion_renders_summary() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "## 7", "## 8");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 300.0);
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

#[test]
fn basic_fixture_en_s13_block_math_renders() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "### 8.1", "### 8.2");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 300.0);
    let node = harness.get_by_label("f(x) = \\int_{0}^{x} \\frac{t^2}{1 + t^4} \\, dt");
    assert!(
        !node.accesskit_node().is_hidden(),
        "Block math content should be rendered and visible"
    );
}

#[test]
fn basic_fixture_en_s13_inline_math_renders() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "### 8.2", "### 8.3");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 200.0);
    let node = harness.get_by_label("E = mc^2");
    assert!(
        !node.accesskit_node().is_hidden(),
        "Inline math node should not be hidden"
    );
}

#[test]
fn basic_fixture_en_s13_singleline_math_renders() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "### 8.3", "---");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 300.0);
    let node = harness.get_by_label("\\sum_{k=1}^{n} k = \\frac{n(n+1)}{2}");
    assert!(
        !node.accesskit_node().is_hidden(),
        "Single-line $$ math content should be rendered and visible"
    );
}

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

#[test]
fn basic_fixture_en_s12_accordion_open_list_items_inline() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "## 7", "## 8");
    let pane = render_snippet(&section_md);
    let harness = build_harness_accordion_open(pane.sections.clone());

    let swords_node = harness.get_by_label("Swords");
    let swords_bounds = swords_node
        .accesskit_node()
        .raw_bounds()
        .expect("'Swords' should have bounds after accordion opens");

    assert!(
        swords_bounds.x1 - swords_bounds.x0 > 5.0,
        "'Swords' text width should be > 5px to confirm it rendered inline, got {:.1}",
        swords_bounds.x1 - swords_bounds.x0
    );
}

#[test]
fn basic_fixture_en_s12_accordion_open_nested_list_indented() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "## 7", "## 8");
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

    assert!(
        m_bounds.y0 > s_bounds.y0,
        "'Muramasa' (Y={:.1}) should be below 'Swords' (Y={:.1})",
        m_bounds.y0,
        s_bounds.y0
    );
    assert!(
        m_bounds.x0 >= s_bounds.x0 + 5.0,
        "'Muramasa' X ({:.1}) should be ≥5px right of 'Swords' X ({:.1}). Nested indent broken!",
        m_bounds.x0,
        s_bounds.x0
    );
}

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

#[test]
fn basic_fixture_en_s12_accordion_open_bottom_margin_not_excessive() {
    let md = "\
<details><summary>Show details</summary><div>

- Last item

</div></details>

After open paragraph.
";
    let pane = render_snippet(md);
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
    assert!(
        gap <= 55.0,
        "Open accordion: gap from last item (y1={:.1}) to next para (y0={:.1}) = {:.1}px must be <=55px (task 4.3-open)",
        last_bounds.y1,
        after_bounds.y0,
        gap
    );
}

#[test]
fn basic_fixture_en_s12_accordion_icon_vertically_centered() {
    let md = "<details><summary>Show details</summary><div>\n\nContent\n\n</div></details>\n";
    let pane = render_snippet(md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 200.0);

    let summary = harness.get_by_label("Show details");
    let sb = summary
        .accesskit_node()
        .raw_bounds()
        .expect("'Show details' header must have bounds");

    let header_height = sb.y1 - sb.y0;

    assert!(
        header_height >= 10.0,
        "Header height {:.1}px < 10px (task 4.4)",
        header_height
    );

    assert!(
        header_height <= 24.0,
        "Header height {:.1}px > 24px — inflated improperly causing vertical misalignment (task 4.4)",
        header_height
    );

    assert!(
        sb.x0 > 0.0 && sb.x0 <= 30.0,
        "Icon slot width {:.1}px must be 0–30px (task 4.4)",
        sb.x0
    );
}

#[test]
fn basic_fixture_en_s11_4_footnote_reference_rendered() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "### 13.4", "### 13.5");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 400.0);
    let node = harness.get_by_label("[1]");
    let bounds = node
        .accesskit_node()
        .raw_bounds()
        .expect("'[1]' footnote reference should have bounds");
    assert!(
        bounds.x0 < PANEL_WIDTH as f64 * 0.8,
        "Footnote reference '[1]' x0={:.1} is too far right (panel={:.1}). Rendered off-screen?",
        bounds.x0,
        PANEL_WIDTH
    );
}

#[test]
fn basic_fixture_en_s11_4_footnote_definition_rendered() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "### 13.4", "### 13.5");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 400.0);
    let node = harness.get_by_label_contains("First footnote content.");
    let bounds = node
        .accesskit_node()
        .raw_bounds()
        .expect("Footnote definition text should have bounds");
    assert!(
        bounds.x0 < PANEL_WIDTH as f64 * 0.5,
        "Footnote text x0={:.1} should be in left half of panel (width={:.1}). Right-side overflow detected!",
        bounds.x0, PANEL_WIDTH
    );
    let text_width = bounds.x1 - bounds.x0;
    assert!(
        text_width > 50.0,
        "Footnote text width={:.1}px should be > 50px. Single-char vertical rendering detected!",
        text_width
    );
}

#[test]
fn basic_fixture_en_s11_4_footnote_return_link_rendered() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "### 13.4", "### 13.5");
    let pane = render_snippet(&section_md);
    let harness = build_harness(pane.sections.clone(), PANEL_WIDTH, 400.0);
    let nodes: Vec<_> = harness
        .query_all(egui_kittest::kittest::By::default().label("↩"))
        .collect();
    assert!(
        !nodes.is_empty(),
        "At least one return link '↩' should be rendered"
    );
}

#[test]
fn basic_fixture_en_s11_4_footnote_blocks_compact_spacing() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "### 13.4", "### 13.5");
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

#[test]
fn basic_fixture_en_s11_4_return_link_vertically_centered() {
    let (_, _, source) = load_fixture("sample_basic.md");
    let section_md = extract_section(&source, "### 13.4", "### 13.5");
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

    let fn_text = harness.get_by_label_contains("footnote body text");
    let bounds = fn_text
        .accesskit_node()
        .raw_bounds()
        .expect("Footnote body text should be visible after accordion");

    assert!(
        bounds.x0 <= PANEL_WIDTH as f64 * 0.2,
        "Regression: footnote x0={:.1} after accordion — too far right (panel={:.1}px, threshold=20%). Spacing leaked from accordion scope? (task 4.4 regression)",
        bounds.x0,
        PANEL_WIDTH
    );

    let text_width = bounds.x1 - bounds.x0;
    assert!(
        text_width >= 50.0,
        "Regression: footnote text width={:.1}px after accordion — text collapsed? (task 4.4 regression)",
        text_width
    );
}
