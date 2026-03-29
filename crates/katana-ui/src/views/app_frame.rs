//! Main application frame layout rendering.
//!
//! Renders the top-level panels when the splash screen is not opaque:
//! menu bar, status bar, title bar, workspace sidebar, tab toolbar,
//! breadcrumbs, and central content area.

use crate::app::action::ActionOps;
use crate::app_state::{AppAction, ViewMode};
use crate::preview_pane::DownloadRequest;
use crate::shell::{KatanaApp, SIDEBAR_COLLAPSED_TOGGLE_WIDTH};
use crate::shell_ui::relative_full_path;
use crate::theme_bridge;
use eframe::egui;

const CHEVRON_ICON_SIZE: f32 = 10.0;

/// Renders the main application panels (everything inside the `if !splash_is_opaque` guard).
///
/// Returns an optional `DownloadRequest` produced by split preview rendering.
pub(crate) struct MainPanels<'a> {
    pub app: &'a mut KatanaApp,
    pub theme_colors: &'a katana_platform::theme::ThemeColors,
}

impl<'a> MainPanels<'a> {
    pub fn new(
        app: &'a mut KatanaApp,
        theme_colors: &'a katana_platform::theme::ThemeColors,
    ) -> Self {
        Self { app, theme_colors }
    }

    pub fn show(self, ctx: &egui::Context) -> Option<DownloadRequest> {
        let app = self.app;
        let theme_colors = self.theme_colors;
        // Menu bar is removed as it's delegated to macOS native menu.
        let export_filenames: Vec<String> = app
            .export_tasks
            .iter()
            .map(|t| t.filename.clone())
            .collect();
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            crate::views::top_bar::StatusBar::new(
                app.state.layout.status_message.as_ref(),
                app.state.is_dirty(),
                &export_filenames,
            )
            .show(ui);
        });

        // Window title
        WindowTitle::new(app).show(ctx);

        // In-app title bar
        TitleBar::new(app, theme_colors).show(ctx);

        // Workspace sidebar
        WorkspaceSidebar::new(app).show(ctx);

        // Tab toolbar (tabs + breadcrumbs + view mode)
        TabToolbar::new(app).show(ctx);

        // Central content area
        CentralContent::new(app).show(ctx)
    }
}

struct WindowTitle<'a> {
    app: &'a mut KatanaApp,
}
impl<'a> WindowTitle<'a> {
    fn new(app: &'a mut KatanaApp) -> Self {
        Self { app }
    }
    fn show(self, ctx: &egui::Context) {
        let app = self.app;
        let ws_root_for_title = app.state.workspace.data.as_ref().map(|ws| ws.root.clone());
        let title_text = match app.state.active_document() {
            Some(doc) => {
                let fname = doc.file_name().unwrap_or("");
                let rel = relative_full_path(&doc.path, ws_root_for_title.as_deref());
                crate::shell_logic::format_window_title(
                    fname,
                    &rel,
                    &crate::i18n::get().menu.release_notes,
                )
            }
            None => "KatanA".to_string(),
        };
        if app.state.layout.last_window_title != title_text {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title_text.clone()));
            app.state.layout.last_window_title = title_text;
        }
    }
}

struct TitleBar<'a> {
    app: &'a KatanaApp,
    theme_colors: &'a katana_platform::theme::ThemeColors,
}
impl<'a> TitleBar<'a> {
    fn new(app: &'a KatanaApp, theme_colors: &'a katana_platform::theme::ThemeColors) -> Self {
        Self { app, theme_colors }
    }
    fn show(self, ctx: &egui::Context) {
        let app = self.app;
        let theme_colors = self.theme_colors;
        let title_text = &app.state.layout.last_window_title;
        egui::TopBottomPanel::top("app_title_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.centered_and_justified(|ui| {
                    let title_color =
                        theme_bridge::rgb_to_color32(theme_colors.system.title_bar_text);
                    ui.label(egui::RichText::new(title_text).small().color(title_color));
                });
            });
        });
    }
}

struct WorkspaceSidebar<'a> {
    app: &'a mut KatanaApp,
}
impl<'a> WorkspaceSidebar<'a> {
    fn new(app: &'a mut KatanaApp) -> Self {
        Self { app }
    }
    fn show(self, ctx: &egui::Context) {
        let app = self.app;
        if !app.state.layout.show_workspace {
            egui::SidePanel::left("workspace_collapsed")
                .resizable(false)
                .exact_width(SIDEBAR_COLLAPSED_TOGGLE_WIDTH)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        if ui
                            .add(egui::Button::image(
                                crate::Icon::ChevronRight
                                    .ui_image(ui, crate::icon::IconSize::Medium),
                            ))
                            .on_hover_text(crate::i18n::get().workspace.workspace_title.clone())
                            .clicked()
                        {
                            app.state.layout.show_workspace = true;
                        }
                    });
                });
        } else {
            egui::SidePanel::left("workspace_tree")
                .resizable(true)
                .min_width(crate::shell::FILE_TREE_PANEL_MIN_WIDTH)
                .default_width(crate::shell::FILE_TREE_PANEL_DEFAULT_WIDTH)
                .show(ctx, |ui| {
                    crate::views::panels::workspace::WorkspacePanel::new(
                        &mut app.state,
                        &mut app.pending_action,
                    )
                    .show(ui);
                });
        }
    }
}

struct TabToolbar<'a> {
    app: &'a mut KatanaApp,
}
impl<'a> TabToolbar<'a> {
    fn new(app: &'a mut KatanaApp) -> Self {
        Self { app }
    }
    fn show(self, ctx: &egui::Context) {
        let app = self.app;
        egui::TopBottomPanel::top("tab_toolbar").show(ctx, |ui| {
            let ws_root = app
                .state
                .workspace
                .data
                .as_ref()
                .map(|ws| ws.root.as_path());
            let tab_action = crate::views::top_bar::TabBar::new(
                ws_root,
                &app.state.document.open_documents,
                app.state.document.active_doc_idx,
                &app.state.document.recently_closed_tabs,
            )
            .show(ui);
            if let Some(a) = tab_action {
                app.pending_action = a;
            }
            let active_doc_props = app.state.active_document();
            if let Some(doc) = active_doc_props {
                let d_path = doc.path.to_string_lossy();
                let is_changelog = d_path.starts_with("Katana://ChangeLog");

                if !is_changelog {
                    let doc_path = doc.path.clone();
                    let ws_root = app.state.workspace.data.as_ref().map(|ws| ws.root.clone());
                    let rel = relative_full_path(&doc_path, ws_root.as_deref());
                    let breadcrumb_action =
                        Breadcrumbs::new(app, &rel, ws_root.as_deref()).show(ui);
                    if let Some(a) = breadcrumb_action {
                        app.pending_action = a;
                    }
                }
                let view_action = crate::views::top_bar::ViewModeBar::new(
                    app.state.active_view_mode(),
                    is_changelog,
                    app.state.active_split_direction(),
                    app.state.active_pane_order(),
                    app.state
                        .config
                        .settings
                        .settings()
                        .behavior
                        .scroll_sync_enabled,
                    app.state.scroll.sync_override,
                    app.state.update.available.is_some(),
                    app.state.update.checking,
                )
                .show(ui);
                if let Some(a) = view_action {
                    app.pending_action = a;
                }
            }
        });
    }
}

struct Breadcrumbs<'a> {
    app: &'a KatanaApp,
    rel: &'a str,
    ws_root: Option<&'a std::path::Path>,
}
impl<'a> Breadcrumbs<'a> {
    fn new(app: &'a KatanaApp, rel: &'a str, ws_root: Option<&'a std::path::Path>) -> Self {
        Self { app, rel, ws_root }
    }
    fn show(self, ui: &mut egui::Ui) -> Option<AppAction> {
        let app = self.app;
        let rel = self.rel;
        let ws_root = self.ws_root;
        let mut breadcrumb_action = None;
        ui.horizontal(|ui| {
            let segments: Vec<&str> = rel.split('/').collect();
            let mut current_path = ws_root.map(std::path::PathBuf::from).unwrap_or_default();
            for (i, seg) in segments.iter().enumerate() {
                if i > 0 {
                    ui.add(
                        egui::Image::new(crate::Icon::ChevronRight.uri())
                            .tint(ui.visuals().text_color())
                            .max_height(CHEVRON_ICON_SIZE),
                    );
                }

                if ws_root.is_none() {
                    ui.label(egui::RichText::new(*seg).small());
                    continue;
                }

                current_path = current_path.join(seg);
                let is_last = i == segments.len() - 1;

                if is_last {
                    ui.add(
                        egui::Label::new(egui::RichText::new(*seg).small())
                            .sense(egui::Sense::hover()),
                    );
                } else {
                    ui.menu_button(egui::RichText::new(*seg).small(), |ui| {
                        let mut ctx_action = crate::app_state::AppAction::None;

                        if let Some(ws) = &app.state.workspace.data {
                            if let Some(katana_core::workspace::TreeEntry::Directory {
                                children,
                                ..
                            }) = crate::views::panels::tree::find_node_in_tree(
                                &ws.tree,
                                &current_path,
                            ) {
                                crate::views::panels::workspace::BreadcrumbMenu::new(
                                    children,
                                    &mut ctx_action,
                                )
                                .show(ui);
                            }
                        }

                        if !matches!(ctx_action, crate::app_state::AppAction::None) {
                            breadcrumb_action = Some(ctx_action);
                            ui.close();
                        }
                    });
                }
            }
        });
        breadcrumb_action
    }
}

struct CentralContent<'a> {
    app: &'a mut KatanaApp,
}
impl<'a> CentralContent<'a> {
    fn new(app: &'a mut KatanaApp) -> Self {
        Self { app }
    }
    fn show(self, ctx: &egui::Context) -> Option<DownloadRequest> {
        let app = self.app;
        let mut download_req: Option<DownloadRequest> = None;
        let current_mode = app.state.active_view_mode();
        let is_split = current_mode == ViewMode::Split;
        let mut is_changelog_tab = false;

        if let Some(doc) = app.state.active_document() {
            if doc.path.to_string_lossy().starts_with("Katana://ChangeLog") {
                is_changelog_tab = true;
            }
        }

        if app.state.layout.show_toc && app.state.config.settings.settings().layout.toc_visible {
            if let Some(doc) = app.state.active_document() {
                if let Some(preview) = app.tab_previews.iter_mut().find(|p| p.path == doc.path) {
                    crate::views::panels::toc::TocPanel::new(&mut preview.pane, &app.state)
                        .show(ctx);
                }
            }
        }

        if is_changelog_tab {
            egui::CentralPanel::default().show(ctx, |ui| {
                crate::changelog::render_release_notes_tab(
                    ui,
                    &app.changelog_sections,
                    app.changelog_rx.is_some(),
                );
            });
        } else {
            if is_split {
                let split_dir = app.state.active_split_direction();
                let pane_order = app.state.active_pane_order();
                download_req =
                    crate::views::layout::split::SplitMode::new(ctx, app, split_dir, pane_order)
                        .show();
            }

            if !is_split {
                egui::CentralPanel::default()
                    .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.0))
                    .show(ctx, |ui| match current_mode {
                        ViewMode::CodeOnly => {
                            crate::views::panels::editor::EditorContent::new(
                                &mut app.state,
                                &mut app.pending_action,
                                false,
                            )
                            .show(ui);
                        }
                        ViewMode::PreviewOnly => {
                            crate::views::layout::split::PreviewOnly::new(ui, app).show();
                        }
                        ViewMode::Split => {}
                    });
            }
        }

        download_req
    }
}

/// Intercepts URL opening requests from egui output commands.
///
/// External URLs (http/https/mailto) are passed through to the browser.
/// Internal file paths are resolved and dispatched as `SelectDocument` actions.
pub(crate) fn intercept_url_commands(ctx: &egui::Context, app: &mut KatanaApp) {
    let commands = ctx.output_mut(|o| std::mem::take(&mut o.commands));
    let mut unprocessed_commands = Vec::new();

    for cmd in commands {
        if let egui::OutputCommand::OpenUrl(open) = &cmd {
            let url = &open.url;
            if url.starts_with("http://")
                || url.starts_with("https://")
                || url.starts_with("mailto:")
            {
                unprocessed_commands.push(cmd);
            } else {
                let mut path = std::path::PathBuf::from(url);
                if path.is_relative() {
                    if let Some(doc) = app.state.active_document() {
                        if let Some(parent) = doc.path.parent() {
                            path = parent.join(path);
                        }
                    }
                }
                app.process_action(ctx, AppAction::SelectDocument(path));
            }
        } else {
            unprocessed_commands.push(cmd);
        }
    }

    if !unprocessed_commands.is_empty() {
        ctx.output_mut(|o| o.commands.extend(unprocessed_commands));
    }
}
