use eframe::egui::{self};

#[derive(Clone)]
pub(crate) struct MathJaxCache(
    std::sync::Arc<egui::mutex::Mutex<std::collections::BTreeMap<String, String>>>,
);

impl Default for MathJaxCache {
    fn default() -> Self {
        Self(std::sync::Arc::new(egui::mutex::Mutex::new(
            Default::default(),
        )))
    }
}

pub(crate) fn render_math(ui: &mut egui::Ui, tex: &str, is_inline: bool) {
    const MATH_BLOCK_H_MARGIN: i8 = 8;
    const MATH_BLOCK_V_MARGIN: i8 = 4;
    const MATH_BLOCK_CORNER_RADIUS: u8 = 4;
    const EX_TO_PX: f32 = 8.5;
    const INLINE_MATH_MARGIN_TOP: i8 = -8;
    const BLOCK_MATH_MARGIN_VERTICAL: i8 = -10;
    let tex = tex.trim();
    if tex.is_empty() {
        return;
    }

    let text_color = ui.visuals().text_color();
    let hex_color = format!(
        "#{:02x}{:02x}{:02x}",
        text_color.r(),
        text_color.g(),
        text_color.b()
    );

    let is_dark = ui.visuals().dark_mode;
    let cache_key = format!(
        "{}:{}:{}:{}",
        if is_dark { "dark" } else { "light" },
        hex_color,
        if is_inline { "inline" } else { "block" },
        tex
    );

    let cache = ui.ctx().memory_mut(|mem| {
        mem.data
            .get_temp_mut_or_default::<MathJaxCache>(egui::Id::new("katana_mathjax_cache"))
            .clone()
    });

    let uri = {
        let mut map = cache.0.lock();
        if let Some(cached_uri) = map.get(&cache_key) {
            cached_uri.clone()
        } else {
            static MATHJAX_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

            let svg_result = {
                let _lock = MATHJAX_LOCK.lock().unwrap();
                if is_inline {
                    mathjax_svg::convert_to_svg_inline(tex)
                } else {
                    mathjax_svg::convert_to_svg(tex)
                }
            };

            let data_uri = match svg_result {
                Ok(svg_string) => {
                    let mut processed_svg = svg_string;
                    let width_re = regex::Regex::new(r#"width="([\d\.]+)ex""#).unwrap();
                    let height_re = regex::Regex::new(r#"height="([\d\.]+)ex""#).unwrap();

                    if let Some(caps) = width_re.captures(&processed_svg) {
                        match caps.get(1).unwrap().as_str().parse::<f32>() {
                            Ok(w_ex) => {
                                let w_px = w_ex * EX_TO_PX;
                                processed_svg = width_re
                                    .replace(&processed_svg, format!("width=\"{w_px}px\""))
                                    .into_owned();
                            }
                            Err(_) => {}
                        }
                    }
                    if let Some(caps) = height_re.captures(&processed_svg) {
                        match caps.get(1).unwrap().as_str().parse::<f32>() {
                            Ok(h_ex) => {
                                let h_px = h_ex * EX_TO_PX;
                                processed_svg = height_re
                                    .replace(&processed_svg, format!("height=\"{h_px}px\""))
                                    .into_owned();
                            }
                            Err(_) => {}
                        }
                    }

                    processed_svg = processed_svg.replace("currentColor", &hex_color);

                    use base64::{engine::general_purpose, Engine as _};
                    let b64 = general_purpose::STANDARD.encode(processed_svg.as_bytes());
                    format!("data:image/svg+xml;base64,{}", b64)
                }
                Err(e) => {
                    tracing::error!("MathJax rendering failed for {:?}: {}", tex, e);
                    String::new()
                }
            };
            map.insert(cache_key.clone(), data_uri.clone());
            data_uri
        }
    };

    if uri.is_empty() {
        if is_inline {
            ui.label(
                egui::RichText::new(tex)
                    .monospace()
                    .color(ui.visuals().error_fg_color),
            );
        } else {
            egui::Frame::new()
                .fill(ui.visuals().extreme_bg_color)
                .inner_margin(egui::Margin::symmetric(
                    MATH_BLOCK_H_MARGIN,
                    MATH_BLOCK_V_MARGIN,
                ))
                .corner_radius(egui::CornerRadius::same(MATH_BLOCK_CORNER_RADIUS))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(tex)
                            .monospace()
                            .color(ui.visuals().error_fg_color),
                    );
                });
        }
        return;
    }

    let response = if is_inline {
        egui::Frame::new()
            .inner_margin(egui::Margin {
                left: 0,
                right: 0,
                top: INLINE_MATH_MARGIN_TOP,
                bottom: 0,
            })
            .show(ui, |ui| {
                ui.add(egui::Image::new(&uri).fit_to_original_size(1.0))
            })
            .inner
    } else {
        egui::Frame::new()
            .inner_margin(egui::Margin {
                left: 0,
                right: 0,
                top: BLOCK_MATH_MARGIN_VERTICAL,
                bottom: BLOCK_MATH_MARGIN_VERTICAL,
            })
            .show(ui, |ui| {
                ui.add(egui::Image::new(&uri).fit_to_original_size(1.0))
            })
            .inner
    };

    response.on_hover_text(tex);

    let mut rect = ui.cursor();
    rect.max = rect.min;
    ui.put(
        rect,
        egui::Label::new(
            egui::RichText::new(tex)
                .size(1.0)
                .color(crate::theme_bridge::TRANSPARENT),
        ),
    );
}