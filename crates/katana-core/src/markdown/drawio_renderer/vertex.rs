use xmltree::Element;

use crate::markdown::color_preset::DiagramColorPreset;

use super::utils::{attr_f64, extract_style_value, xml_escape};

pub fn render_vertex(
    cell: &Element,
    shapes: &mut String,
    labels: &mut String,
    preset: &DiagramColorPreset,
) {
    let Some(geo) = cell.get_child("mxGeometry") else {
        return;
    };
    let style = cell
        .attributes
        .get("style")
        .map(String::as_str)
        .unwrap_or("");
    let cx = attr_f64(geo, "x") + attr_f64(geo, "width").max(1.0) / 2.0;
    let cy = attr_f64(geo, "y") + attr_f64(geo, "height").max(1.0) / 2.0;
    render_shape(geo, style, shapes, preset);
    append_label(cell, cx, cy, labels, preset);
}

fn render_shape(geo: &Element, style: &str, shapes: &mut String, preset: &DiagramColorPreset) {
    let x = attr_f64(geo, "x");
    let y = attr_f64(geo, "y");
    let w = attr_f64(geo, "width").max(1.0);
    let h = attr_f64(geo, "height").max(1.0);
    let fill = extract_style_value(style, "fillColor").unwrap_or(preset.fill);
    let stroke = extract_style_value(style, "strokeColor").unwrap_or(preset.stroke);
    if style.contains("ellipse") {
        shapes.push_str(&format!(
            r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{fill}" stroke="{stroke}" stroke-width="1.5"/>"#,
            x + w / 2.0,
            y + h / 2.0,
            w / 2.0,
            h / 2.0
        ));
    } else {
        let rx = if style.contains("rounded=1") {
            "6"
        } else {
            "0"
        };
        shapes.push_str(&format!(
            r#"<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{rx}" fill="{fill}" stroke="{stroke}" stroke-width="1.5"/>"#
        ));
    }
}

fn append_label(
    cell: &Element,
    cx: f64,
    cy: f64,
    labels: &mut String,
    preset: &DiagramColorPreset,
) {
    let label = match cell.attributes.get("value") {
        Some(v) if !v.is_empty() => v.as_str(),
        _ => return,
    };
    let style = cell
        .attributes
        .get("style")
        .map(String::as_str)
        .unwrap_or("");
    let text_color = extract_style_value(style, "fontColor").unwrap_or(preset.drawio_label_color);
    labels.push_str(&format!(
        r#"<text x="{cx}" y="{cy}" dy="0.35em" text-anchor="middle" font-family="sans-serif" font-size="12" fill="{text_color}">{}</text>"#,
        xml_escape(label)
    ));
}
