//! Preview pane の純粋な egui UI 描画関数群。
//!
//! このモジュールはすべて egui の UI コンテキスト（`egui::Ui`）に依存するコードを含む。
//! - ボタンクリックイベント（`button().clicked()`）
//! - テクスチャロード（`ui.ctx().load_texture()`）
//! - UI コンポーネントの描画
//!
//! これらは egui のフレームコンテキストなしには実行できないため、
//! カバレッジ計測では `--ignore-filename-regex` で除外する。

use eframe::egui::{self, Vec2};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use katana_core::markdown::svg_rasterize::RasterizedSvg;

use crate::preview_pane::{DownloadRequest, RenderedSection};

/// ツール未インストール警告のテキスト色 (オレンジ)。
const WARNING_TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(255, 165, 0);

/// 単一セクションを描画する。
/// ダウンロードボタンが押された場合は `Some(DownloadRequest)` を返す。
pub(crate) fn show_section(
    ui: &mut egui::Ui,
    cache: &mut CommonMarkCache,
    section: &RenderedSection,
    id: usize,
) -> Option<DownloadRequest> {
    match section {
        RenderedSection::Markdown(md) => {
            CommonMarkViewer::new().show(ui, cache, md);
            None
        }
        RenderedSection::Image { svg_data, alt } => {
            show_rasterized(ui, svg_data, alt, id);
            None
        }
        RenderedSection::Error {
            kind,
            _source: _,
            message,
        } => {
            ui.label(
                egui::RichText::new(crate::i18n::tf(
                    "render_error",
                    &[("kind", kind), ("message", message)],
                ))
                .color(egui::Color32::YELLOW)
                .small(),
            );
            None
        }
        RenderedSection::CommandNotFound {
            tool_name,
            install_hint,
            _source: _,
        } => {
            let msg = crate::i18n::t("missing_dependency")
                .replace("{tool_name}", tool_name)
                .replace("{install_hint}", install_hint);
            ui.label(
                egui::RichText::new(msg)
                    .color(egui::Color32::YELLOW)
                    .small(),
            );
            None
        }
        RenderedSection::NotInstalled {
            kind,
            download_url,
            install_path,
        } => show_not_installed(ui, kind, download_url, install_path),
        RenderedSection::Pending { kind } => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(
                    egui::RichText::new(crate::i18n::tf("rendering", &[("kind", kind)])).weak(),
                );
            });
            None
        }
    }
}

/// 未インストールツールのダウンロードボタン UI。
pub(crate) fn show_not_installed(
    ui: &mut egui::Ui,
    kind: &str,
    download_url: &str,
    install_path: &std::path::Path,
) -> Option<DownloadRequest> {
    let mut request = None;
    ui.group(|ui| {
        ui.label(
            egui::RichText::new(crate::i18n::tf("tool_not_installed", &[("tool", kind)]))
                .color(WARNING_TEXT_COLOR),
        );
        ui.label(
            egui::RichText::new(crate::i18n::tf(
                "tool_install_path",
                &[("path", &install_path.display().to_string())],
            ))
            .small()
            .weak(),
        );
        if ui
            .button(crate::i18n::tf("tool_download", &[("tool", kind)]))
            .clicked()
        {
            request = Some(DownloadRequest {
                url: download_url.to_string(),
                dest: install_path.to_path_buf(),
            });
        }
    });
    request
}

/// ラスタライズ済み SVG を egui テクスチャとして表示する。
pub(crate) fn show_rasterized(ui: &mut egui::Ui, img: &RasterizedSvg, _alt: &str, id: usize) {
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
