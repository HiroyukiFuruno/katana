//! Draw.io (mxGraph) XML → SVG 変換レンダラー。
//!
//! MVP 対応範囲:
//! - 非圧縮の `<mxfile>` / `<mxGraphModel>` XML のみ受け付ける。
//! - `<mxCell>` の `vertex`（矩形・角丸矩形）と `edge`（直線矢印）を SVG に変換する。
//! - スタイルパースは最小限（`rounded`, `ellipse`, `label`, `fillColor`, `strokeColor`）。
//! - 非対応要素はスキップし、対応要素のみ描画する。

use xmltree::Element;

use super::diagram::{DiagramBlock, DiagramResult};

/// Draw.io XML を SVG HTML フラグメントに変換する。
pub fn render_drawio(block: &DiagramBlock) -> DiagramResult {
    match convert_xml_to_svg(&block.source) {
        Ok(svg) => DiagramResult::Ok(format!(r#"<div class="katana-diagram drawio">{svg}</div>"#)),
        Err(e) => DiagramResult::Err {
            source: block.source.clone(),
            error: e,
        },
    }
}

/// XML を解析して SVG 文字列を返す。
fn convert_xml_to_svg(xml: &str) -> Result<String, String> {
    let root = Element::parse(xml.as_bytes()).map_err(|e| format!("XML パースエラー: {e}"))?;
    let model = extract_graph_model(&root)?;
    let cells = collect_cells(model);
    let (w, h) = estimate_canvas_size(&cells);
    Ok(build_svg(&cells, w, h))
}

/// `<mxfile>` か `<mxGraphModel>` のどちらかから `<mxGraphModel>` 要素を返す。
fn extract_graph_model(root: &Element) -> Result<&Element, String> {
    if root.name == "mxGraphModel" {
        return Ok(root);
    }
    if root.name == "mxfile" {
        let diagram = root
            .get_child("diagram")
            .ok_or("<diagram> 要素が見つかりません")?;
        return diagram
            .get_child("mxGraphModel")
            .ok_or("<mxGraphModel> 要素が見つかりません".to_string());
    }
    Err(format!("サポート外のルート要素: {}", root.name))
}

/// `<root>` 以下の全 `<mxCell>` を収集する。
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

/// キャンバスサイズを推定する（全頂点の最大座標 + マージン）。
fn estimate_canvas_size(cells: &[&Element]) -> (f64, f64) {
    let (mut max_x, mut max_y) = (400.0_f64, 300.0_f64);
    for cell in cells {
        if let Some(geo) = cell.get_child("mxGeometry") {
            let x: f64 = attr_f64(geo, "x");
            let y: f64 = attr_f64(geo, "y");
            let w: f64 = attr_f64(geo, "width");
            let h: f64 = attr_f64(geo, "height");
            max_x = max_x.max(x + w + 20.0);
            max_y = max_y.max(y + h + 20.0);
        }
    }
    (max_x, max_y)
}

/// SVG 文書全体を組み立てる。
fn build_svg(cells: &[&Element], width: f64, height: f64) -> String {
    // セル ID → (x, y, w, h) のマップを構築する。
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
    let mut shapes = String::new();
    let mut labels = String::new();
    // SVG 矢印マーカーを定義する。
    shapes.push_str(
        "<defs><marker id=\"katana-arrow\" markerWidth=\"8\" markerHeight=\"6\" refX=\"8\" refY=\"3\" orient=\"auto\"><polygon points=\"0 0, 8 3, 0 6\" fill=\"#555555\"/></marker></defs>"
    );
    for cell in cells {
        render_cell(cell, &mut shapes, &mut labels, &geo_map);
    }
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">{shapes}{labels}</svg>"#
    )
}

/// 単一の `<mxCell>` を shapes/labels バッファに書き出す。
fn render_cell(
    cell: &Element,
    shapes: &mut String,
    labels: &mut String,
    geo_map: &std::collections::HashMap<String, (f64, f64, f64, f64)>,
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
        render_vertex(cell, shapes, labels);
    } else if is_edge {
        render_edge(cell, shapes, geo_map);
    }
}

/// 頂点の図形部分（rect/ellipse）を描画する。
fn render_shape(geo: &Element, style: &str, shapes: &mut String) {
    let x = attr_f64(geo, "x");
    let y = attr_f64(geo, "y");
    let w = attr_f64(geo, "width").max(1.0);
    let h = attr_f64(geo, "height").max(1.0);
    let fill = extract_style_value(style, "fillColor").unwrap_or("#fff2cc");
    let stroke = extract_style_value(style, "strokeColor").unwrap_or("#d6b656");
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

/// 頂点セルを SVG 図形 + ラベルとして描画する。
fn render_vertex(cell: &Element, shapes: &mut String, labels: &mut String) {
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
    render_shape(geo, style, shapes);
    append_label(cell, cx, cy, labels);
}

/// エッジセルを矢印付き折れ線（ポリライン）として描画する。
///
/// ソース・ターゲットの矩形ボーダー上の最近点を接続点とし、
/// `mxGeometry` 内の `Array` 要素に含まれる `mxPoint` 中間点も経由する。
fn render_edge(
    cell: &Element,
    shapes: &mut String,
    geo_map: &std::collections::HashMap<String, (f64, f64, f64, f64)>,
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

    // 各ボックスの中心を求める。
    let scx = sx + sw / 2.0;
    let scy = sy + sh / 2.0;
    let tcx = tx + tw / 2.0;
    let tcy = ty + th / 2.0;

    // mxGeometry 内の Array/mxPoint から中間ウェイポイントを収集する。
    let waypoints = collect_waypoints(cell);

    // 最初のウェイポイント（または最終ターゲット中心）への方向でソース接続点を算出する。
    let first_target = waypoints.first().copied().unwrap_or((tcx, tcy));
    let (x1, y1) = border_point(sx, sy, sw, sh, scx, scy, first_target.0, first_target.1);

    // 最後のウェイポイント（またはソース中心）からの方向でターゲット接続点を算出する。
    let last_source = waypoints.last().copied().unwrap_or((scx, scy));
    let (x2, y2) = border_point(tx, ty, tw, th, tcx, tcy, last_source.0, last_source.1);

    // ポリライン座標列を組み立てる。
    let mut points_str = format!("{x1:.1},{y1:.1}");
    for (wx, wy) in &waypoints {
        points_str.push_str(&format!(" {wx:.1},{wy:.1}"));
    }
    points_str.push_str(&format!(" {x2:.1},{y2:.1}"));

    shapes.push_str(&format!(
        "<polyline points=\"{points_str}\" fill=\"none\" stroke=\"#555555\" stroke-width=\"1.5\" marker-end=\"url(#katana-arrow)\"/>"
    ));

    // エッジラベルがあれば中間地点に描画する。
    if let Some(label) = cell.attributes.get("value") {
        if !label.is_empty() {
            let mid_x = (x1 + x2) / 2.0;
            let mid_y = (y1 + y2) / 2.0 - 6.0;
            const TEXT_COLOR: &str = "#333333";
            shapes.push_str(&format!(
                r#"<text x="{mid_x:.1}" y="{mid_y:.1}" text-anchor="middle" font-family="sans-serif" font-size="10" fill="{TEXT_COLOR}">{}</text>"#,
                xml_escape(label)
            ));
        }
    }
}

/// mxGeometry 内の Array > mxPoint 要素からウェイポイント座標を収集する。
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

/// 矩形のボーダー上で、(fx, fy) から (tx, ty) への方向ベクトルに沿った接続点を返す。
fn border_point(
    rx: f64,
    ry: f64,
    rw: f64,
    rh: f64,
    cx: f64,
    cy: f64,
    tx: f64,
    ty: f64,
) -> (f64, f64) {
    let dx = tx - cx;
    let dy = ty - cy;
    if dx.abs() < 0.001 && dy.abs() < 0.001 {
        return (cx, cy);
    }
    if dx.abs() * rh >= dy.abs() * rw {
        // 左右のボーダーに当たる
        if dx >= 0.0 {
            (rx + rw, cy + dy * (rw / 2.0) / dx.abs())
        } else {
            (rx, cy - dy * (rw / 2.0) / dx.abs())
        }
    } else {
        // 上下のボーダーに当たる
        if dy >= 0.0 {
            (cx + dx * (rh / 2.0) / dy.abs(), ry + rh)
        } else {
            (cx - dx * (rh / 2.0) / dy.abs(), ry)
        }
    }
}

/// セルのラベルテキストを SVG <text> として追加する。
fn append_label(cell: &Element, cx: f64, cy: f64, labels: &mut String) {
    let label = match cell.attributes.get("value") {
        Some(v) if !v.is_empty() => v.as_str(),
        _ => return,
    };
    // dy="0.35em" で垂直中央揃え（dominant-baseline は resvg が未サポートのため）。
    // fill を明示指定しないと resvg がテキストをスキップする場合がある。
    const TEXT_COLOR: &str = "#333333";
    labels.push_str(&format!(
        r#"<text x="{cx}" y="{cy}" dy="0.35em" text-anchor="middle" font-family="sans-serif" font-size="12" fill="{TEXT_COLOR}">{}</text>"#,
        xml_escape(label)
    ));
}

/// XML 属性を `f64` として取得する。存在しない場合は 0.0。
fn attr_f64(el: &Element, name: &str) -> f64 {
    el.attributes
        .get(name)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0)
}

/// mxGraph スタイル文字列から `key=value` を取り出す。
fn extract_style_value<'a>(style: &'a str, key: &str) -> Option<&'a str> {
    style.split(';').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        (k.trim() == key).then_some(v.trim())
    })
}

/// SVG テキストノード用の最小限 XML エスケープ。
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::diagram::{DiagramBlock, DiagramKind};

    const SIMPLE_DRAWIO_XML: &str = r#"<mxfile><diagram name="test"><mxGraphModel><root>
<mxCell id="0"/>
<mxCell id="1" parent="0"/>
<mxCell id="2" value="Box A" style="rounded=1;fillColor=#fff2cc;strokeColor=#d6b656;" vertex="1" parent="1">
    <mxGeometry x="80" y="80" width="120" height="60" as="geometry"/>
</mxCell>
<mxCell id="3" value="Box B" vertex="1" parent="1">
    <mxGeometry x="280" y="80" width="120" height="60" as="geometry"/>
</mxCell>
</root></mxGraphModel></diagram></mxfile>"#;

    #[test]
    fn 有効なdrawio_xmlがsvgに変換される() {
        let block = DiagramBlock {
            kind: DiagramKind::DrawIo,
            source: SIMPLE_DRAWIO_XML.to_string(),
        };
        let result = render_drawio(&block);
        assert!(matches!(result, DiagramResult::Ok(_)));
        if let DiagramResult::Ok(html) = result {
            assert!(html.contains("<svg"));
            assert!(html.contains("Box A"));
            assert!(html.contains("Box B"));
        }
    }

    #[test]
    fn 無効なxmlはエラー結果を返す() {
        let block = DiagramBlock {
            kind: DiagramKind::DrawIo,
            source: "not xml".to_string(),
        };
        let result = render_drawio(&block);
        assert!(matches!(result, DiagramResult::Err { .. }));
    }

    #[test]
    fn 楕円スタイルが処理される() {
        let xml = r#"<mxGraphModel><root>
<mxCell id="0"/><mxCell id="1" parent="0"/>
<mxCell id="2" value="Circle" style="ellipse;" vertex="1" parent="1">
    <mxGeometry x="50" y="50" width="100" height="100" as="geometry"/>
</mxCell>
</root></mxGraphModel>"#;
        let block = DiagramBlock {
            kind: DiagramKind::DrawIo,
            source: xml.to_string(),
        };
        let result = render_drawio(&block);
        if let DiagramResult::Ok(html) = result {
            assert!(html.contains("<ellipse"));
        }
    }

    #[test]
    fn spaceの抽出() {
        assert_eq!(
            extract_style_value("rounded=1;fillColor=#fff;", "fillColor"),
            Some("#fff")
        );
        assert_eq!(extract_style_value("rounded=1;", "fillColor"), None);
    }
}
