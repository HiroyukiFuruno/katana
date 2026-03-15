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
    // max(1) により width/height >= 1 が保証されるため Pixmap::new は常に Some。
    let mut pixmap =
        Pixmap::new(width, height).expect("BUG: width/height >= 1 guaranteed by max(1)");
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
}
