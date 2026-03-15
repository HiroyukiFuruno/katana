use katana_core::markdown::svg_rasterize::*;

// Minimal valid SVG.
const MINIMAL_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="100" height="100" fill="red"/></svg>"#;

#[test]
fn valid_svg_is_rasterized() {
    let result = rasterize_svg(MINIMAL_SVG, 1.0).expect("rasterize failed");
    assert_eq!(result.width, 100);
    assert_eq!(result.height, 100);
    assert_eq!(result.rgba.len(), 100 * 100 * 4);
}

#[test]
fn scale_is_applied() {
    let result = rasterize_svg(MINIMAL_SVG, 2.0).expect("rasterize failed");
    assert_eq!(result.width, 200);
    assert_eq!(result.height, 200);
}

#[test]
fn invalid_svg_returns_error() {
    let result = rasterize_svg("not valid svg", 1.0);
    assert!(matches!(result, Err(SvgRasterizeError::ParseFailed(_))));
}
