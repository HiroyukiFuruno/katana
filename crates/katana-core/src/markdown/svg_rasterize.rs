//! SVG rasterization utility.
//!
//! Uses `resvg` + `usvg` to convert SVG text to an RGBA pixel buffer.
//! Returns the result as raw bytes compatible with egui's `ColorImage`.

use resvg::{render, usvg};
use tiny_skia::Pixmap;

/// Rasterized SVG image.
#[derive(Debug, Clone)]
pub struct RasterizedSvg {
    /// Pixel width.
    pub width: u32,
    /// Pixel height.
    pub height: u32,
    /// RGBA bytes (row-major).
    pub rgba: Vec<u8>,
}

/// Converts SVG text to an RGBA pixel buffer.
///
/// Use `scale` to adjust the output resolution (1.0 = original size).
pub fn rasterize_svg(svg_text: &str, scale: f32) -> Result<RasterizedSvg, SvgRasterizeError> {
    let opts = usvg::Options {
        // Text inside SVG becomes invisible if system fonts are not provided.
        fontdb: font_db(),
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_str(svg_text, &opts)
        .map_err(|e| SvgRasterizeError::ParseFailed(e.to_string()))?;
    let size = tree.size();
    let width = ((size.width() * scale) as u32).max(1);
    let height = ((size.height() * scale) as u32).max(1);
    // `Pixmap::new` is always `Some` because `max(1)` guarantees width/height >= 1.
    let mut pixmap =
        Pixmap::new(width, height).expect("BUG: width/height >= 1 guaranteed by max(1)");
    // Fill with white before rendering so SVG content doesn't disappear on dark backgrounds.
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

/// Errors that occur during SVG rasterization.
#[derive(Debug, thiserror::Error)]
pub enum SvgRasterizeError {
    #[error("Failed to parse SVG: {0}")]
    ParseFailed(String),
}
