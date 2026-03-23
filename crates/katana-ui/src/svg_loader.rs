use std::mem::size_of;
use std::sync::{
    atomic::{AtomicU64, Ordering::Relaxed},
    Arc,
};

use egui::Vec2;
use egui::{
    load::{BytesPoll, ImageLoadResult, ImageLoader, ImagePoll, LoadError, SizeHint},
    mutex::Mutex,
    ColorImage, Context,
};
use resvg::{
    tiny_skia::Pixmap,
    usvg::{Transform, Tree},
};

const HEX_RADIX: u32 = 16;
const PERCENT_ENCODE_LEN: usize = 3;

struct Entry {
    last_used: AtomicU64,
    result: Result<Arc<ColorImage>, String>,
}

struct SvgCacheEntry {
    size_hint: SizeHint,
    data: Entry,
}

struct SvgCacheBucket {
    uri: String,
    entries: Vec<SvgCacheEntry>,
}

pub struct KatanaSvgLoader {
    pass_index: AtomicU64,
    cache: Mutex<Vec<SvgCacheBucket>>,
    options: resvg::usvg::Options<'static>,
}

impl KatanaSvgLoader {
    pub const ID: &'static str = egui::generate_loader_id!(KatanaSvgLoader);
}

pub fn install_image_loaders(ctx: &Context) {
    egui_extras::install_image_loaders(ctx);

    ctx.include_bytes(
        "bytes://icon/copy.svg",
        include_bytes!("../../../vendor/egui_commonmark_backend/src/copy.svg"),
    );
    ctx.include_bytes(
        "bytes://icon/success.svg",
        include_bytes!("../../../vendor/egui_commonmark_backend/src/check.svg"),
    );

    if !ctx.is_loader_installed(crate::http_cache_loader::PersistentHttpLoader::ID) {
        ctx.add_bytes_loader(Arc::new(
            crate::http_cache_loader::PersistentHttpLoader::default(),
        ));
    }

    if !ctx.is_loader_installed(KatanaSvgLoader::ID) {
        ctx.add_image_loader(Arc::new(KatanaSvgLoader::default()));
    }
}

fn is_supported(uri: &str) -> bool {
    if uri.starts_with("data:image/svg+xml") {
        return true;
    }
    let path = uri
        .split_once('?')
        .map_or(uri, |(before_query, _)| before_query);
    let path = path
        .split_once('#')
        .map_or(path, |(before_fragment, _)| before_fragment);
    path.ends_with(".svg") || uri.contains("img.shields.io")
}

fn preprocess_svg_bytes(bytes: &[u8]) -> Result<String, String> {
    let svg = std::str::from_utf8(bytes).map_err(|err| err.to_string())?;
    Ok(katana_core::emoji::prefer_apple_color_emoji_in_svg(svg))
}

fn rasterize_svg_bytes_with_size(
    svg_bytes: &[u8],
    size_hint: SizeHint,
    options: &resvg::usvg::Options<'_>,
) -> Result<ColorImage, String> {
    let tree = Tree::from_data(svg_bytes, options).map_err(|err| err.to_string())?;
    let source_size = Vec2::new(tree.size().width(), tree.size().height());
    let scaled_size = match size_hint {
        SizeHint::Size {
            width,
            height,
            maintain_aspect_ratio,
        } => {
            if maintain_aspect_ratio {
                let mut size = source_size;
                size *= width as f32 / source_size.x;
                if size.y > height as f32 {
                    size *= height as f32 / size.y;
                }
                size
            } else {
                Vec2::new(width as _, height as _)
            }
        }
        SizeHint::Height(height) => source_size * (height as f32 / source_size.y),
        SizeHint::Width(width) => source_size * (width as f32 / source_size.x),
        SizeHint::Scale(scale) => scale.into_inner() * source_size,
    }
    .round();

    // Fall back to 2x the SVG's original dimensions when the size hint resolves to zero
    // (e.g. when the image is placed in a zero-sized rect by egui Button).
    // Use 2x for Retina display sharpness.
    let width = if scaled_size.x < 1.0 {
        (source_size.x * 2.0) as u32
    } else {
        scaled_size.x as u32
    };
    let height = if scaled_size.y < 1.0 {
        (source_size.y * 2.0) as u32
    } else {
        scaled_size.y as u32
    };
    let mut pixmap = Pixmap::new(width, height)
        .ok_or_else(|| format!("Failed to create SVG Pixmap of size {width}x{height}"))?;
    resvg::render(
        &tree,
        Transform::from_scale(width as f32 / source_size.x, height as f32 / source_size.y),
        &mut pixmap.as_mut(),
    );

    Ok(ColorImage::from_rgba_premultiplied(
        vec![width as usize, height as usize].try_into().unwrap(),
        pixmap.data(),
    )
    .with_source_size(source_size))
}

impl Default for KatanaSvgLoader {
    fn default() -> Self {
        let mut options = resvg::usvg::Options::default();
        options.fontdb_mut().load_system_fonts();
        // Set an explicit default font family to avoid warnings on systems where Arial is missing
        // or has character coverage issues.
        options.font_family = "Verdana".to_string();

        Self {
            pass_index: AtomicU64::new(0),
            cache: Mutex::new(Vec::new()),
            options,
        }
    }
}

impl ImageLoader for KatanaSvgLoader {
    fn id(&self) -> &str {
        Self::ID
    }

    fn load(&self, ctx: &Context, uri: &str, size_hint: SizeHint) -> ImageLoadResult {
        if !is_supported(uri) {
            return Err(LoadError::NotSupported);
        }

        let mut cache = self.cache.lock();
        let bucket_idx = if let Some(idx) = cache.iter().position(|b| b.uri == uri) {
            idx
        } else {
            cache.push(SvgCacheBucket {
                uri: uri.to_owned(),
                entries: Vec::new(),
            });
            cache.len() - 1
        };
        let bucket = &mut cache[bucket_idx];

        if let Some(entry) = bucket
            .entries
            .iter()
            .find(|e| e.size_hint == size_hint)
            .map(|e| &e.data)
        {
            entry
                .last_used
                .store(self.pass_index.load(Relaxed), Relaxed);
            match entry.result.clone() {
                Ok(image) => Ok(ImagePoll::Ready { image }),
                Err(_) => Err(LoadError::NotSupported),
            }
        } else {
            let bytes_load_result = if let Some(data) = uri.strip_prefix("data:") {
                if let Some((meta, content)) = data.split_once(',') {
                    if meta.ends_with(";base64") {
                        use base64::{engine::general_purpose, Engine as _};
                        match general_purpose::STANDARD.decode(content.trim()) {
                            Ok(bytes) => Ok(BytesPoll::Ready {
                                size: None,
                                bytes: egui::load::Bytes::Shared(std::sync::Arc::from(bytes)),
                                mime: None,
                            }),
                            Err(e) => {
                                Err(LoadError::Loading(format!("Base64 decode error: {}", e)))
                            }
                        }
                    } else {
                        // Decode percent-encoded data URI content manually
                        let bytes = content.as_bytes();
                        let mut decoded = Vec::with_capacity(bytes.len());
                        let mut i = 0;
                        while i < bytes.len() {
                            if bytes[i] == b'%' && i + 2 < bytes.len() {
                                if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..=i + 2]) {
                                    if let Ok(byte) = u8::from_str_radix(hex, HEX_RADIX) {
                                        decoded.push(byte);
                                        i += PERCENT_ENCODE_LEN;
                                        continue;
                                    }
                                }
                            }
                            decoded.push(bytes[i]);
                            i += 1;
                        }
                        let decoded_str = String::from_utf8_lossy(&decoded).into_owned();

                        Ok(BytesPoll::Ready {
                            size: None,
                            bytes: egui::load::Bytes::Shared(std::sync::Arc::from(
                                decoded_str.into_bytes(),
                            )),
                            mime: None,
                        })
                    }
                } else {
                    Err(LoadError::Loading("Invalid data URI format".into()))
                }
            } else {
                ctx.try_load_bytes(uri)
            };

            match bytes_load_result {
                Ok(BytesPoll::Ready { bytes, .. }) => {
                    let result = preprocess_svg_bytes(&bytes)
                        .and_then(|svg| {
                            rasterize_svg_bytes_with_size(svg.as_bytes(), size_hint, &self.options)
                        })
                        .map(Arc::new);

                    bucket.entries.push(SvgCacheEntry {
                        size_hint,
                        data: Entry {
                            last_used: AtomicU64::new(self.pass_index.load(Relaxed)),
                            result: result.clone(),
                        },
                    });

                    match result {
                        Ok(image) => Ok(ImagePoll::Ready { image }),
                        Err(e) => {
                            tracing::warn!("SVG rasterization failed for {uri}: {e}");
                            Err(LoadError::NotSupported)
                        }
                    }
                }
                Ok(BytesPoll::Pending { size }) => Ok(ImagePoll::Pending { size }),
                Err(err) => {
                    bucket.entries.push(SvgCacheEntry {
                        size_hint,
                        data: Entry {
                            last_used: AtomicU64::new(self.pass_index.load(Relaxed)),
                            result: Err(err.to_string()),
                        },
                    });
                    Err(err)
                }
            }
        }
    }

    fn forget(&self, uri: &str) {
        self.cache.lock().retain(|bucket| bucket.uri != uri);
    }

    fn forget_all(&self) {
        self.cache.lock().clear();
    }

    fn byte_size(&self) -> usize {
        self.cache
            .lock()
            .iter()
            .flat_map(|bucket| bucket.entries.iter())
            .map(|entry| match &entry.data.result {
                Ok(image) => image.pixels.len() * size_of::<egui::Color32>(),
                Err(err) => err.len(),
            })
            .sum()
    }

    fn end_pass(&self, pass_index: u64) {
        self.pass_index.store(pass_index, Relaxed);
        let mut cache = self.cache.lock();
        cache.retain_mut(|bucket| {
            if 2 <= bucket.entries.len() {
                bucket
                    .entries
                    .retain(|entry| pass_index <= entry.data.last_used.load(Relaxed) + 1);
            }
            !bucket.entries.is_empty()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPONSOR_LOGO_DATA_URI: &str = "data:image/svg+xml;base64,PHN2ZyBmaWxsPSIjRUE0QUFBIiByb2xlPSJpbWciIHZpZXdCb3g9IjAgMCAyNCAyNCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48dGl0bGU+R2l0SHViIFNwb25zb3JzPC90aXRsZT48cGF0aCBkPSJNMTcuNjI1IDEuNDk5Yy0yLjMyIDAtNC4zNTQgMS4yMDMtNS42MjUgMy4wMy0xLjI3MS0xLjgyNy0zLjMwNS0zLjAzLTUuNjI1LTMuMDNDMy4xMjkgMS40OTkgMCA0LjI1MyAwIDguMjQ5YzAgNC4yNzUgMy4wNjggNy44NDcgNS44MjggMTAuMjI3YTMzLjE0IDMzLjE0IDAgMCAwIDUuNjE2IDMuODc2bC4wMjguMDE3LjAwOC4wMDMtLjAwMS4wMDNjLjE2My4wODUuMzQyLjEyNi41MjEuMTI1LjE3OS4wMDEuMzU4LS4wNDEuNTIxLS4xMjVsLS4wMDEtLjAwMy4wMDgtLjAwMy4wMjgtLjAxN2EzMy4xNCAzMy4xNCAwIDAgMCA1LjYxNi0zLjg3NkMyMC45MzIgMTYuMDk2IDI0IDEyLjUyNCAyNCA4LjI0OWMwLTMuOTk2LTMuMTI5LTYuNzUtNi4zNzUtNi43NXptLS45MTkgMTUuMjc1YTMwLjc2NiAzMC43NjYgMCAwIDEtNC43MDMgMy4zMTZsLS4wMDQtLjAwMi0uMDA0LjAwMmEzMC45NTUgMzAuOTU1IDAgMCAxLTQuNzAzLTMuMzE2Yy0yLjY3Ny0yLjMwNy01LjA0Ny01LjI5OC01LjA0Ny04LjUyMyAwLTIuNzU0IDIuMTIxLTQuNSA0LjEyNS00LjUgMi4wNiAwIDMuOTE0IDEuNDc5IDQuNTQ0IDMuNjg0LjE0My40OTUuNTk2Ljc5NyAxLjA4Ni43OTYuNDkuMDAxLjk0My0uMzAyIDEuMDg1LS43OTYuNjMtMi4yMDUgMi40ODQtMy42ODQgNC41NDQtMy42ODQgMi4wMDQgMCA0LjEyNSAxLjc0NiA0LjEyNSA0LjUgMCAzLjIyNS0yLjM3IDYuMjE2LTUuMDQ4IDguNTIzeiIvPjwvc3ZnPg==";

    #[test]
    fn install_image_loaders_registers_katana_svg_loader() {
        let ctx = Context::default();

        install_image_loaders(&ctx);

        assert!(ctx.is_loader_installed(KatanaSvgLoader::ID));
        assert!(ctx.is_loader_installed(crate::http_cache_loader::PersistentHttpLoader::ID));
    }

    #[test]
    fn preprocess_svg_bytes_leaves_plain_badges_unchanged() {
        let svg = br#"<svg><text x="10" font-family="Verdana">Sponsor</text></svg>"#;

        let processed = preprocess_svg_bytes(svg).expect("svg text");

        assert_eq!(
            processed,
            r#"<svg><text x="10" font-family="Verdana">Sponsor</text></svg>"#
        );
    }

    #[test]
    fn is_supported_accepts_svg_with_query_string() {
        assert!(is_supported(
            "https://img.shields.io/badge/License-MIT-blue.svg?style=flat"
        ));
    }

    #[test]
    fn is_supported_accepts_known_badge_host_without_svg_suffix() {
        assert!(is_supported(
            "https://img.shields.io/badge/Sponsor-❤️-ea4aaa?style=for-the-badge&logo=github-sponsors"
        ));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn preprocess_svg_bytes_prefers_apple_color_emoji_for_badges() {
        let svg = r#"<svg><text x="10" font-family="Verdana">❤️ Sponsor</text></svg>"#;

        let processed = preprocess_svg_bytes(svg.as_bytes()).expect("svg text");

        assert!(processed.contains(r#"font-family="Apple Color Emoji, Verdana""#));
    }

    #[test]
    fn test_svg_loader_invalid_base64() {
        use egui::load::ImageLoader;
        let loader = KatanaSvgLoader::default();
        let ctx = egui::Context::default();
        let res = loader.load(
            &ctx,
            "data:image/svg+xml;base64,!!!",
            egui::load::SizeHint::default(),
        );
        let Err(egui::load::LoadError::Loading(s)) = res else {
            panic!("expected loading error");
        };
        assert!(s.contains("Base64 decode error"));
    }

    #[test]
    fn test_svg_loader_utf8_fallback() {
        use egui::load::ImageLoader;
        let loader = KatanaSvgLoader::default();
        let ctx = egui::Context::default();
        let res = loader.load(
            &ctx,
            "data:image/svg+xml;utf8,<svg width=\"100\" height=\"100\"></svg>",
            egui::load::SizeHint::default(),
        );
        assert!(res.is_ok());
    }

    #[test]
    fn test_svg_loader_invalid_data_uri() {
        use egui::load::ImageLoader;
        let loader = KatanaSvgLoader::default();
        let ctx = egui::Context::default();
        let res = loader.load(&ctx, "data:image/svg+xml", egui::load::SizeHint::default());
        let Err(egui::load::LoadError::Loading(s)) = res else {
            panic!("expected loading error");
        };
        assert!(s.contains("Invalid data URI format"));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn rasterize_svg_bytes_renders_sponsor_badge_logo_and_emoji() {
        let svg = format!(
            concat!(
                r##"<svg xmlns="http://www.w3.org/2000/svg" width="144.25" height="28" role="img" aria-label="SPONSOR: ❤️">"##,
                r##"<g shape-rendering="crispEdges"><rect width="98.75" height="28" fill="#555"/><rect x="98.75" width="45.5" height="28" fill="#ea4aaa"/></g>"##,
                r##"<g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-rendering="geometricPrecision" font-size="100">"##,
                r##"<image x="9" y="7" width="14" height="14" href="{logo}"/>"##,
                r##"<text transform="scale(.1)" x="578.75" y="175" textLength="577.5" fill="#fff">SPONSOR</text>"##,
                r##"<text transform="scale(.1)" x="1215" y="175" textLength="215" fill="#fff" font-weight="bold">❤️</text>"##,
                r##"</g></svg>"##
            ),
            logo = SPONSOR_LOGO_DATA_URI,
        );
        let processed = preprocess_svg_bytes(svg.as_bytes()).expect("processed badge svg");
        let mut options = resvg::usvg::Options::default();
        options.fontdb_mut().load_system_fonts();
        let image = rasterize_svg_bytes_with_size(
            processed.as_bytes(),
            SizeHint::Scale(1.0.into()),
            &options,
        )
        .expect("rasterized sponsor badge");

        let logo_pixels = pixels_in_region(&image, 9, 7, 23, 21);
        let message_pixels = pixels_in_region(&image, 100, 4, 140, 24);

        assert!(
            logo_pixels
                .iter()
                .any(|pixel| pixel.r() > pixel.g() && pixel.r() > pixel.b()),
            "embedded GitHub Sponsors logo should contribute colored pixels"
        );
        assert!(
            message_pixels
                .iter()
                .any(|pixel| pixel.r() > pixel.g() && pixel.r() > pixel.b()),
            "emoji text in the right badge segment should contribute colored pixels"
        );
    }

    #[cfg(target_os = "macos")]
    fn pixels_in_region(
        image: &ColorImage,
        x_min: usize,
        y_min: usize,
        x_max: usize,
        y_max: usize,
    ) -> Vec<egui::Color32> {
        let width = image.size[0];
        let mut pixels = Vec::new();
        for y in y_min..=y_max {
            for x in x_min..=x_max {
                pixels.push(image.pixels[y * width + x]);
            }
        }
        pixels
    }

    // ── rasterize_svg_bytes_with_size: SizeHint coverage ──

    fn sample_svg() -> &'static [u8] {
        br#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50"><rect width="100" height="50" fill="red"/></svg>"#
    }

    fn sample_options() -> resvg::usvg::Options<'static> {
        resvg::usvg::Options::default()
    }

    #[test]
    fn rasterize_with_width_hint_scales_proportionally() {
        let image =
            rasterize_svg_bytes_with_size(sample_svg(), SizeHint::Width(200), &sample_options())
                .expect("rasterize with Width hint");
        assert_eq!(image.size[0], 200);
        assert_eq!(image.size[1], 100); // 50 * (200/100)
    }

    #[test]
    fn rasterize_with_height_hint_scales_proportionally() {
        let image =
            rasterize_svg_bytes_with_size(sample_svg(), SizeHint::Height(100), &sample_options())
                .expect("rasterize with Height hint");
        assert_eq!(image.size[0], 200); // 100 * (100/50)
        assert_eq!(image.size[1], 100);
    }

    #[test]
    fn rasterize_with_size_hint_maintains_aspect_ratio() {
        let image = rasterize_svg_bytes_with_size(
            sample_svg(),
            SizeHint::Size {
                width: 200,
                height: 200,
                maintain_aspect_ratio: true,
            },
            &sample_options(),
        )
        .expect("rasterize with Size hint (maintain AR)");
        // Source is 100x50 (2:1). Target maxes at 200x200.
        // Width-based: 200x100. Height fits (100 <= 200), so final is 200x100.
        assert_eq!(image.size[0], 200);
        assert_eq!(image.size[1], 100);
    }

    #[test]
    fn rasterize_with_size_hint_maintains_aspect_ratio_height_constrained() {
        let image = rasterize_svg_bytes_with_size(
            sample_svg(),
            SizeHint::Size {
                width: 400,
                height: 50,
                maintain_aspect_ratio: true,
            },
            &sample_options(),
        )
        .expect("rasterize with Size hint (height constrained)");
        // Source is 100x50. Width-based: 400x200 → height 200 > 50,
        // so re-scale: *= 50/200 → 100x50.
        assert_eq!(image.size[0], 100);
        assert_eq!(image.size[1], 50);
    }

    #[test]
    fn rasterize_with_size_hint_no_aspect_ratio() {
        let image = rasterize_svg_bytes_with_size(
            sample_svg(),
            SizeHint::Size {
                width: 300,
                height: 150,
                maintain_aspect_ratio: false,
            },
            &sample_options(),
        )
        .expect("rasterize with Size hint (no AR)");
        assert_eq!(image.size[0], 300);
        assert_eq!(image.size[1], 150);
    }

    // ── ImageLoader trait ──

    #[test]
    fn svg_loader_load_unsupported_uri_returns_not_supported() {
        let loader = KatanaSvgLoader::default();
        let ctx = Context::default();
        let result = loader.load(&ctx, "https://example.com/image.png", SizeHint::default());
        assert!(result.is_err());
    }

    #[test]
    fn svg_loader_forget_removes_cached_entry() {
        let loader = KatanaSvgLoader::default();
        loader.cache.lock().push(SvgCacheBucket {
            uri: "test.svg".to_owned(),
            entries: Vec::new(),
        });
        assert!(loader.cache.lock().iter().any(|b| b.uri == "test.svg"));
        loader.forget("test.svg");
        assert!(!loader.cache.lock().iter().any(|b| b.uri == "test.svg"));
    }

    #[test]
    fn svg_loader_forget_all_clears_cache() {
        let loader = KatanaSvgLoader::default();
        loader.cache.lock().push(SvgCacheBucket {
            uri: "a.svg".to_owned(),
            entries: Vec::new(),
        });
        loader.cache.lock().push(SvgCacheBucket {
            uri: "b.svg".to_owned(),
            entries: Vec::new(),
        });
        loader.forget_all();
        assert!(loader.cache.lock().is_empty());
    }

    #[test]
    fn svg_loader_byte_size_empty_cache() {
        let loader = KatanaSvgLoader::default();
        assert_eq!(loader.byte_size(), 0);
    }

    #[test]
    fn svg_loader_byte_size_with_entries() {
        let loader = KatanaSvgLoader::default();
        let image = ColorImage::new([2, 2], vec![egui::Color32::RED; 4]);
        let mut bucket = SvgCacheBucket {
            uri: "test.svg".to_owned(),
            entries: Vec::new(),
        };
        bucket.entries.push(SvgCacheEntry {
            size_hint: SizeHint::default(),
            data: Entry {
                last_used: AtomicU64::new(0),
                result: Ok(Arc::new(image)),
            },
        });
        loader.cache.lock().push(bucket);
        // 2x2 pixels * 4 bytes per Color32 = 16
        assert_eq!(loader.byte_size(), 4 * size_of::<egui::Color32>());
    }

    #[test]
    fn svg_loader_byte_size_with_error_entry() {
        let loader = KatanaSvgLoader::default();
        let mut bucket = SvgCacheBucket {
            uri: "err.svg".to_owned(),
            entries: Vec::new(),
        };
        bucket.entries.push(SvgCacheEntry {
            size_hint: SizeHint::default(),
            data: Entry {
                last_used: AtomicU64::new(0),
                result: Err("rasterize failed".to_string()),
            },
        });
        loader.cache.lock().push(bucket);
        assert_eq!(loader.byte_size(), "rasterize failed".len());
    }

    #[test]
    fn svg_loader_end_pass_evicts_stale_entries() {
        let loader = KatanaSvgLoader::default();
        let image = ColorImage::new([1, 1], vec![egui::Color32::RED]);

        // Create a bucket with two size hints:
        // - Scale(1.0): last used at pass 0 — stale at pass 10
        // - Scale(2.0): last used at pass 9 — still fresh (10 <= 9+1 = 10)
        let mut bucket = SvgCacheBucket {
            uri: "test.svg".to_owned(),
            entries: Vec::new(),
        };
        bucket.entries.push(SvgCacheEntry {
            size_hint: SizeHint::Scale(1.0.into()),
            data: Entry {
                last_used: AtomicU64::new(0),
                result: Ok(Arc::new(image.clone())),
            },
        });
        bucket.entries.push(SvgCacheEntry {
            size_hint: SizeHint::Scale(2.0.into()),
            data: Entry {
                last_used: AtomicU64::new(9),
                result: Ok(Arc::new(image)),
            },
        });
        loader.cache.lock().push(bucket);

        // end_pass at index 10 — eviction only runs on buckets with 2+ entries
        // Keeps entries where pass_index <= last_used + 1
        loader.end_pass(10);

        let cache = loader.cache.lock();
        let bucket = cache
            .iter()
            .find(|b| b.uri == "test.svg")
            .expect("uri should still exist");
        assert_eq!(
            bucket.entries.len(),
            1,
            "stale entry should have been evicted"
        );
    }

    // ── ImageLoader::load integration (exercising ctx.try_load_bytes) ──

    #[test]
    fn svg_loader_load_data_uri_rasterizes_and_caches() {
        let ctx = Context::default();
        install_image_loaders(&ctx);

        // Provide SVG bytes directly via include_bytes so try_load_bytes resolves.
        let svg_bytes: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50"><rect width="100" height="50" fill="red"/></svg>"#;
        let uri = "bytes://test_red_rect.svg";
        ctx.include_bytes(uri, svg_bytes);

        let loader = KatanaSvgLoader::default();

        // First load — cache miss, should rasterize and return Ready
        let result = loader
            .load(&ctx, uri, SizeHint::default())
            .expect("svg should load from included bytes");
        match result {
            ImagePoll::Ready { image } => {
                assert!(image.size[0] > 0);
                assert!(image.size[1] > 0);
            }
            ImagePoll::Pending { .. } => panic!("expected Ready on first load"),
        }

        // Second load — should hit cache and return Ready immediately
        let result2 = loader
            .load(&ctx, uri, SizeHint::default())
            .expect("cached svg should load");
        assert!(
            matches!(result2, ImagePoll::Ready { .. }),
            "second load should hit cache"
        );
    }

    #[test]
    fn svg_loader_load_returns_cached_error() {
        let loader = KatanaSvgLoader::default();
        let ctx = Context::default();

        // Insert an error entry with a .svg URI (must pass is_supported check)
        let uri = "https://example.com/broken.svg";
        let mut bucket = SvgCacheBucket {
            uri: uri.to_owned(),
            entries: Vec::new(),
        };
        bucket.entries.push(SvgCacheEntry {
            size_hint: SizeHint::default(),
            data: Entry {
                last_used: AtomicU64::new(0),
                result: Err("forced error".to_string()),
            },
        });
        loader.cache.lock().push(bucket);

        let result = loader.load(&ctx, uri, SizeHint::default());
        assert!(result.is_err(), "cached error should be returned");
    }

    #[test]
    fn svg_loader_load_invalid_svg_returns_loading_error() {
        let loader = KatanaSvgLoader::default();
        let ctx = Context::default();
        install_image_loaders(&ctx);

        // Provide invalid SVG bytes
        let uri = "bytes://invalid.svg";
        ctx.include_bytes(uri, b"not valid svg at all");

        let result = loader.load(&ctx, uri, SizeHint::default());
        assert!(result.is_err(), "invalid SVG should produce LoadError");
    }

    #[test]
    fn svg_loader_load_error_from_try_load_bytes() {
        let loader = KatanaSvgLoader::default();
        let ctx = Context::default();
        // Intentionally do NOT install image loaders — ctx has no bytes loader

        // ctx.try_load_bytes("...svg") returns Err(NotSupported) → line 166
        let result = loader.load(&ctx, "https://example.com/image.svg", SizeHint::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_data_uri_base64() {
        let loader = KatanaSvgLoader::default();
        let ctx = Context::default();
        let uri = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIxIiBoZWlnaHQ9IjEiPjwvc3ZnPg==";
        let result = loader.load(&ctx, uri, SizeHint::default());
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_data_uri_percent_encoded() {
        let loader = KatanaSvgLoader::default();
        let ctx = Context::default();
        let uri = "data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20width%3D%221%22%20height%3D%221%22%3E%3C%2Fsvg%3E";
        let result = loader.load(&ctx, uri, SizeHint::default());
        assert!(result.is_ok());
    }

    #[test]
    fn test_svg_rasterize_fallback_size() {
        let svg_data = r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"></svg>"#;
        // Zero or tiny scale should trigger the fallback
        let result = super::rasterize_svg_bytes_with_size(
            svg_data.as_bytes(),
            egui::load::SizeHint::Width(0),
            &resvg::usvg::Options::default(),
        );
        assert!(result.is_ok());
        let image = result.unwrap();
        // Since original size is 10x10, fallback should be 20x20
        assert_eq!(image.width(), 20);
        assert_eq!(image.height(), 20);
    }
}
