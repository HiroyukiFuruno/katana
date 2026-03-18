//! Tests for font size application via theme_bridge.
//!
//! Verifies that `apply_font_size` correctly updates egui text styles.

use katana_ui::theme_bridge::apply_font_size;

#[test]
fn apply_font_size_changes_body_text_style() {
    let ctx = eframe::egui::Context::default();
    apply_font_size(&ctx, 20.0);

    let style = ctx.style();
    let body_size = style
        .text_styles
        .get(&eframe::egui::TextStyle::Body)
        .expect("Body TextStyle should exist")
        .size;
    assert!(
        (body_size - 20.0).abs() < f32::EPSILON,
        "Body font size should be 20.0, got {body_size}"
    );
}

#[test]
fn apply_font_size_changes_monospace_text_style() {
    let ctx = eframe::egui::Context::default();
    apply_font_size(&ctx, 16.0);

    let style = ctx.style();
    let mono_size = style
        .text_styles
        .get(&eframe::egui::TextStyle::Monospace)
        .expect("Monospace TextStyle should exist")
        .size;
    assert!(
        (mono_size - 16.0).abs() < f32::EPSILON,
        "Monospace font size should be 16.0, got {mono_size}"
    );
}

#[test]
fn apply_font_size_preserves_heading_ratio() {
    let ctx = eframe::egui::Context::default();
    let base_size = 18.0;
    apply_font_size(&ctx, base_size);

    let style = ctx.style();
    let heading_size = style
        .text_styles
        .get(&eframe::egui::TextStyle::Heading)
        .expect("Heading TextStyle should exist")
        .size;
    // Heading is approximately 1.5× the base size
    let expected = base_size * 1.5;
    assert!(
        (heading_size - expected).abs() < 0.1,
        "Heading should be ~{expected}, got {heading_size}"
    );
}

#[test]
fn apply_font_size_updates_small_text_style() {
    let ctx = eframe::egui::Context::default();
    let base_size = 16.0;
    apply_font_size(&ctx, base_size);

    let style = ctx.style();
    let small_size = style
        .text_styles
        .get(&eframe::egui::TextStyle::Small)
        .expect("Small TextStyle should exist")
        .size;
    // Small is approximately 0.75× the base size
    let expected = base_size * 0.75;
    assert!(
        (small_size - expected).abs() < 0.1,
        "Small should be ~{expected}, got {small_size}"
    );
}

#[test]
fn apply_font_size_updates_button_text_style() {
    let ctx = eframe::egui::Context::default();
    apply_font_size(&ctx, 14.0);

    let style = ctx.style();
    let button_size = style
        .text_styles
        .get(&eframe::egui::TextStyle::Button)
        .expect("Button TextStyle should exist")
        .size;
    // Button is the same size as the base
    assert!(
        (button_size - 14.0).abs() < f32::EPSILON,
        "Button font size should be 14.0, got {button_size}"
    );
}
