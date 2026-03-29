use xmltree::Element;

use crate::markdown::color_preset::DiagramColorPreset;

use super::utils::{attr_f64, xml_escape, Rect};

/// Upward offset of edge labels from the baseline (px).
const EDGE_LABEL_VERTICAL_OFFSET: f64 = 6.0;

/// Minimum vector length threshold to prevent division by zero in `border_point()`.
const BORDER_POINT_EPSILON: f64 = 0.001;

/// Renders an edge cell as a polyline with an arrow.
///
/// Uses the nearest point on the source and target rectangle borders as connection points,
/// and also routes through `mxPoint` waypoints included in the `Array` element within `mxGeometry`.
#[allow(clippy::type_complexity)]
pub fn render_edge(
    cell: &Element,
    shapes: &mut String,
    geo_map: &[(String, (f64, f64, f64, f64))],
    preset: &DiagramColorPreset,
) {
    let Some((src, tgt)) = resolve_edge_rects(cell, geo_map) else {
        return;
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

fn resolve_edge_rects(
    cell: &Element,
    geo_map: &[(String, (f64, f64, f64, f64))],
) -> Option<(Rect, Rect)> {
    let src_id = cell.attributes.get("source")?;
    let tgt_id = cell.attributes.get("target")?;
    let (_, (sx, sy, sw, sh)) = geo_map.iter().find(|(k, _)| k == src_id)?;
    let (_, (tx, ty, tw, th)) = geo_map.iter().find(|(k, _)| k == tgt_id)?;
    Some((
        Rect {
            x: *sx,
            y: *sy,
            w: *sw,
            h: *sh,
        },
        Rect {
            x: *tx,
            y: *ty,
            w: *tw,
            h: *th,
        },
    ))
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
    let Some(geo) = cell.get_child("mxGeometry") else { return Vec::new(); };
    let mut points = Vec::new();
    for child in &geo.children {
        let Some(el) = child.as_element() else { continue; };
        if el.name != "Array" { continue; }
        
        for pt_node in &el.children {
            let Some(pt) = pt_node.as_element() else { continue; };
            if pt.name != "mxPoint" { continue; }
            let x = attr_f64(pt, "x");
            let y = attr_f64(pt, "y");
            points.push((x, y));
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
        // WHY: Hits the left or right border
        if dx >= 0.0 {
            (rect.x + rect.w, cy + dy * (rect.w / 2.0) / dx.abs())
        } else {
            (rect.x, cy - dy * (rect.w / 2.0) / dx.abs())
        }
    } else {
        // WHY: Hits the top or bottom border
        if dy >= 0.0 {
            (cx + dx * (rect.h / 2.0) / dy.abs(), rect.y + rect.h)
        } else {
            (cx - dx * (rect.h / 2.0) / dy.abs(), rect.y)
        }
    }
}
