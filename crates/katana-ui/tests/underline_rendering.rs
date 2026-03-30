use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use katana_ui::preview_pane::PreviewPane;

#[test]
fn underline_tags_render_without_crash() {
    let mut pane = PreviewPane::default();

    pane.update_markdown_sections(
        "Here is some <u>underlined text</u> in the preview.",
        std::path::Path::new("/tmp/test.md"),
    );

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui, None, None);
    });
    harness.step();
    harness.run();

    let _text_node = harness.get_by_label_contains("underlined text");
}

#[test]
fn multiple_underlines_and_strikethroughs_in_same_block_render_safely() {
    let mut pane = PreviewPane::default();

    pane.update_markdown_sections(
        "A <u>custom underline</u> and a ~~strikethrough~~ mixed.",
        std::path::Path::new("/tmp/test.md"),
    );

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui, None, None);
    });
    harness.step();
    harness.run();

    let _text_block = harness.get_by_label_contains("custom underline");
}