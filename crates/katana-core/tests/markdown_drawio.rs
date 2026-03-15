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
