use xmltree::Element;

use crate::markdown::color_preset::DiagramColorPreset;

use super::{edge::render_edge, utils::attr_f64, vertex::render_vertex};

/// Assembles the entire SVG document.
pub fn build_svg(cells: &[&Element], width: f64, height: f64) -> String {
    // WHY: Builds a map of cell ID -> (x, y, w, h).
    let geo_map = build_geo_map(cells);
    let preset = DiagramColorPreset::current();
    let mut shapes = String::new();
    let mut labels = String::new();
    // WHY: Defines the SVG arrow marker.
    shapes.push_str(&format!(
        "<defs><marker id=\"katana-arrow\" markerWidth=\"8\" markerHeight=\"6\" refX=\"8\" refY=\"3\" orient=\"auto\"><polygon points=\"0 0, 8 3, 0 6\" fill=\"{}\"/></marker></defs>",
        preset.arrow
    ));
    for cell in cells {
        render_cell(cell, &mut shapes, &mut labels, &geo_map, preset);
    }
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">{shapes}{labels}</svg>"#
    )
}

fn build_geo_map(cells: &[&Element]) -> Vec<(String, (f64, f64, f64, f64))> {
    let mut geo_map = Vec::new();
    for cell in cells {
        if let (Some(id), Some(geo)) = (cell.attributes.get("id"), cell.get_child("mxGeometry")) {
            let is_vertex = cell.attributes.get("vertex").map_or(false, |v| v == "1");
            if is_vertex {
                let x = attr_f64(geo, "x");
                let y = attr_f64(geo, "y");
                let w = attr_f64(geo, "width").max(1.0);
                let h = attr_f64(geo, "height").max(1.0);
                geo_map.push((id.clone(), (x, y, w, h)));
            }
        }
    }
    geo_map
}

/// Writes a single `<mxCell>` to the shapes/labels buffers.
pub fn render_cell(
    cell: &Element,
    shapes: &mut String,
    labels: &mut String,
    geo_map: &[(String, (f64, f64, f64, f64))],
    preset: &DiagramColorPreset,
) {
    let is_vertex = cell
        .attributes
        .get("vertex")
        .map(|v| v == "1")
        .unwrap_or(false);
    let is_edge = cell
        .attributes
        .get("edge")
        .map(|v| v == "1")
        .unwrap_or(false);
    if is_vertex {
        render_vertex(cell, shapes, labels, preset);
    } else if is_edge {
        render_edge(cell, shapes, geo_map, preset);
    }
}
