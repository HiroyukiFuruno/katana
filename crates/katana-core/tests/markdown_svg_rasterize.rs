use katana_core::markdown::svg_rasterize::*;

// 最小限の有効な SVG。
const MINIMAL_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="100" height="100" fill="red"/></svg>"#;

#[test]
fn 有効なsvgがラスタライズされる() {
    let result = rasterize_svg(MINIMAL_SVG, 1.0).expect("rasterize failed");
    assert_eq!(result.width, 100);
    assert_eq!(result.height, 100);
    assert_eq!(result.rgba.len(), 100 * 100 * 4);
}

#[test]
fn スケールが適用される() {
    let result = rasterize_svg(MINIMAL_SVG, 2.0).expect("rasterize failed");
    assert_eq!(result.width, 200);
    assert_eq!(result.height, 200);
}

#[test]
fn 無効なsvgはエラーを返す() {
    let result = rasterize_svg("not valid svg", 1.0);
    assert!(matches!(result, Err(SvgRasterizeError::ParseFailed(_))));
}
