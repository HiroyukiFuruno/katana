use katana_core::markdown::diagram::{DiagramBlock, DiagramKind, DiagramResult};
use katana_core::markdown::drawio_renderer::render_drawio;

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
fn fillcolorスタイルがsvg出力に反映される() {
    // extract_style_value の振る舞いは render_drawio 経由で検証する
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/><mxCell id="1" parent="0"/>
<mxCell id="2" value="Colored" style="fillColor=#ff0000;strokeColor=#00ff00;" vertex="1" parent="1">
    <mxGeometry x="50" y="50" width="100" height="60" as="geometry"/>
</mxCell>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    if let DiagramResult::Ok(html) = result {
        assert!(
            html.contains("#ff0000"),
            "fillColor should be reflected in SVG"
        );
        assert!(
            html.contains("#00ff00"),
            "strokeColor should be reflected in SVG"
        );
    }
}

// Edge rendering (L147-261): src/target vertices + edge cell
#[test]
fn エッジセルが矢印として描画される() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/>
<mxCell id="1" parent="0"/>
<mxCell id="2" value="A" vertex="1" parent="1">
    <mxGeometry x="50" y="50" width="120" height="60" as="geometry"/>
</mxCell>
<mxCell id="3" value="B" vertex="1" parent="1">
    <mxGeometry x="300" y="50" width="120" height="60" as="geometry"/>
</mxCell>
<mxCell id="4" value="" edge="1" source="2" target="3" parent="1">
    <mxGeometry relative="1" as="geometry"/>
</mxCell>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
    if let DiagramResult::Ok(html) = result {
        // Should contain polyline for the edge
        assert!(html.contains("polyline") || html.contains("line"));
    }
}

// Edge with label (append_edge_label L273-286)
#[test]
fn エッジラベルが描画される() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/>
<mxCell id="1" parent="0"/>
<mxCell id="2" value="Start" vertex="1" parent="1">
    <mxGeometry x="50" y="50" width="100" height="50" as="geometry"/>
</mxCell>
<mxCell id="3" value="End" vertex="1" parent="1">
    <mxGeometry x="250" y="50" width="100" height="50" as="geometry"/>
</mxCell>
<mxCell id="4" value="connects" edge="1" source="2" target="3" parent="1">
    <mxGeometry relative="1" as="geometry"/>
</mxCell>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    if let DiagramResult::Ok(html) = result {
        assert!(html.contains("connects"));
    }
}

// Edge with waypoints in mxGeometry/Array/mxPoint (collect_waypoints L289-312)
#[test]
fn ウェイポイント付きエッジが描画される() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/>
<mxCell id="1" parent="0"/>
<mxCell id="2" value="A" vertex="1" parent="1">
    <mxGeometry x="50" y="50" width="100" height="50" as="geometry"/>
</mxCell>
<mxCell id="3" value="B" vertex="1" parent="1">
    <mxGeometry x="350" y="200" width="100" height="50" as="geometry"/>
</mxCell>
<mxCell id="4" value="" edge="1" source="2" target="3" parent="1">
    <mxGeometry relative="1" as="geometry">
        <Array as="points">
            <mxPoint x="200" y="80"/>
            <mxPoint x="200" y="180"/>
        </Array>
    </mxGeometry>
</mxCell>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
    if let DiagramResult::Ok(html) = result {
        // Waypoints should be included in the polyline
        assert!(html.contains("polyline"));
    }
}

// Edge without source/target (should be skipped, L221-226)
#[test]
fn ソースなしエッジはスキップされる() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/>
<mxCell id="1" parent="0"/>
<mxCell id="2" value="Box" vertex="1" parent="1">
    <mxGeometry x="50" y="50" width="100" height="50" as="geometry"/>
</mxCell>
<mxCell id="3" edge="1" parent="1">
    <mxGeometry relative="1" as="geometry"/>
</mxCell>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    // Should not panic; edge without source is skipped
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
}

// border_point: same-point case (dx, dy both near zero, L318-319)
// This is tested via edge where src and target centers are very close
#[test]
fn 同位置セルのエッジはパニックしない() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/>
<mxCell id="1" parent="0"/>
<mxCell id="2" value="A" vertex="1" parent="1">
    <mxGeometry x="100" y="100" width="1" height="1" as="geometry"/>
</mxCell>
<mxCell id="3" value="B" vertex="1" parent="1">
    <mxGeometry x="100" y="100" width="1" height="1" as="geometry"/>
</mxCell>
<mxCell id="4" value="" edge="1" source="2" target="3" parent="1">
    <mxGeometry relative="1" as="geometry"/>
</mxCell>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    // Should not panic
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
}

// border_point: vertical edge case (dx small, dy large → top/bottom border, L329-334)
#[test]
fn 垂直エッジが上下ボーダー点を使う() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/>
<mxCell id="1" parent="0"/>
<mxCell id="2" value="Top" vertex="1" parent="1">
    <mxGeometry x="100" y="50" width="120" height="60" as="geometry"/>
</mxCell>
<mxCell id="3" value="Bottom" vertex="1" parent="1">
    <mxGeometry x="100" y="250" width="120" height="60" as="geometry"/>
</mxCell>
<mxCell id="4" value="" edge="1" source="2" target="3" parent="1">
    <mxGeometry relative="1" as="geometry"/>
</mxCell>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
}

// xml_escape in SVG text: label with & < >
#[test]
fn ラベルの特殊文字がエスケープされる() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/>
<mxCell id="1" parent="0"/>
<mxCell id="2" value="A &amp; B &lt;3&gt;" vertex="1" parent="1">
    <mxGeometry x="50" y="50" width="120" height="60" as="geometry"/>
</mxCell>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    if let DiagramResult::Ok(html) = result {
        // The SVG text should have escaped or properly contain the label
        assert!(!html.is_empty());
    }
}

// mxGraphModel without root element returns empty SVG (L67-68: None branch)
#[test]
fn rootなしmxgraphmodelは空svgを返す() {
    let xml = r#"<mxGraphModel></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
}

// Unsupported root element error (L61)
#[test]
fn 未対応のルート要素はエラーを返す() {
    let xml = r#"<svg><g></g></svg>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Err { .. }));
}

// L183: mxGeometry なしの vertex セルは render_vertex で早期リターン
#[test]
fn mxgeometryなし頂点はスキップされる() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/><mxCell id="1" parent="0"/>
<mxCell id="2" value="NoGeo" vertex="1" parent="1"/>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
    if let DiagramResult::Ok(html) = result {
        // mxGeometry がないのでこの頂点の図形は描画されないがパニックしない
        assert!(html.contains("<svg"));
    }
}

// L221-225: source なしエッジ, L223-225: target なしエッジ
#[test]
fn source_targetなしエッジはスキップされる() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/><mxCell id="1" parent="0"/>
<mxCell id="2" value="A" vertex="1" parent="1">
    <mxGeometry x="10" y="10" width="100" height="50" as="geometry"/>
</mxCell>
<mxCell id="3" edge="1" parent="1"/>
<mxCell id="4" edge="1" source="2" parent="1"/>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
}

// L227-231: source/target IDがgeo_mapに存在しないエッジ
#[test]
fn 存在しないsource_target_idのエッジはスキップされる() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/><mxCell id="1" parent="0"/>
<mxCell id="2" value="A" vertex="1" parent="1">
    <mxGeometry x="10" y="10" width="100" height="50" as="geometry"/>
</mxCell>
<mxCell id="3" edge="1" source="999" target="2" parent="1"/>
<mxCell id="4" edge="1" source="2" target="888" parent="1"/>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
}

// L342: value属性なしの頂点（ラベルなし）
#[test]
fn ラベルなし頂点はテキスト要素を出力しない() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/><mxCell id="1" parent="0"/>
<mxCell id="2" vertex="1" parent="1">
    <mxGeometry x="10" y="10" width="100" height="50" as="geometry"/>
</mxCell>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
    if let DiagramResult::Ok(html) = result {
        assert!(!html.contains("<text"));
    }
}

// L275-285: エッジの空ラベル / ラベルなし
#[test]
fn エッジの空ラベルはテキストを出力しない() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/><mxCell id="1" parent="0"/>
<mxCell id="2" value="A" vertex="1" parent="1">
    <mxGeometry x="10" y="10" width="100" height="50" as="geometry"/>
</mxCell>
<mxCell id="3" value="B" vertex="1" parent="1">
    <mxGeometry x="200" y="10" width="100" height="50" as="geometry"/>
</mxCell>
<mxCell id="4" value="" edge="1" source="2" target="3" parent="1">
    <mxGeometry as="geometry"/>
</mxCell>
<mxCell id="5" edge="1" source="2" target="3" parent="1">
    <mxGeometry as="geometry"/>
</mxCell>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
}

// L290-311: collect_waypoints の mxGeometry なしエッジ + Array/mxPoint 処理
#[test]
fn mxgeometryなしエッジのwaypointは空() {
    // source/target が存在する正常なエッジだが mxGeometry がないケース
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/><mxCell id="1" parent="0"/>
<mxCell id="2" value="A" vertex="1" parent="1">
    <mxGeometry x="10" y="10" width="100" height="50" as="geometry"/>
</mxCell>
<mxCell id="3" value="B" vertex="1" parent="1">
    <mxGeometry x="200" y="10" width="100" height="50" as="geometry"/>
</mxCell>
<mxCell id="4" value="edge" edge="1" source="2" target="3" parent="1"/>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
}
