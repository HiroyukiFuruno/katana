//! Katana three-pane egui shell.

#![deny(clippy::too_many_lines, clippy::cognitive_complexity)]

use eframe::egui;
use katana_platform::FilesystemService;

use crate::{
    app_state::{AppAction, AppState, ViewMode},
    i18n,
    preview_pane::{DownloadRequest, PreviewPane},
};

pub struct KatanaApp {
    state: AppState,
    fs: FilesystemService,
    pending_action: AppAction,
    preview_pane: PreviewPane,
    /// バックグラウンドダウンロードの完了通知レシーバ。
    download_rx: Option<std::sync::mpsc::Receiver<Result<(), String>>>,
}

impl KatanaApp {
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            fs: FilesystemService::new(),
            pending_action: AppAction::None,
            preview_pane: PreviewPane::default(),
            download_rx: None,
        }
    }

    fn take_action(&mut self) -> AppAction {
        std::mem::replace(&mut self.pending_action, AppAction::None)
    }

    fn refresh_preview(&mut self, source: &str) {
        // テキスト変更は即時反映（ダイアグラムは既存の画像を保持）。
        self.preview_pane.update_markdown_sections(source);
    }

    fn full_refresh_preview(&mut self, source: &str) {
        // ドキュメント選択や Refresh ボタン押下で全セクションを再レンダリング。
        self.preview_pane.full_render(source);
    }

    fn handle_open_workspace(&mut self, path: std::path::PathBuf) {
        match self.fs.open_workspace(&path) {
            Ok(ws) => {
                let name = ws.name().unwrap_or("unknown").to_string();
                self.state.status_message = Some(format!("Opened workspace: {name}"));
                self.state.workspace = Some(ws);
                self.state.open_documents.clear();
                self.state.active_doc_idx = None;
            }
            Err(e) => {
                self.state.status_message = Some(format!("Cannot open workspace: {e}"));
            }
        }
    }

    fn handle_select_document(&mut self, path: std::path::PathBuf) {
        match self.fs.load_document(&path) {
            Ok(doc) => {
                // ドキュメント選択時はダイアグラム含め完全レンダリング。
                self.full_refresh_preview(&doc.buffer.clone());
                self.state.open_documents.push(doc);
                self.state.active_doc_idx = Some(self.state.open_documents.len() - 1);
            }
            Err(e) => {
                self.state.status_message = Some(format!("Cannot open file: {e}"));
            }
        }
    }

    fn handle_update_buffer(&mut self, content: String) {
        if let Some(doc) = self.state.active_document_mut() {
            doc.update_buffer(content.clone());
        }
        self.refresh_preview(&content);
    }

    fn handle_save_document(&mut self) {
        let Some(doc) = self.state.active_document_mut() else {
            return;
        };
        match self.fs.save_document(doc) {
            Ok(()) => self.state.status_message = Some("Saved.".to_string()),
            Err(e) => self.state.status_message = Some(format!("Save failed: {e}")),
        }
    }

    fn process_action(&mut self, action: AppAction) {
        match action {
            AppAction::OpenWorkspace(p) => self.handle_open_workspace(p),
            AppAction::SelectDocument(p) => self.handle_select_document(p),
            AppAction::CloseDocument(idx) => {
                if idx < self.state.open_documents.len() {
                    self.state.open_documents.remove(idx);
                    self.state.active_doc_idx = if self.state.open_documents.is_empty() {
                        None
                    } else {
                        Some(self.state.open_documents.len() - 1)
                    };
                }
            }
            AppAction::UpdateBuffer(c) => self.handle_update_buffer(c),
            AppAction::SaveDocument => self.handle_save_document(),
            AppAction::RefreshDiagrams => {
                if let Some(doc) = self.state.active_document() {
                    let src = doc.buffer.clone();
                    self.full_refresh_preview(&src);
                }
            }
            AppAction::SetViewMode(mode) => {
                self.state.view_mode = mode;
            }
            AppAction::ChangeLanguage(lang) => {
                crate::i18n::set_language(&lang);
            }
            AppAction::None => {}
        }
    }

    /// ダウンロードリクエストをバックグラウンドスレッドで処理する。
    fn start_download(&mut self, req: DownloadRequest) {
        let (tx, rx) = std::sync::mpsc::channel();
        self.download_rx = Some(rx);
        self.state.status_message = Some(crate::i18n::t("downloading_plantuml"));
        let url = req.url;
        let dest = req.dest;
        std::thread::spawn(move || {
            let result = download_with_curl(&url, &dest);
            let _ = tx.send(result);
        });
    }

    /// ダウンロード完了をポーリングし、完了時にプレビューを再レンダリングする。
    fn poll_download(&mut self, ctx: &egui::Context) {
        let done = if let Some(rx) = &self.download_rx {
            match rx.try_recv() {
                Ok(Ok(())) => {
                    self.state.status_message = Some(
                        crate::i18n::t("plantuml_installed")
                    );
                    // ダウンロード完了 → プレビュー全体を再レンダリング。
                    self.pending_action = AppAction::RefreshDiagrams;
                    true
                }
                Ok(Err(e)) => {
                    self.state.status_message = Some(format!("{}{}", crate::i18n::t("download_error"), e));
                    true
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // まだ完了していないので再描画を依頼する。
                    ctx.request_repaint_after(std::time::Duration::from_millis(200));
                    false
                }
                Err(_) => true, // チャンネルクローズど。
            }
        } else {
            false
        };
        if done {
            self.download_rx = None;
        }
    }
}

/// `curl` をサブプロセスとして呼び出し、ファイルをダウンロードする。
fn download_with_curl(url: &str, dest: &std::path::Path) -> Result<(), String> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let status = std::process::Command::new("curl")
        .args(["-L", "-o", dest.to_str().unwrap_or(""), url])
        .status()
        .map_err(|e| format!("curl 起動失敗: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("ダウンロードに失敗しました".to_string())
    }
}

impl eframe::App for KatanaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ダウンロード完了をポーリング。
        self.poll_download(ctx);

        let action = self.take_action();
        self.process_action(action);

        render_menu_bar(ctx, &mut self.state, &mut self.pending_action);
        render_status_bar(ctx, &self.state);
        render_workspace_panel(ctx, &mut self.state, &mut self.pending_action);

        let mut download_req = None;
        if self.state.view_mode == ViewMode::PreviewOnly || self.state.view_mode == ViewMode::Split {
            download_req = render_preview_panel(
                ctx,
                &mut self.preview_pane,
                &self.state,
                &mut self.pending_action,
            );
        }
        if let Some(req) = download_req {
            self.start_download(req);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            render_tabs_and_toolbar(ui, &mut self.state, &mut self.pending_action);
            ui.separator();

            if self.state.view_mode == ViewMode::CodeOnly || self.state.view_mode == ViewMode::Split {
                render_editor_content(ui, &mut self.state, &mut self.pending_action);
            } else {
                // Keep the center panel from looking entirely empty if not showing code
                ui.label(i18n::t("view_mode_preview"));
            }
        });
    }
}

fn open_folder_dialog() -> Option<std::path::PathBuf> {
    rfd::FileDialog::new().pick_folder()
}

fn render_menu_bar(ctx: &egui::Context, state: &mut AppState, action: &mut AppAction) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button(crate::i18n::t("menu_file"), |ui| {
                render_file_menu(ui, state, action);
            });
            ui.menu_button(crate::i18n::t("menu_settings"), |ui| {
                render_settings_menu(ui, state, action);
            });
            render_header_right(ui, state);
        });
    });
}

fn render_file_menu(ui: &mut egui::Ui, state: &AppState, action: &mut AppAction) {
    if ui.button(crate::i18n::t("menu_open_workspace")).clicked() {
        if let Some(path) = open_folder_dialog() {
            *action = AppAction::OpenWorkspace(path);
        }
        ui.close_menu();
    }
    ui.separator();
    if ui
        .add_enabled(state.is_dirty(), egui::Button::new(crate::i18n::t("menu_save")))
        .clicked()
    {
        *action = AppAction::SaveDocument;
        ui.close_menu();
    }
}

fn render_settings_menu(ui: &mut egui::Ui, _state: &AppState, action: &mut AppAction) {
    ui.menu_button(crate::i18n::t("menu_language"), |ui| {
        let mut reset_layout = false;
        
        // en
        if ui.button("English").clicked() {
            *action = AppAction::ChangeLanguage("en".to_string());
            reset_layout = true;
        }

        // ja
        if ui.button("日本語").clicked() {
            *action = AppAction::ChangeLanguage("ja".to_string());
            reset_layout = true;
        }

        if reset_layout {
            ui.close_menu();
        }
    });
}

fn render_header_right(ui: &mut egui::Ui, state: &AppState) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if state.is_dirty() {
            ui.label("●");
        }
        if !state.ai_available() {
            ui.label("[AI: unconfigured]");
        }
    });
}

fn render_status_bar(ctx: &egui::Context, state: &AppState) {
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        let msg = state.status_message.as_deref().unwrap_or("Ready");
        ui.label(msg);
    });
}

fn render_workspace_panel(ctx: &egui::Context, state: &mut AppState, action: &mut AppAction) {
    egui::SidePanel::left("workspace_tree")
        .resizable(true)
        .min_width(160.0)
        .default_width(220.0)
        .show(ctx, |ui| {
            ui.heading(crate::i18n::t("workspace_title"));
            ui.separator();
            render_workspace_content(ui, state, action);
        });
}

fn render_workspace_content(ui: &mut egui::Ui, state: &AppState, action: &mut AppAction) {
    if let Some(ws) = &state.workspace {
        let entries = ws.tree.clone();
        let mut selected: Option<std::path::PathBuf> = None;
        for entry in &entries {
            render_tree_entry(ui, entry, &mut selected);
        }
        if let Some(path) = selected {
            *action = AppAction::SelectDocument(path);
        }
    } else {
        ui.label(crate::i18n::t("no_workspace_open"));
        ui.add_space(8.0);
        if ui.button(crate::i18n::t("menu_open_workspace")).clicked() {
            if let Some(path) = open_folder_dialog() {
                *action = AppAction::OpenWorkspace(path);
            }
        }
    }
}

fn render_preview_panel(
    ctx: &egui::Context,
    preview: &mut PreviewPane,
    state: &AppState,
    action: &mut AppAction,
) -> Option<DownloadRequest> {
    let mut download_req = None;
    egui::SidePanel::right("preview_pane")
        .resizable(true)
        .min_width(200.0)
        .default_width(400.0)
        .show(ctx, |ui| {
            render_preview_header(ui, state, action);
            ui.separator();
            if state.active_document().is_none() {
                ui.label(i18n::t("no_document_selected"));
            } else {
                download_req = preview.show(ui);
            }
        });
    download_req
}

fn render_preview_header(ui: &mut egui::Ui, state: &AppState, action: &mut AppAction) {
    ui.horizontal(|ui| {
        ui.heading("Preview");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let has_doc = state.active_document().is_some();
            if ui
                .add_enabled(has_doc, egui::Button::new("🔄"))
                .on_hover_text("Refresh diagrams")
                .clicked()
            {
                *action = AppAction::RefreshDiagrams;
            }
        });
    });
}

fn render_tabs_and_toolbar(ui: &mut egui::Ui, state: &mut AppState, action: &mut AppAction) {
    ui.horizontal(|ui| {
        // Toolbar for ViewMode
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut state.view_mode,
                ViewMode::PreviewOnly,
                i18n::t("view_mode_preview"),
            );
            ui.selectable_value(
                &mut state.view_mode,
                ViewMode::CodeOnly,
                i18n::t("view_mode_code"),
            );
            ui.selectable_value(
                &mut state.view_mode,
                ViewMode::Split,
                i18n::t("view_mode_split"),
            );
        });

        ui.separator();

        // Tabs
        let mut close_idx = None;
        egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal(|ui| {
                for (idx, doc) in state.open_documents.iter().enumerate() {
                    let is_active = state.active_doc_idx == Some(idx);
                    let mut title = doc.file_name().unwrap_or("untitled").to_string();
                    if doc.is_dirty {
                        title.push_str(" ●");
                    }

                    // A basic tab: Button + Close button
                    let resp = ui.selectable_label(is_active, title);
                    if resp.clicked() {
                        state.active_doc_idx = Some(idx);
                    }
                    if ui.button("❌").clicked() {
                        close_idx = Some(idx);
                    }
                }
            });
        });

        if let Some(idx) = close_idx {
            *action = AppAction::CloseDocument(idx);
        }
    });
}

fn render_editor_content(ui: &mut egui::Ui, state: &mut AppState, action: &mut AppAction) {
    if let Some(doc) = state.active_document() {
        let mut buffer = doc.buffer.clone();
        egui::ScrollArea::vertical().show(ui, |ui| {
            let response = ui.add(
                egui::TextEdit::multiline(&mut buffer)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .desired_rows(40),
            );
            if response.changed() {
                *action = AppAction::UpdateBuffer(buffer);
            }
        });
    }
}

fn render_tree_entry(
    ui: &mut egui::Ui,
    entry: &katana_core::workspace::TreeEntry,
    selected: &mut Option<std::path::PathBuf>,
) {
    use katana_core::workspace::TreeEntry;
    match entry {
        TreeEntry::Directory { path, children } => {
            render_directory_entry(ui, path, children, selected);
        }
        TreeEntry::File { path } => {
            render_file_entry(ui, entry, path, selected);
        }
    }
}

fn render_directory_entry(
    ui: &mut egui::Ui,
    path: &std::path::Path,
    children: &[katana_core::workspace::TreeEntry],
    selected: &mut Option<std::path::PathBuf>,
) {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
    egui::CollapsingHeader::new(format!("📁 {name}"))
        .default_open(true)
        .show(ui, |ui| {
            for child in children {
                render_tree_entry(ui, child, selected);
            }
        });
}

fn render_file_entry(
    ui: &mut egui::Ui,
    entry: &katana_core::workspace::TreeEntry,
    path: &std::path::Path,
    selected: &mut Option<std::path::PathBuf>,
) {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
    let label = if entry.is_markdown() {
        format!("📄 {name}")
    } else {
        format!("  {name}")
    };
    if ui.selectable_label(false, label).clicked() && entry.is_markdown() {
        *selected = Some(path.to_path_buf());
    }
}
