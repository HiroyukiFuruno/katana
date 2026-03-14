//! プレビューペイン — egui_commonmark によるネイティブ Markdown レンダリング
//! + ダイアグラムブロックのラスタライズ画像表示。
//!
//! 設計方針（MVP）:
//! - テキスト変更のたびに Markdown 部分は即座に更新する（egui_commonmark）。
//! - ダイアグラムはサブプロセスを伴うため、「🔄 Refresh」ボタン or
//!   ドキュメント選択時にのみ再レンダリングする。

use eframe::egui::{self, ScrollArea, Vec2};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use katana_core::markdown::diagram::DiagramKind;
use katana_core::{
    markdown::{
        diagram::{DiagramBlock, DiagramResult},
        drawio_renderer, mermaid_renderer, plantuml_renderer,
        svg_rasterize::{rasterize_svg, RasterizedSvg},
    },
    preview::{split_into_sections, PreviewSection},
};

/// UI 層で保持するレンダリング済みセクション。
#[derive(Debug, Clone)]
pub enum RenderedSection {
    /// egui_commonmark で描画する Markdown テキスト。
    Markdown(String),
    /// ラスタライズ済みダイアグラム画像。
    Image {
        svg_data: RasterizedSvg,
        alt: String,
    },
    /// レンダリングエラー（ソースとメッセージを保持）。
    Error {
        kind: String,
        _source: String,
        message: String,
    },
}

#[derive(Default)]
pub struct PreviewPane {
    commonmark_cache: CommonMarkCache,
    pub sections: Vec<RenderedSection>,
}

impl PreviewPane {
    /// Markdown ソースからテキストセクションのみ即時更新する（ダイアグラムは保持）。
    pub fn update_markdown_sections(&mut self, source: &str) {
        let raw = split_into_sections(source);
        let mut new_sections = Vec::with_capacity(raw.len());
        let mut diagram_iter = self
            .sections
            .iter()
            .filter(|s| !matches!(s, RenderedSection::Markdown(_)));
        for section in &raw {
            match section {
                PreviewSection::Markdown(md) => {
                    new_sections.push(RenderedSection::Markdown(md.clone()));
                }
                PreviewSection::Diagram { kind, source } => {
                    // 既存のレンダリング済み画像があれば再利用する。
                    let reused =
                        diagram_iter
                            .next()
                            .cloned()
                            .unwrap_or_else(|| RenderedSection::Error {
                                kind: format!("{kind:?}"),
                                _source: source.clone(),
                                message: "🔄 プレビューを更新してください".to_string(),
                            });
                    new_sections.push(reused);
                }
            }
        }
        self.sections = new_sections;
    }

    /// 全セクション（ダイアグラム含む）を完全に再レンダリングする。
    pub fn full_render(&mut self, source: &str) {
        let raw = split_into_sections(source);
        self.sections = raw.iter().map(render_section).collect();
    }

    /// プレビューペインの内容を描画する。
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (i, section) in self.sections.iter().enumerate() {
                    show_section(ui, &mut self.commonmark_cache, section, i);
                    ui.separator();
                }
                if self.sections.is_empty() {
                    ui.label(egui::RichText::new("（プレビューなし）").weak());
                }
            });
    }
}

/// 単一セクションを描画する。
fn show_section(
    ui: &mut egui::Ui,
    cache: &mut CommonMarkCache,
    section: &RenderedSection,
    id: usize,
) {
    match section {
        RenderedSection::Markdown(md) => {
            CommonMarkViewer::new().show(ui, cache, md);
        }
        RenderedSection::Image { svg_data, alt } => {
            show_rasterized(ui, svg_data, alt, id);
        }
        RenderedSection::Error {
            kind,
            _source: _,
            message,
        } => {
            ui.label(
                egui::RichText::new(format!("⚠ [{kind}] {message}"))
                    .color(egui::Color32::YELLOW)
                    .small(),
            );
        }
    }
}

/// ラスタライズ済み SVG を egui テクスチャとして表示する。
fn show_rasterized(ui: &mut egui::Ui, img: &RasterizedSvg, _alt: &str, id: usize) {
    let color_img = egui::ColorImage::from_rgba_unmultiplied(
        [img.width as usize, img.height as usize],
        &img.rgba,
    );
    let texture = ui.ctx().load_texture(
        format!("diagram_{id}"),
        color_img,
        egui::TextureOptions::LINEAR,
    );
    let max_w = ui.available_width();
    let scale = (max_w / img.width as f32).min(1.0);
    let size = Vec2::new(img.width as f32 * scale, img.height as f32 * scale);
    ui.add(egui::Image::new((texture.id(), size)));
}

/// `PreviewSection` をレンダリングして `RenderedSection` に変換する。
fn render_section(section: &PreviewSection) -> RenderedSection {
    match section {
        PreviewSection::Markdown(md) => RenderedSection::Markdown(md.clone()),
        PreviewSection::Diagram { kind, source } => render_diagram(kind, source),
    }
}

/// ダイアグラムブロックをレンダラー経由で変換し、SVG ラスタライズを試みる。
fn render_diagram(kind: &DiagramKind, source: &str) -> RenderedSection {
    let block = DiagramBlock {
        kind: kind.clone(),
        source: source.to_string(),
    };
    let result = dispatch_renderer(&block);
    match result {
        DiagramResult::Ok(html) => try_rasterize(kind, source, &html),
        DiagramResult::Err { source, error } => RenderedSection::Error {
            kind: format!("{kind:?}"),
            _source: source,
            message: error,
        },
    }
}

/// ダイアグラム種別ごとのレンダラーに委譲する。
fn dispatch_renderer(block: &DiagramBlock) -> DiagramResult {
    match block.kind {
        DiagramKind::Mermaid => mermaid_renderer::render_mermaid(block),
        DiagramKind::PlantUml => plantuml_renderer::render_plantuml(block),
        DiagramKind::DrawIo => drawio_renderer::render_drawio(block),
    }
}

/// HTML フラグメントから SVG を抽出してラスタライズする。
fn try_rasterize(kind: &DiagramKind, source: &str, html: &str) -> RenderedSection {
    let Some(svg) = extract_svg(html) else {
        return RenderedSection::Error {
            kind: format!("{kind:?}"),
            _source: source.to_string(),
            message: "SVG の抽出に失敗しました".to_string(),
        };
    };
    match rasterize_svg(svg, 1.5) {
        Ok(img) => RenderedSection::Image {
            svg_data: img,
            alt: format!("{kind:?} diagram"),
        },
        Err(e) => RenderedSection::Error {
            kind: format!("{kind:?}"),
            _source: source.to_string(),
            message: e.to_string(),
        },
    }
}

/// HTML フラグメントから `<svg...>...</svg>` を抽出する。
fn extract_svg(html: &str) -> Option<&str> {
    let start = html.find("<svg")?;
    let end = html.rfind("</svg>")? + "</svg>".len();
    Some(&html[start..end])
}
