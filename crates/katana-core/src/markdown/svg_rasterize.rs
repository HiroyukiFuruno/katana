//! SVG ラスタライズユーティリティ。
//!
//! `resvg` + `usvg` を使い SVG テキストを RGBA ピクセルバッファに変換する。
//! 結果は egui の `ColorImage` 互換の生バイト列として返す。

use resvg::{render, usvg};
use tiny_skia::Pixmap;

/// ラスタライズ済みの SVG 画像。
#[derive(Debug, Clone)]
pub struct RasterizedSvg {
    /// ピクセル幅。
    pub width: u32,
    /// ピクセル高さ。
    pub height: u32,
    /// RGBA バイト列（row-major）。
    pub rgba: Vec<u8>,
}

/// SVG テキストを RGBA ピクセルバッファに変換する。
///
/// `scale` で出力解像度を調整する（1.0 = 元のサイズ）。
pub fn rasterize_svg(svg_text: &str, scale: f32) -> Result<RasterizedSvg, SvgRasterizeError> {
    let opts = usvg::Options {
        // システムフォントを渡さないと SVG 内テキストが不可視になる。
        fontdb: font_db(),
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_str(svg_text, &opts)
        .map_err(|e| SvgRasterizeError::ParseFailed(e.to_string()))?;
    let size = tree.size();
    let width = ((size.width() * scale) as u32).max(1);
    let height = ((size.height() * scale) as u32).max(1);
    let mut pixmap = Pixmap::new(width, height).ok_or(SvgRasterizeError::PixmapAllocationFailed)?;
    // SVG コンテンツがダーク背景で消えないよう、レンダリング前に白で塗りつぶす。
    pixmap.fill(tiny_skia::Color::WHITE);
    let transform = tiny_skia::Transform::from_scale(scale, scale);
    render(&tree, transform, &mut pixmap.as_mut());
    Ok(RasterizedSvg {
        width,
        height,
        rgba: pixmap.take(),
    })
}

/// システムフォント DB を初期化済み Arc として返す（プロセス全体で一度だけ初期化）。
fn font_db() -> std::sync::Arc<usvg::fontdb::Database> {
    static FONT_DB: std::sync::OnceLock<std::sync::Arc<usvg::fontdb::Database>> =
        std::sync::OnceLock::new();
    std::sync::Arc::clone(FONT_DB.get_or_init(|| {
        let mut db = usvg::fontdb::Database::new();
        db.load_system_fonts();
        std::sync::Arc::new(db)
    }))
}

/// SVG ラスタライズ時に発生するエラー。
#[derive(Debug, thiserror::Error)]
pub enum SvgRasterizeError {
    #[error("SVG パースに失敗しました: {0}")]
    ParseFailed(String),

    #[error("Pixmap の確保に失敗しました")]
    PixmapAllocationFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
