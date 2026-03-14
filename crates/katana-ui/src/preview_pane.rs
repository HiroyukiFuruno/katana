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

#[cfg(test)]
mod tests {
    use super::*;

    // ─── extract_svg ──────────────────────────────────────────────────────────

    #[test]
    fn svgタグが正しく抽出される() {
        let html =
            r#"<div class="diagram"><svg xmlns="http://www.w3.org/2000/svg"><rect/></svg></div>"#;
        let result = extract_svg(html).unwrap();
        assert!(result.starts_with("<svg"));
        assert!(result.ends_with("</svg>"));
    }

    #[test]
    fn svgがなければnoneを返す() {
        assert!(extract_svg("<div>no svg here</div>").is_none());
    }

    #[test]
    fn 複数svgは先頭開始から末尾終了を返す() {
        // rfind で末尾の</svg>を使うため、最初の<svg>から最後の</svg>まで。
        let html = r#"<svg id="a"><text>A</text></svg><svg id="b"></svg>"#;
        let result = extract_svg(html).unwrap();
        assert!(result.starts_with("<svg"));
        assert!(result.ends_with("</svg>"));
    }

    // ─── update_markdown_sections (preview synchronization) ───────────────────

    #[test]
    fn markdownのみの場合テキストセクションが更新される() {
        let mut pane = PreviewPane::default();
        pane.update_markdown_sections("# Hello\n\nWorld");
        assert_eq!(pane.sections.len(), 1);
        assert!(matches!(&pane.sections[0], RenderedSection::Markdown(s) if s.contains("Hello")));
    }

    #[test]
    fn ダイアグラムセクションは既存レンダリング結果を再利用する() {
        let mut pane = PreviewPane::default();
        // 事前に「レンダリング済み」ダイアグラムを格納しておく。
        pane.sections = vec![
            RenderedSection::Markdown("old".to_string()),
            RenderedSection::Error {
                kind: "Mermaid".to_string(),
                _source: "graph TD; A-->B".to_string(),
                message: "dummy".to_string(),
            },
        ];
        let new_source = "new text\n```mermaid\ngraph TD; A-->B\n```\nfooter";
        pane.update_markdown_sections(new_source);
        // markdown + diagram(再利用) + markdown の 3 セクション。
        assert_eq!(pane.sections.len(), 3);
        assert!(matches!(pane.sections[0], RenderedSection::Markdown(_)));
        // 既存の Error がそのまま再利用される（再レンダリングされない）。
        assert!(matches!(pane.sections[1], RenderedSection::Error { .. }));
        assert!(matches!(pane.sections[2], RenderedSection::Markdown(_)));
    }

    #[test]
    fn 既存ダイアグラムがない場合プレースホルダーエラーになる() {
        let mut pane = PreviewPane::default();
        pane.update_markdown_sections("intro\n```mermaid\ngraph TD; A-->B\n```\nfin");
        assert_eq!(pane.sections.len(), 3);
        // 未レンダリングのプレースホルダー Error になる。
        assert!(matches!(pane.sections[1], RenderedSection::Error { .. }));
    }

    #[test]
    fn テキスト変更がダイアグラムセクション数を変えない場合再利用される() {
        let mut pane = PreviewPane::default();
        // 初期ロード（テキストのみ）。
        pane.update_markdown_sections("intro\n```drawio\n<mxGraphModel/>\n```");
        assert_eq!(pane.sections.len(), 2);
        let first_kind = match &pane.sections[1] {
            RenderedSection::Error { kind, .. } => kind.clone(),
            _ => "Image".to_string(),
        };
        // テキスト変更後もダイアグラム部分は再利用される。
        pane.update_markdown_sections("changed intro\n```drawio\n<mxGraphModel/>\n```");
        assert_eq!(pane.sections.len(), 2);
        assert!(matches!(&pane.sections[1],
            RenderedSection::Error { kind, .. } if *kind == first_kind));
    }

    // ─── full_render ──────────────────────────────────────────────────────────

    #[test]
    fn markdownのみの文書はmarkdownセクション一つになる() {
        let mut pane = PreviewPane::default();
        pane.full_render("# Hello\n\nNo diagrams here.");
        assert_eq!(pane.sections.len(), 1);
        assert!(matches!(pane.sections[0], RenderedSection::Markdown(_)));
    }

    #[test]
    fn drawio_xmlは描画を試みてエラーまたは画像になる() {
        let xml = r#"<mxGraphModel><root><mxCell id="0"/></root></mxGraphModel>"#;
        let src = format!("before\n```drawio\n{xml}\n```\nafter");
        let mut pane = PreviewPane::default();
        pane.full_render(&src);
        assert_eq!(pane.sections.len(), 3);
        // Draw.io はネイティブ Rust なので Image か Error のどちらか。
        assert!(matches!(
            pane.sections[1],
            RenderedSection::Image { .. } | RenderedSection::Error { .. }
        ));
    }

    #[test]
    fn 不明フェンスはmarkdownセクションに残る() {
        let src = "intro\n```rust\nfn main() {}\n```\nfin";
        let mut pane = PreviewPane::default();
        pane.full_render(src);
        assert!(pane
            .sections
            .iter()
            .all(|s| matches!(s, RenderedSection::Markdown(_))));
    }

    #[test]
    fn 空文字列はセクションなしになる() {
        let mut pane = PreviewPane::default();
        pane.full_render("");
        assert!(pane.sections.is_empty());
    }
}
