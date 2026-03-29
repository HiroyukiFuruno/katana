#![allow(unused_imports)]
#![allow(dead_code)]
use crate::app_state::{AppAction, AppState};
use crate::shell::KatanaApp;
use crate::state::update::UpdatePhase;
use crate::Icon;
use katana_core::update::ReleaseInfo;

use crate::i18n;
use egui::{Align, Layout};
use std::path::{Path, PathBuf};

pub(crate) fn render_about_window(
    ctx: &egui::Context,
    open: &mut bool,
    icon: Option<&egui::TextureHandle>,
    action: &mut AppAction,
) {
    const ABOUT_WINDOW_WIDTH: f32 = 400.0;
    const INNER_PADDING: f32 = 8.0;
    const ICON_SIZE: f32 = 64.0;
    const HEADING_SIZE: f32 = 20.0;
    const DESCRIPTION_SIZE: f32 = 12.0;
    const SECTION_HEADER_SIZE: f32 = 13.0;
    const SECTION_SPACING: f32 = 8.0;
    const HEADING_SPACING: f32 = 8.0;
    const SECTION_HEADER_BOTTOM: f32 = 2.0;

    let info = crate::about_info::about_info();

    egui::Window::new(format!("About {}", crate::about_info::APP_DISPLAY_NAME))
        .open(open)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .default_width(ABOUT_WINDOW_WIDTH)
        .frame(egui::Frame::window(&ctx.style()).inner_margin(INNER_PADDING))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(HEADING_SPACING);
                // App icon
                if let Some(tex) = icon {
                    ui.image(egui::load::SizedTexture::new(
                        tex.id(),
                        egui::vec2(ICON_SIZE, ICON_SIZE),
                    ));
                    ui.add_space(SECTION_SPACING);
                }
                ui.heading(
                    egui::RichText::new(info.product_name)
                        .strong()
                        .size(HEADING_SIZE),
                );
                ui.label(
                    egui::RichText::new(info.description)
                        .weak()
                        .size(DESCRIPTION_SIZE),
                );
                ui.add_space(HEADING_SPACING);
            });

            let i18n_about = &crate::i18n::get().about;

            // ── 1. Basic Info ──
            about_section_header(
                ui,
                &i18n_about.basic_info,
                SECTION_HEADER_SIZE,
                SECTION_HEADER_BOTTOM,
            );
            about_row(ui, &i18n_about.version, &format!("v{}", info.version));
            about_row(ui, &i18n_about.build, info.build);
            about_row(ui, &i18n_about.copyright, info.copyright);
            ui.add_space(SECTION_SPACING);

            // ── 2. Runtime ──
            about_section_header(
                ui,
                &i18n_about.runtime,
                SECTION_HEADER_SIZE,
                SECTION_HEADER_BOTTOM,
            );
            about_row(ui, &i18n_about.platform, &info.system.os);
            about_row(ui, &i18n_about.architecture, &info.system.arch);
            about_row(ui, &i18n_about.rust, &info.system.rustc_version);
            ui.add_space(SECTION_SPACING);

            // ── 3. License ──
            about_section_header(
                ui,
                &i18n_about.license,
                SECTION_HEADER_SIZE,
                SECTION_HEADER_BOTTOM,
            );
            about_row(ui, &i18n_about.license, info.license);
            ui.add_space(SECTION_SPACING);

            // ── 4-6. Links ──
            about_section_header(
                ui,
                &i18n_about.links,
                SECTION_HEADER_SIZE,
                SECTION_HEADER_BOTTOM,
            );
            about_link_row(
                ui,
                &i18n_about.source_code,
                info.repository,
                crate::Icon::Github,
            );
            about_link_row(
                ui,
                &i18n_about.documentation,
                info.docs_url,
                crate::Icon::Document,
            );
            about_link_row(
                ui,
                &i18n_about.report_issue,
                info.issues_url,
                crate::Icon::Bug,
            );
            ui.add_space(SECTION_SPACING);

            // ── 7. Support / Sponsor ──
            about_section_header(
                ui,
                &i18n_about.support,
                SECTION_HEADER_SIZE,
                SECTION_HEADER_BOTTOM,
            );
            if info.sponsor_url.is_empty() {
                ui.horizontal(|ui| {
                    ui.add(crate::Icon::Document.ui_image(ui, crate::icon::IconSize::Medium));
                    ui.label(
                        egui::RichText::new(crate::i18n::get().menu.release_notes.clone()).weak(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::image(
                                    crate::Icon::ExternalLink
                                        .ui_image(ui, crate::icon::IconSize::Small),
                                )
                                .frame(false),
                            )
                            .on_hover_text(crate::i18n::get().menu.release_notes.clone())
                            .clicked()
                        {
                            *action = AppAction::ShowReleaseNotes;
                        }
                    });
                });
            } else {
                about_link_row(
                    ui,
                    &i18n_about.sponsor,
                    info.sponsor_url,
                    crate::Icon::Heart,
                );
            }
            ui.add_space(SECTION_SPACING);
        });
}

fn about_section_header(ui: &mut egui::Ui, title: &str, size: f32, bottom: f32) {
    ui.separator();
    ui.label(egui::RichText::new(title).strong().size(size));
    ui.add_space(bottom);
}

fn about_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).weak());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(value);
        });
    });
}

fn about_link_row(ui: &mut egui::Ui, label: &str, url: &str, icon: crate::Icon) {
    ui.horizontal(|ui| {
        ui.add(icon.ui_image(ui, crate::icon::IconSize::Medium));
        ui.label(egui::RichText::new(label).weak());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add(
                    egui::Button::image(
                        crate::Icon::ExternalLink.ui_image(ui, crate::icon::IconSize::Small),
                    )
                    .frame(false),
                )
                .on_hover_text(url)
                .clicked()
            {
                ui.ctx().open_url(egui::OpenUrl::new_tab(url));
            }
        });
    });
}
