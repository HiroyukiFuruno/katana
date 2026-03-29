#[cfg(target_os = "macos")]
use resvg::{render, usvg};
#[cfg(target_os = "macos")]
use std::sync::{Arc, Mutex, OnceLock};
#[cfg(target_os = "macos")]
use tiny_skia::{Pixmap, PixmapPaint, Transform};

#[cfg(target_os = "macos")]
#[rustfmt::skip]
mod constants {
    pub const APPLE_COLOR_EMOJI_FONT_PATH: &str = "/System/Library/Fonts/Apple Color Emoji.ttc";
    pub const APPLE_COLOR_EMOJI_FONT_FAMILY: &str = "Apple Color Emoji";
    pub const MIN_EMOJI_PIXEL_SIZE: u32 = 16; pub const EMOJI_RASTER_SCALE: u32 = 2;
    pub const EMOJI_CANVAS_MULTIPLIER: u32 = 4; pub const EMOJI_FONT_SIZE_RATIO: f32 = 1.0;
    pub const EMOJI_CROP_PADDING: usize = 2; pub const EMOJI_BASELINE_PADDING_RATIO: f32 = 0.2;
    pub const EMOJI_TOP_PADDING_SHARE: f32 = 0.35;
    pub const RGBA_CHANNEL_COUNT: usize = 4; pub const RGBA_ALPHA_CHANNEL_OFFSET: usize = 3;
}
#[cfg(target_os = "macos")]
use constants::*;

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct EmojiCacheKey {
    grapheme: String,
    pixel_size: u32,
}

#[cfg(target_os = "macos")]
struct EmojiCacheEntry {
    key: EmojiCacheKey,
    png: Option<Vec<u8>>,
}

/// Rasterizes a single emoji grapheme into PNG bytes using Apple Color Emoji on macOS.
#[cfg(not(target_os = "macos"))]
pub fn render_apple_color_emoji_png(_grapheme: &str, _pixel_size: u32) -> Option<Vec<u8>> {
    None
}

#[cfg(target_os = "macos")]
pub fn render_apple_color_emoji_png(grapheme: &str, pixel_size: u32) -> Option<Vec<u8>> {
    if grapheme.is_empty() || !std::path::Path::new(APPLE_COLOR_EMOJI_FONT_PATH).exists() {
        return None;
    }
    let key = EmojiCacheKey {
        grapheme: grapheme.to_owned(),
        pixel_size: pixel_size.max(MIN_EMOJI_PIXEL_SIZE),
    };

    if let Some(cached) = check_emoji_cache(&key) {
        return cached;
    }

    let rendered = render_apple_color_emoji_png_uncached(&key.grapheme, key.pixel_size);
    store_emoji_cache(key, rendered.clone());
    rendered
}

#[cfg(target_os = "macos")]
fn check_emoji_cache(key: &EmojiCacheKey) -> Option<Option<Vec<u8>>> {
    emoji_png_cache()
        .lock()
        .expect("lock")
        .iter()
        .find(|e| &e.key == key)
        .map(|e| e.png.clone())
}

#[cfg(target_os = "macos")]
fn store_emoji_cache(key: EmojiCacheKey, png: Option<Vec<u8>>) {
    emoji_png_cache()
        .lock()
        .expect("lock")
        .push(EmojiCacheEntry { key, png });
}

#[cfg(target_os = "macos")]
fn emoji_png_cache() -> &'static Mutex<Vec<EmojiCacheEntry>> {
    static EMOJI_PNG_CACHE: OnceLock<Mutex<Vec<EmojiCacheEntry>>> = OnceLock::new();
    EMOJI_PNG_CACHE.get_or_init(|| Mutex::new(Vec::new()))
}

#[cfg(target_os = "macos")]
fn render_apple_color_emoji_png_uncached(grapheme: &str, pixel_size: u32) -> Option<Vec<u8>> {
    let raster_size = pixel_size.max(MIN_EMOJI_PIXEL_SIZE) * EMOJI_RASTER_SCALE;
    let canvas_size = raster_size.saturating_mul(EMOJI_CANVAS_MULTIPLIER);
    let font_size = (raster_size as f32 * EMOJI_FONT_SIZE_RATIO).ceil();
    let midpoint = canvas_size as f32 / 2.0;
    let escaped = escape_svg_text(grapheme);
    let svg = format!(
        concat!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{canvas}\" height=\"{canvas}\" ",
            "viewBox=\"0 0 {canvas} {canvas}\">",
            "<text x=\"{midpoint}\" y=\"{midpoint}\" text-anchor=\"middle\" ",
            "dominant-baseline=\"middle\" font-family=\"{font_family}\" ",
            "font-size=\"{font_size}px\">{escaped}</text></svg>"
        ),
        canvas = canvas_size,
        midpoint = midpoint,
        font_family = APPLE_COLOR_EMOJI_FONT_FAMILY,
        font_size = font_size,
        escaped = escaped,
    );

    let opts = usvg::Options {
        fontdb: emoji_font_db(),
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_str(&svg, &opts).ok()?;
    let mut pixmap = Pixmap::new(canvas_size, canvas_size)?;
    render(&tree, Transform::identity(), &mut pixmap.as_mut());
    crop_non_transparent(&pixmap)?.encode_png().ok()
}

#[cfg(target_os = "macos")]
fn emoji_font_db() -> Arc<usvg::fontdb::Database> {
    static FONT_DB: OnceLock<Arc<usvg::fontdb::Database>> = OnceLock::new();
    Arc::clone(FONT_DB.get_or_init(|| {
        let mut db = usvg::fontdb::Database::new();
        db.load_system_fonts();
        Arc::new(db)
    }))
}

#[cfg(target_os = "macos")]
struct BoundingBox {
    min_x: usize,
    min_y: usize,
    max_x: usize,
    max_y: usize,
}

#[cfg(target_os = "macos")]
fn find_opaque_bounding_box(pixmap: &Pixmap) -> Option<BoundingBox> {
    let (width, height) = (pixmap.width() as usize, pixmap.height() as usize);
    let (mut min_x, mut min_y, mut max_x, mut max_y, mut found) =
        (width, height, 0usize, 0usize, false);

    for y in 0..height {
        for x in 0..width {
            let offset = (y * width + x) * RGBA_CHANNEL_COUNT + RGBA_ALPHA_CHANNEL_OFFSET;
            if pixmap.data()[offset] > 0 {
                found = true;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }
    found.then_some(BoundingBox {
        min_x,
        min_y,
        max_x,
        max_y,
    })
}

#[cfg(target_os = "macos")]
fn crop_non_transparent(pixmap: &Pixmap) -> Option<Pixmap> {
    let bbox = find_opaque_bounding_box(pixmap)?;
    let width = pixmap.width() as usize;
    let height = pixmap.height() as usize;

    let left = bbox.min_x.saturating_sub(EMOJI_CROP_PADDING);
    let top = bbox.min_y.saturating_sub(EMOJI_CROP_PADDING);
    let right = (bbox.max_x + EMOJI_CROP_PADDING).min(width - 1);
    let bottom = (bbox.max_y + EMOJI_CROP_PADDING).min(height - 1);

    let cropped_width = (right - left + 1) as u32;
    let cropped_height = (bottom - top + 1) as u32;
    let baseline_padding = ((cropped_height as f32) * EMOJI_BASELINE_PADDING_RATIO).round() as u32;
    let square_side = cropped_width.max(cropped_height.saturating_add(baseline_padding));
    let extra_x = (square_side - cropped_width) / 2;
    let extra_y = square_side - cropped_height;
    let extra_top = ((extra_y as f32) * EMOJI_TOP_PADDING_SHARE).round() as u32;

    let mut cropped = Pixmap::new(square_side, square_side)?;
    cropped.draw_pixmap(
        extra_x as i32 - left as i32,
        extra_top as i32 - top as i32,
        pixmap.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );
    Some(cropped)
}

fn escape_svg_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
