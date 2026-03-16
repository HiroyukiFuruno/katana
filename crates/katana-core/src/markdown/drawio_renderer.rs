//! Draw.io (mxGraph) XML to SVG conversion renderer.
//!
//! MVP supported scope:
//! - Only accepts uncompressed `<mxfile>` / `<mxGraphModel>` XML.
//! - Converts `vertex` (rectangle/rounded rectangle) and `edge` (straight arrow) of `<mxCell>` to SVG.
//! - Minimal style parsing (`rounded`, `ellipse`, `label`, `fillColor`, `strokeColor`).
//! - Unsupported elements are skipped, rendering only supported ones.

use xmltree::Element;

use super::color_preset::DiagramColorPreset;
use super::diagram::{DiagramBlock, DiagramResult};

/// Minimum width of the Draw.io canvas (fallback when there are no elements).
const CANVAS_MIN_WIDTH: f64 = 400.0;

/// Minimum height of the Draw.io canvas (fallback when there are no elements).
const CANVAS_MIN_HEIGHT: f64 = 300.0;

/// Margin added from the edges of each element when estimating canvas size (px).
const CANVAS_EDGE_MARGIN: f64 = 20.0;

/// Upward offset of edge labels from the baseline (px).
const EDGE_LABEL_VERTICAL_OFFSET: f64 = 6.0;

/// Minimum vector length threshold to prevent division by zero in `border_point()`.
const BORDER_POINT_EPSILON: f64 = 0.001;

/// Converts Draw.io XML to an SVG HTML fragment.
pub fn render_drawio(block: &DiagramBlock) -> DiagramResult {
    match convert_xml_to_svg(&block.source) {
        Ok(svg) => DiagramResult::Ok(format!(r#"<div class="katana-diagram drawio">{svg}</div>"#)),
        Err(e) => DiagramResult::Err {
            source: block.source.clone(),
            error: e,
        },
    }
}

/// Parses XML and returns an SVG string.
fn convert_xml_to_svg(xml: &str) -> Result<String, String> {
    let root = Element::parse(xml.as_bytes()).map_err(|e| format!("XML parse error: {e}"))?;
    let model = extract_graph_model(&root)?;
    let cells = collect_cells(model);
    let (w, h) = estimate_canvas_size(&cells);
    Ok(build_svg(&cells, w, h))
}

/// Returns the `<mxGraphModel>` element from either `<mxfile>` or `<mxGraphModel>`.
fn extract_graph_model(root: &Element) -> Result<&Element, String> {
    if root.name == "mxGraphModel" {
        return Ok(root);
    }
    if root.name == "mxfile" {
        let diagram = root
            .get_child("diagram")
            .ok_or("<diagram> element not found")?;
        return diagram
            .get_child("mxGraphModel")
            .ok_or("<mxGraphModel> element not found".to_string());
    }
    Err(format!("Unsupported root element: {}", root.name))
}

/// Collects all `<mxCell>` elements under `<root>`.
fn collect_cells(model: &Element) -> Vec<&Element> {
    let root = match model.get_child("root") {
        Some(r) => r,
        None => return Vec::new(),
    };
    root.children
        .iter()
        .filter_map(|node| node.as_element())
        .filter(|el| el.name == "mxCell")
        .collect()
}

/// Estimates canvas size (maximum coordinates of all vertices + margin).
fn estimate_canvas_size(cells: &[&Element]) -> (f64, f64) {
    let (mut max_x, mut max_y) = (CANVAS_MIN_WIDTH, CANVAS_MIN_HEIGHT);
    for cell in cells {
        if let Some(geo) = cell.get_child("mxGeometry") {
            let x: f64 = attr_f64(geo, "x");
            let y: f64 = attr_f64(geo, "y");
            let w: f64 = attr_f64(geo, "width");
            let h: f64 = attr_f64(geo, "height");
            max_x = max_x.max(x + w + CANVAS_EDGE_MARGIN);
            max_y = max_y.max(y + h + CANVAS_EDGE_MARGIN);
        }
    }
    (max_x, max_y)
}

/// Assembles the entire SVG document.
fn build_svg(cells: &[&Element], width: f64, height: f64) -> String {
    // Builds a map of cell ID -> (x, y, w, h).
    let mut geo_map: std::collections::HashMap<String, (f64, f64, f64, f64)> =
        std::collections::HashMap::new();
    for cell in cells {
        if let (Some(id), Some(geo)) = (cell.attributes.get("id"), cell.get_child("mxGeometry")) {
            let is_vertex = cell
                .attributes
                .get("vertex")
                .map(|v| v == "1")
                .unwrap_or(false);
            if is_vertex {
                let x = attr_f64(geo, "x");
                let y = attr_f64(geo, "y");
                let w = attr_f64(geo, "width").max(1.0);
                let h = attr_f64(geo, "height").max(1.0);
                geo_map.insert(id.clone(), (x, y, w, h));
            }
        }
    }
    let preset = DiagramColorPreset::current();
    let mut shapes = String::new();
    let mut labels = String::new();
    // Defines the SVG arrow marker.
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

/// Writes a single `<mxCell>` to the shapes/labels buffers.
fn render_cell(
    cell: &Element,
    shapes: &mut String,
    labels: &mut String,
    geo_map: &std::collections::HashMap<String, (f64, f64, f64, f64)>,
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

/// Renders the shape part (rect/ellipse) of a vertex.
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

/// Renders a vertex cell as an SVG shape + label.
fn render_vertex(
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

/// Struct holding the position and size of a rectangle.
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Rect {
    fn center(&self) -> (f64, f64) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }
}

/// Renders an edge cell as a polyline with an arrow.
///
/// Uses the nearest point on the source and target rectangle borders as connection points,
/// and also routes through `mxPoint` waypoints included in the `Array` element within `mxGeometry`.
fn render_edge(
    cell: &Element,
    shapes: &mut String,
    geo_map: &std::collections::HashMap<String, (f64, f64, f64, f64)>,
    preset: &DiagramColorPreset,
) {
    let src_id = match cell.attributes.get("source") {
        Some(id) => id.as_str(),
        None => return,
    };
    let tgt_id = match cell.attributes.get("target") {
        Some(id) => id.as_str(),
        None => return,
    };
    let Some(&(sx, sy, sw, sh)) = geo_map.get(src_id) else {
        return;
    };
    let Some(&(tx, ty, tw, th)) = geo_map.get(tgt_id) else {
        return;
    };

    let src = Rect {
        x: sx,
        y: sy,
        w: sw,
        h: sh,
    };
    let tgt = Rect {
        x: tx,
        y: ty,
        w: tw,
        h: th,
    };
    let (scx, scy) = src.center();
    let (tcx, tcy) = tgt.center();
    let waypoints = collect_waypoints(cell);

    let first_target = waypoints.first().copied().unwrap_or((tcx, tcy));
    let (x1, y1) = border_point(&src, scx, scy, first_target.0, first_target.1);
    let last_source = waypoints.last().copied().unwrap_or((scx, scy));
    let (x2, y2) = border_point(&tgt, tcx, tcy, last_source.0, last_source.1);

    let points_str = build_polyline_points(x1, y1, &waypoints, x2, y2);
    shapes.push_str(&format!(
        "<polyline points=\"{points_str}\" fill=\"none\" stroke=\"{}\" stroke-width=\"1.5\" marker-end=\"url(#katana-arrow)\"/>",
        preset.arrow
    ));

    append_edge_label(cell, shapes, x1, y1, x2, y2, preset);
}

/// Assembles the polyline coordinate sequence.
fn build_polyline_points(x1: f64, y1: f64, waypoints: &[(f64, f64)], x2: f64, y2: f64) -> String {
    let mut s = format!("{x1:.1},{y1:.1}");
    for (wx, wy) in waypoints {
        s.push_str(&format!(" {wx:.1},{wy:.1}"));
    }
    s.push_str(&format!(" {x2:.1},{y2:.1}"));
    s
}

/// Renders the edge label at the midpoint, if it exists.
fn append_edge_label(
    cell: &Element,
    shapes: &mut String,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    preset: &DiagramColorPreset,
) {
    if let Some(label) = cell.attributes.get("value") {
        if !label.is_empty() {
            let mid_x = (x1 + x2) / 2.0;
            let mid_y = (y1 + y2) / 2.0 - EDGE_LABEL_VERTICAL_OFFSET;
            let text_color = preset.text;
            shapes.push_str(&format!(
                r#"<text x="{mid_x:.1}" y="{mid_y:.1}" text-anchor="middle" font-family="sans-serif" font-size="10" fill="{text_color}">{}</text>"#,
                xml_escape(label)
            ));
        }
    }
}

/// Collects waypoint coordinates from `Array` > `mxPoint` elements within `mxGeometry`.
fn collect_waypoints(cell: &Element) -> Vec<(f64, f64)> {
    let Some(geo) = cell.get_child("mxGeometry") else {
        return Vec::new();
    };
    let mut points = Vec::new();
    for child in &geo.children {
        let Some(el) = child.as_element() else {
            continue;
        };
        if el.name == "Array" {
            for pt_node in &el.children {
                let Some(pt) = pt_node.as_element() else {
                    continue;
                };
                if pt.name == "mxPoint" {
                    let x = attr_f64(pt, "x");
                    let y = attr_f64(pt, "y");
                    points.push((x, y));
                }
            }
        }
    }
    points
}

/// Returns the connection point on the rectangle's border along the direction vector from `(cx, cy)` to `(tx, ty)`.
fn border_point(rect: &Rect, cx: f64, cy: f64, tx: f64, ty: f64) -> (f64, f64) {
    let dx = tx - cx;
    let dy = ty - cy;
    if dx.abs() < BORDER_POINT_EPSILON && dy.abs() < BORDER_POINT_EPSILON {
        return (cx, cy);
    }
    if dx.abs() * rect.h >= dy.abs() * rect.w {
        // Hits the left or right border
        if dx >= 0.0 {
            (rect.x + rect.w, cy + dy * (rect.w / 2.0) / dx.abs())
        } else {
            (rect.x, cy - dy * (rect.w / 2.0) / dx.abs())
        }
    } else {
        // Hits the top or bottom border
        if dy >= 0.0 {
            (cx + dx * (rect.h / 2.0) / dy.abs(), rect.y + rect.h)
        } else {
            (cx - dx * (rect.h / 2.0) / dy.abs(), rect.y)
        }
    }
}

/// Adds the cell's label text as an SVG `<text>` element.
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
    // Vertical centering with `dy="0.35em"` (because `dominant-baseline` is not supported by resvg).
    // If `fill` is not explicitly specified, resvg may skip the text.
    let text_color = preset.text;
    labels.push_str(&format!(
        r#"<text x="{cx}" y="{cy}" dy="0.35em" text-anchor="middle" font-family="sans-serif" font-size="12" fill="{text_color}">{}</text>"#,
        xml_escape(label)
    ));
}

/// Gets an XML attribute as `f64`. Returns 0.0 if it doesn't exist.
fn attr_f64(el: &Element, name: &str) -> f64 {
    el.attributes
        .get(name)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0)
}

/// Extracts `key=value` from an mxGraph style string.
fn extract_style_value<'a>(style: &'a str, key: &str) -> Option<&'a str> {
    style.split(';').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        (k.trim() == key).then_some(v.trim())
    })
}

/// Minimal XML escape for SVG text nodes.
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
