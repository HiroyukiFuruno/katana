//! プレビューペイン — egui_commonmark によるネイティブ Markdown レンダリング
//! + ダイアグラムブロックのラスタライズ画像表示。
//!
//! 設計方針（MVP）:
//! - テキスト変更のたびに Markdown 部分は即座に更新する（egui_commonmark）。
//! - ダイアグラムはサブプロセスを伴うため、「🔄 Refresh」ボタン or
//!   ドキュメント選択時にのみ再レンダリングする。

use eframe::egui::{self, ScrollArea};
use egui_commonmark::CommonMarkCache;
use katana_core::markdown::diagram::DiagramKind;
use katana_core::{
    markdown::{
        diagram::{DiagramBlock, DiagramResult},
        drawio_renderer, mermaid_renderer, plantuml_renderer,
        svg_rasterize::{rasterize_svg, RasterizedSvg},
    },
    preview::{split_into_sections, PreviewSection},
};

// ─────────────────────────────────────────────
// 定数
// ─────────────────────────────────────────────

/// ダイアグラム SVG をピクセル画像に変換する際の表示スケール。
const DIAGRAM_SVG_DISPLAY_SCALE: f32 = 1.5;

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
    /// コマンドラインツールが見つからない（パスの問題など）。
    CommandNotFound {
        tool_name: String,
        install_hint: String,
        _source: String,
    },
    /// 必要なツールが未インストール— UI からダウンロードできる。
    NotInstalled {
        kind: String,
        download_url: String,
        install_path: std::path::PathBuf,
    },
    /// バックグラウンドレンダリング中のプレースホルダー。
    Pending { kind: String },
}

/// プレビューペインから返されるダウンロードリクエスト。
#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub url: String,
    pub dest: std::path::PathBuf,
}

#[derive(Default)]
pub struct PreviewPane {
    commonmark_cache: CommonMarkCache,
    pub sections: Vec<RenderedSection>,
    /// バックグラウンドレンダリング完了通知チャネル。
    render_rx: Option<std::sync::mpsc::Receiver<(usize, RenderedSection)>>,
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
    ///
    /// Markdown セクションは即座に返す。ダイアグラムは `Pending` にせて
    /// バックグラウンドスレッドでレンダリングする。
    pub fn full_render(&mut self, source: &str) {
        let raw = split_into_sections(source);
        // 前回レンダリングをキャンセル。
        self.render_rx = None;

        let mut sections = Vec::with_capacity(raw.len());
        let mut jobs: Vec<(usize, DiagramKind, String)> = Vec::new();

        for (i, section) in raw.iter().enumerate() {
            match section {
                PreviewSection::Markdown(md) => {
                    sections.push(RenderedSection::Markdown(md.clone()));
                }
                PreviewSection::Diagram { kind, source: src } => {
                    sections.push(RenderedSection::Pending {
                        kind: format!("{kind:?}"),
                    });
                    jobs.push((i, kind.clone(), src.clone()));
                }
            }
        }
        self.sections = sections;

        if jobs.is_empty() {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        self.render_rx = Some(rx);
        std::thread::spawn(move || {
            for (index, kind, src) in jobs {
                let section = render_diagram(&kind, &src);
                if tx.send((index, section)).is_err() {
                    break; // レシーバがドロップされた。
                }
            }
        });
    }

    /// プレビューペインの内容を描画する（ScrollArea 込み）。
    /// PreviewOnly モードなどスクロール同期が不要な場面で使う。
    /// ダウンロードボタンが押された場合は `Some(DownloadRequest)` を返す。
    #[allow(dead_code)]
    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<DownloadRequest> {
        // バックグラウンドレンダリング完了をポーリング。
        self.poll_renders(ui.ctx());

        let mut request: Option<DownloadRequest> = None;
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                request = self.render_sections(ui);
            });
        request
    }

    /// ScrollArea なしでプレビューコンテンツだけを描画する。
    /// 外側で ScrollArea を制御したい場合（スクロール同期など）に使う。
    pub fn show_content(&mut self, ui: &mut egui::Ui) -> Option<DownloadRequest> {
        self.poll_renders(ui.ctx());
        self.render_sections(ui)
    }

    /// セクションを順に描画する内部メソッド。
    fn render_sections(&mut self, ui: &mut egui::Ui) -> Option<DownloadRequest> {
        let mut request: Option<DownloadRequest> = None;
        for (i, section) in self.sections.iter().enumerate() {
            // セクションごとに ID スコープを分離し、同一ドキュメント内に
            // 複数のテーブルがあっても egui の Grid ID が衝突しないようにする。
            ui.push_id(format!("section_{i}"), |ui| {
                if let Some(req) =
                    crate::preview_pane_ui::show_section(ui, &mut self.commonmark_cache, section, i)
                {
                    request = Some(req);
                }
                ui.separator();
            });
        }
        if self.sections.is_empty() {
            ui.label(egui::RichText::new(crate::i18n::t("no_preview")).weak());
        }
        request
    }

    /// バックグラウンドレンダリング完了をポーリングし、届いた結果でセクションを更新する。
    fn poll_renders(&mut self, ctx: &egui::Context) {
        let still_pending = if let Some(rx) = &self.render_rx {
            let mut updated = false;
            while let Ok((idx, section)) = rx.try_recv() {
                if idx < self.sections.len() {
                    self.sections[idx] = section;
                    updated = true;
                }
            }
            if updated {
                ctx.request_repaint();
            }
            self.sections
                .iter()
                .any(|s| matches!(s, RenderedSection::Pending { .. }))
        } else {
            false
        };
        if still_pending {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        } else {
            self.render_rx = None;
        }
    }

    /// テスト用: Pending がなくなるまでバックグラウンドスレッドをブロック待機する。
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn wait_for_renders(&mut self) {
        while let Some(rx) = &self.render_rx {
            while let Ok((idx, section)) = rx.try_recv() {
                if idx < self.sections.len() {
                    self.sections[idx] = section;
                }
            }
            if self
                .sections
                .iter()
                .any(|s| matches!(s, RenderedSection::Pending { .. }))
            {
                std::thread::sleep(std::time::Duration::from_millis(50));
            } else {
                self.render_rx = None;
                break;
            }
        }
    }
}

/// `PreviewSection` をレンダリングして `RenderedSection` に変換する。
/// ダイアグラムブロックをレンダラー経由で変換し、SVG ラスタライズを試みる。
fn render_diagram(kind: &DiagramKind, source: &str) -> RenderedSection {
    let block = DiagramBlock {
        kind: kind.clone(),
        source: source.to_string(),
    };
    let result = dispatch_renderer(&block);
    map_diagram_result(kind, source, result)
}

/// `DiagramResult` を `RenderedSection` に変換する純粋関数。テスト用に公開。
pub(crate) fn map_diagram_result(
    kind: &DiagramKind,
    source: &str,
    result: DiagramResult,
) -> RenderedSection {
    match result {
        DiagramResult::Ok(html) => try_rasterize(kind, source, &html),
        DiagramResult::OkPng(bytes) => decode_png_to_section(kind, source, bytes),
        DiagramResult::Err { source, error } => RenderedSection::Error {
            kind: format!("{kind:?}"),
            _source: source,
            message: error,
        },
        DiagramResult::CommandNotFound {
            tool_name,
            install_hint,
            source,
        } => RenderedSection::CommandNotFound {
            tool_name,
            install_hint,
            _source: source,
        },
        DiagramResult::NotInstalled {
            kind: k,
            download_url,
            install_path,
        } => RenderedSection::NotInstalled {
            kind: k,
            download_url,
            install_path,
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
    match rasterize_svg(svg, DIAGRAM_SVG_DISPLAY_SCALE) {
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
pub fn extract_svg(html: &str) -> Option<&str> {
    let start = html.find("<svg")?;
    let end = html.rfind("</svg>")? + "</svg>".len();
    Some(&html[start..end])
}

/// PNG バイト列を `RenderedSection::Image` に変換する。
///
/// mmdc の PNG 出力を `image` クレートでデコードし、RGBA ピクセルバッファを取得する。
/// これにより resvg の `<foreignObject>` 非対応を完全に回避できる。
fn decode_png_to_section(kind: &DiagramKind, source: &str, bytes: Vec<u8>) -> RenderedSection {
    match decode_png_rgba(&bytes) {
        Ok(rasterized) => RenderedSection::Image {
            svg_data: rasterized,
            alt: format!("{kind:?} diagram"),
        },
        Err(e) => RenderedSection::Error {
            kind: format!("{kind:?}"),
            _source: source.to_string(),
            message: format!("PNG デコード失敗: {e}"),
        },
    }
}

/// PNG バイト列を RGBA ピクセルに変換する。
pub fn decode_png_rgba(bytes: &[u8]) -> Result<RasterizedSvg, String> {
    let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    let rgba = img.into_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(RasterizedSvg {
        width,
        height,
        rgba: rgba.into_raw(),
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // render_diagram: DrawIO の結果を RenderedSection にマップ
    #[test]
    fn render_diagram_drawio_returns_ok_section() {
        let xml = r#"<mxGraphModel><root><mxCell id="0"/><mxCell id="1" parent="0"/></root></mxGraphModel>"#;
        let section = render_diagram(&DiagramKind::DrawIo, xml);
        assert!(matches!(
            section,
            RenderedSection::Image { .. } | RenderedSection::Error { .. }
        ));
    }

    // dispatch_renderer: DrawIo 分岐
    #[test]
    fn dispatch_renderer_drawio_returns_result() {
        let block = DiagramBlock {
            kind: DiagramKind::DrawIo,
            source: r#"<mxGraphModel><root><mxCell id="0"/></root></mxGraphModel>"#.to_string(),
        };
        let result = dispatch_renderer(&block);
        assert!(matches!(
            result,
            DiagramResult::Ok(_) | DiagramResult::Err { .. }
        ));
    }

    // dispatch_renderer: Mermaid 分岐
    #[test]
    fn dispatch_renderer_mermaid_when_no_mmdc_returns_command_not_found() {
        let block = DiagramBlock {
            kind: DiagramKind::Mermaid,
            source: "graph TD; A-->B".to_string(),
        };
        let result = dispatch_renderer(&block);
        assert!(matches!(
            result,
            DiagramResult::CommandNotFound { .. }
                | DiagramResult::OkPng(_)
                | DiagramResult::Err { .. }
        ));
    }

    // dispatch_renderer: PlantUml 分岐
    #[test]
    fn dispatch_renderer_plantuml_when_no_jar_returns_not_installed() {
        std::env::set_var("PLANTUML_JAR", "/nonexistent/plantuml.jar");
        let block = DiagramBlock {
            kind: DiagramKind::PlantUml,
            source: "@startuml\nA->B\n@enduml".to_string(),
        };
        let result = dispatch_renderer(&block);
        std::env::remove_var("PLANTUML_JAR");
        assert!(matches!(result, DiagramResult::NotInstalled { .. }));
    }

    // try_rasterize: SVG 抽出失敗ケース
    #[test]
    fn try_rasterize_returns_error_when_no_svg_in_html() {
        let kind = DiagramKind::DrawIo;
        let section = try_rasterize(&kind, "source", "<div>no svg here</div>");
        assert!(matches!(section, RenderedSection::Error { .. }));
    }

    // try_rasterize: 有効な SVG で成功
    #[test]
    fn try_rasterize_returns_image_for_valid_svg() {
        let kind = DiagramKind::DrawIo;
        let html = r#"<div class="diagram"><svg width="10" height="10"><rect fill="white" width="10" height="10"/></svg></div>"#;
        let section = try_rasterize(&kind, "source", html);
        assert!(matches!(
            section,
            RenderedSection::Image { .. } | RenderedSection::Error { .. }
        ));
    }

    // decode_png_to_section: 有効 PNG
    #[test]
    fn decode_png_to_section_returns_image_for_valid_png() {
        use image::{ImageBuffer, Rgba};
        let mut buf = Vec::new();
        let img = ImageBuffer::from_pixel(2, 2, Rgba([100u8, 150, 200, 255]));
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        let section = decode_png_to_section(&DiagramKind::DrawIo, "source", buf);
        assert!(matches!(section, RenderedSection::Image { .. }));
    }

    // decode_png_to_section: 無効データ
    #[test]
    fn decode_png_to_section_returns_error_for_invalid_data() {
        let section = decode_png_to_section(&DiagramKind::DrawIo, "source", b"not png".to_vec());
        assert!(matches!(section, RenderedSection::Error { .. }));
    }

    // map_diagram_result: 全バリアント網羅テスト
    #[test]
    fn map_diagram_result_ok_delegates_to_try_rasterize() {
        let section = map_diagram_result(
            &DiagramKind::DrawIo,
            "src",
            DiagramResult::Ok("<svg width=\"10\" height=\"10\"></svg>".to_string()),
        );
        assert!(matches!(
            section,
            RenderedSection::Image { .. } | RenderedSection::Error { .. }
        ));
    }

    #[test]
    fn map_diagram_result_ok_png_delegates_to_decode() {
        use image::{ImageBuffer, Rgba};
        let mut buf = Vec::new();
        let img = ImageBuffer::from_pixel(2, 2, Rgba([0u8, 0, 0, 255]));
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        let section = map_diagram_result(&DiagramKind::Mermaid, "src", DiagramResult::OkPng(buf));
        assert!(matches!(section, RenderedSection::Image { .. }));
    }

    #[test]
    fn map_diagram_result_err_maps_to_error_section() {
        let section = map_diagram_result(
            &DiagramKind::DrawIo,
            "src",
            DiagramResult::Err {
                source: "src".to_string(),
                error: "render failed".to_string(),
            },
        );
        assert!(matches!(section, RenderedSection::Error { .. }));
    }

    #[test]
    fn map_diagram_result_command_not_found_maps_to_section() {
        let section = map_diagram_result(
            &DiagramKind::Mermaid,
            "src",
            DiagramResult::CommandNotFound {
                tool_name: "mmdc".to_string(),
                install_hint: "npm install".to_string(),
                source: "src".to_string(),
            },
        );
        assert!(matches!(section, RenderedSection::CommandNotFound { .. }));
    }

    #[test]
    fn map_diagram_result_not_installed_maps_to_section() {
        let section = map_diagram_result(
            &DiagramKind::PlantUml,
            "src",
            DiagramResult::NotInstalled {
                kind: "PlantUML".to_string(),
                download_url: "https://example.com".to_string(),
                install_path: std::path::PathBuf::from("/tmp/plantuml.jar"),
            },
        );
        assert!(matches!(section, RenderedSection::NotInstalled { .. }));
    }

    // render_diagram_mermaid: 統合テスト（mmdc の有無に依存しない）
    #[test]
    fn render_diagram_mermaid_produces_valid_section() {
        let section = render_diagram(&DiagramKind::Mermaid, "graph TD; A-->B");
        // mmdc がなければ CommandNotFound、あれば Image
        assert!(!matches!(section, RenderedSection::Pending { .. }));
    }

    // poll_renders: バックグラウンドスレッドから結果を受信してセクションを更新 (L200-206)
    #[test]
    fn poll_renders_receives_background_result_and_updates_section() {
        use std::sync::mpsc;
        let mut pane = PreviewPane::default();

        // Pending セクションを設定
        pane.sections = vec![RenderedSection::Pending {
            kind: "DrawIo".to_string(),
        }];

        // mpsc channel を作成して render_rx に設定
        let (tx, rx) = mpsc::channel();
        pane.render_rx = Some(rx);

        // バックグラウンドスレッドから結果を送信
        tx.send((0, RenderedSection::Markdown("# Result".to_string())))
            .unwrap();
        // tx をドロップして receiver が Disconnected になるようにする
        drop(tx);

        // poll_renders を呼ぶために egui Context が必要
        let ctx = egui::Context::default();
        pane.poll_renders(&ctx);

        // セクションが更新されている
        assert!(matches!(pane.sections[0], RenderedSection::Markdown(_)));
        // render_rx は None になっている（Pendingがなくなったため）
        assert!(pane.render_rx.is_none());
    }

    // wait_for_renders: Pending がなくなるまで待機する (L224-242)
    #[test]
    fn wait_for_renders_blocks_until_all_rendered() {
        use std::sync::mpsc;
        let mut pane = PreviewPane::default();

        pane.sections = vec![RenderedSection::Pending {
            kind: "DrawIo".to_string(),
        }];

        let (tx, rx) = mpsc::channel();
        pane.render_rx = Some(rx);

        // 別スレッドで送信
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(10));
            let _ = tx.send((0, RenderedSection::Markdown("# Done".to_string())));
        });

        pane.wait_for_renders();

        // 完了後は Pending でない
        assert!(pane.render_rx.is_none());
        assert!(matches!(pane.sections[0], RenderedSection::Markdown(_)));
    }

    // poll_renders: render_rx なしは何もしない (L211-213)
    #[test]
    fn poll_renders_without_rx_does_nothing() {
        let mut pane = PreviewPane::default();
        // render_rx は None のまま
        let ctx = egui::Context::default();
        pane.poll_renders(&ctx);
        // クラッシュしなければOK
        assert!(pane.render_rx.is_none());
    }

    // full_render: スレッドが起動して Pending セクションが生成される (L140-149)
    #[test]
    fn full_render_with_diagram_creates_pending_section_then_renders() {
        let mut pane = PreviewPane::default();
        // DrawIO ダイアグラムを含む内容 → Pending になる
        let source = "# Title\n```drawio\n<mxGraphModel><root></root></mxGraphModel>\n```";
        pane.full_render(source);

        // render_rx が設定される（ダイアグラムがあるため）
        assert!(pane.render_rx.is_some());

        // クラッシュしないことを確認して待機
        pane.wait_for_renders();
        assert!(pane.render_rx.is_none());
    }
}
