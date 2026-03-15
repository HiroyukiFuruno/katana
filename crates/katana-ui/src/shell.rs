//! Katana three-pane egui shell.

use std::collections::HashMap;

use eframe::egui;
use katana_platform::FilesystemService;

use crate::{
    app_state::{AppAction, AppState, ScrollSource, ViewMode},
    i18n,
    preview_pane::{DownloadRequest, PreviewPane},
};

// macOS ネイティブメニュー FFI
#[cfg(target_os = "macos")]
mod native_menu {
    // macos_menu.m (Objective-C) で定義されたタグ定数と一致させる。
    pub const TAG_OPEN_WORKSPACE: i32 = 1;
    pub const TAG_SAVE: i32 = 2;
    pub const TAG_LANG_EN: i32 = 3;
    pub const TAG_LANG_JA: i32 = 4;

    #[allow(dead_code)]
    extern "C" {
        pub fn katana_setup_native_menu();
        pub fn katana_poll_menu_action() -> i32;
    }
}

/// macOS ネイティブメニューバーを初期化する。
/// eframe がウィンドウを生成した後に main.rs から呼ばれる。
///
/// # Safety
/// Objective-C ランタイム呼び出しを含む。メインスレッドから1回だけ呼ぶこと。
#[cfg(all(target_os = "macos", not(test)))]
pub unsafe fn native_menu_setup() {
    native_menu::katana_setup_native_menu();
}

/// FNV-1a ハッシュで文字列をu64に変換する。
fn hash_str(s: &str) -> u64 {
    crate::shell_logic::hash_str(s)
}

pub struct KatanaApp {
    state: AppState,
    fs: FilesystemService,
    pending_action: AppAction,
    /// タブ別プレビューペイン。キーはファイルパス。タブ切り替え時にキャッシュを再利用する。
    tab_panes: HashMap<std::path::PathBuf, PreviewPane>,
    /// タブ別の最終レンダリング済みコンテンツハッシュ。変化検知に使う。
    tab_hashes: HashMap<std::path::PathBuf, u64>,
    /// バックグラウンドダウンロードの完了通知レシーバ。
    download_rx: Option<std::sync::mpsc::Receiver<Result<(), String>>>,
}

impl KatanaApp {
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            fs: FilesystemService::new(),
            pending_action: AppAction::None,
            tab_panes: HashMap::new(),
            tab_hashes: HashMap::new(),
            download_rx: None,
        }
    }

    fn take_action(&mut self) -> AppAction {
        std::mem::replace(&mut self.pending_action, AppAction::None)
    }

    /// テキスト変更のみ反映（ダイアグラムは既存画像を保持）。
    fn refresh_preview(&mut self, path: &std::path::Path, source: &str) {
        self.tab_panes
            .entry(path.to_path_buf())
            .or_default()
            .update_markdown_sections(source);
    }

    /// 全セクション再レンダリング。コンテンツハッシュも更新する。
    fn full_refresh_preview(&mut self, path: &std::path::Path, source: &str) {
        let h = hash_str(source);
        self.tab_hashes.insert(path.to_path_buf(), h);
        self.tab_panes
            .entry(path.to_path_buf())
            .or_default()
            .full_render(source);
    }

    fn handle_open_workspace(&mut self, path: std::path::PathBuf) {
        match self.fs.open_workspace(&path) {
            Ok(ws) => {
                let name = ws.name().unwrap_or("unknown").to_string();
                self.state.status_message = Some(crate::i18n::tf(
                    "status_opened_workspace",
                    &[("name", &name)],
                ));
                self.state.workspace = Some(ws);
                self.state.open_documents.clear();
                self.state.active_doc_idx = None;
            }
            Err(e) => {
                let error = e.to_string();
                self.state.status_message = Some(crate::i18n::tf(
                    "status_cannot_open_workspace",
                    &[("error", &error)],
                ));
            }
        }
    }

    fn handle_select_document(&mut self, path: std::path::PathBuf) {
        // すでに開いているタブならフォーカスを移す。内容が変化していない場合はキャッシュを再利用。
        if let Some(existing_idx) = self
            .state
            .open_documents
            .iter()
            .position(|d| d.path == path)
        {
            self.state.active_doc_idx = Some(existing_idx);
            let src = self.state.open_documents[existing_idx].buffer.clone();
            let h = hash_str(&src);
            let last_h = self.tab_hashes.get(&path).copied().unwrap_or(0);
            if h != last_h {
                // 内容が変化した場合のみ再レンダリング
                self.full_refresh_preview(&path, &src);
            }
            // 変化なし → 既存の PreviewPane をそのまま使う（キャッシュ済み）
            return;
        }

        match self.fs.load_document(&path) {
            Ok(doc) => {
                let src = doc.buffer.clone();
                self.state.open_documents.push(doc);
                self.state.active_doc_idx = Some(self.state.open_documents.len() - 1);
                self.full_refresh_preview(&path, &src);
            }
            Err(e) => {
                let error = e.to_string();
                self.state.status_message = Some(crate::i18n::tf(
                    "status_cannot_open_file",
                    &[("error", &error)],
                ));
            }
        }
    }

    fn handle_update_buffer(&mut self, content: String) {
        let path = if let Some(doc) = self.state.active_document_mut() {
            doc.update_buffer(content.clone());
            doc.path.clone()
        } else {
            return;
        };
        self.refresh_preview(&path, &content);
    }

    fn handle_save_document(&mut self) {
        let Some(doc) = self.state.active_document_mut() else {
            return;
        };
        match self.fs.save_document(doc) {
            Ok(()) => self.state.status_message = Some(crate::i18n::t("status_saved")),
            Err(e) => {
                let error = e.to_string();
                self.state.status_message =
                    Some(crate::i18n::tf("status_save_failed", &[("error", &error)]));
            }
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
                    let path = doc.path.clone();
                    self.full_refresh_preview(&path, &src);
                }
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
                    self.state.status_message = Some(crate::i18n::t("plantuml_installed"));
                    self.pending_action = AppAction::RefreshDiagrams;
                    true
                }
                Ok(Err(e)) => {
                    self.state.status_message =
                        Some(format!("{}{}", crate::i18n::t("download_error"), e));
                    true
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(200));
                    false
                }
                Err(_) => true,
            }
        } else {
            false
        };
        if done {
            self.download_rx = None;
        }
    }

    /// テスト時などにプログラムからアクションを注入するためのメソッド
    pub fn trigger_action(&mut self, action: AppAction) {
        self.pending_action = action;
    }

    /// テスト時などに AppState のメソッドを呼び出すためのヘルパー
    #[doc(hidden)]
    pub fn app_state_mut(&mut self) -> &mut AppState {
        &mut self.state
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
        .map_err(|e| format!("{}: {e}", crate::i18n::t("curl_launch_failed")))?;
    if status.success() {
        Ok(())
    } else {
        Err(crate::i18n::t("download_failed"))
    }
}

impl eframe::App for KatanaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_download(ctx);

        // macOS: ネイティブメニューからのアクションをポーリングする。
        #[cfg(target_os = "macos")]
        {
            let action = unsafe { native_menu::katana_poll_menu_action() };
            match action {
                native_menu::TAG_OPEN_WORKSPACE => {
                    if let Some(path) = open_folder_dialog() {
                        self.pending_action = AppAction::OpenWorkspace(path);
                    }
                }
                native_menu::TAG_SAVE => {
                    self.pending_action = AppAction::SaveDocument;
                }
                native_menu::TAG_LANG_EN => {
                    self.pending_action = AppAction::ChangeLanguage("en".to_string());
                }
                native_menu::TAG_LANG_JA => {
                    self.pending_action = AppAction::ChangeLanguage("ja".to_string());
                }
                _ => {}
            }
        }

        let action = self.take_action();
        self.process_action(action);

        // macOS ではネイティブメニューバーを使うため egui 内メニューは非表示。
        #[cfg(not(target_os = "macos"))]
        render_menu_bar(ctx, &mut self.state, &mut self.pending_action);
        render_status_bar(ctx, &self.state);

        // ウィンドウタイトルにファイル名を反映
        let ws_root_for_title = self.state.workspace.as_ref().map(|ws| ws.root.clone());
        let title_text = match self.state.active_document() {
            Some(doc) => {
                let rel = relative_full_path(&doc.path, ws_root_for_title.as_deref());
                format!("katana — {rel}")
            }
            None => "katana".to_string(),
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title_text.clone()));

        // アプリ内タイトルバー：ウィンドウ最大化時でも常に見える位置にファイル名を表示
        egui::TopBottomPanel::top("app_title_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new(&title_text)
                            .small()
                            .color(egui::Color32::from_gray(180)),
                    );
                });
            });
        });

        // ワークスペースが非表示のときも折りたたみトグルボタンを表示する。
        if !self.state.show_workspace {
            egui::SidePanel::left("workspace_collapsed")
                .resizable(false)
                .exact_width(24.0)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        if ui
                            .button("›")
                            .on_hover_text(crate::i18n::t("workspace_title"))
                            .clicked()
                        {
                            self.state.show_workspace = true;
                        }
                    });
                });
        } else {
            render_workspace_panel(ctx, &mut self.state, &mut self.pending_action);
        }

        // タブ行 + パンくずリスト + ビューモード行を TopBottomPanel に配置する。
        egui::TopBottomPanel::top("tab_toolbar").show(ctx, |ui| {
            render_tab_bar(ui, &mut self.state, &mut self.pending_action);
            if let Some(doc) = self.state.active_document() {
                // パンくずリスト（VSCode スタイル）
                let ws_root = self.state.workspace.as_ref().map(|ws| ws.root.clone());
                let rel = relative_full_path(&doc.path, ws_root.as_deref());
                ui.horizontal(|ui| {
                    let segments: Vec<&str> = rel.split('/').collect();
                    for (i, seg) in segments.iter().enumerate() {
                        if i > 0 {
                            ui.label(egui::RichText::new("›").small());
                        }
                        ui.label(egui::RichText::new(*seg).small());
                    }
                });
                render_view_mode_bar(ui, &mut self.state);
            }
        });

        let mut download_req: Option<DownloadRequest> = None;
        let current_mode = self.state.active_view_mode();
        let is_split = current_mode == ViewMode::Split;

        // Split モード: プレビューを右ペインに表示（タブバーの下のみを分割）。
        if is_split {
            let active_path = self.state.active_document().map(|d| d.path.clone());
            let mut scroll_state = (
                self.state.scroll_fraction,
                self.state.scroll_source,
                self.state.preview_max_scroll,
            );
            egui::SidePanel::right("preview_panel")
                .resizable(true)
                .min_width(200.0)
                .default_width(400.0)
                .show(ctx, |ui| {
                    if let Some(path) = &active_path {
                        let pane = self.tab_panes.entry(path.clone()).or_default();
                        download_req = render_preview_content(
                            ui,
                            pane,
                            &self.state,
                            &mut self.pending_action,
                            true,
                            &mut scroll_state,
                        );
                    }
                });
            self.state.scroll_fraction = scroll_state.0;
            self.state.scroll_source = scroll_state.1;
            self.state.preview_max_scroll = scroll_state.2;
        }

        egui::CentralPanel::default().show(ctx, |ui| match current_mode {
            ViewMode::CodeOnly | ViewMode::Split => {
                render_editor_content(ui, &mut self.state, &mut self.pending_action, is_split);
            }
            ViewMode::PreviewOnly => {
                let active_path = self.state.active_document().map(|d| d.path.clone());
                let mut scroll_state = (0.0_f32, ScrollSource::Neither, 0.0_f32);
                if let Some(path) = active_path {
                    let pane = self.tab_panes.entry(path).or_default();
                    let req = render_preview_content(
                        ui,
                        pane,
                        &self.state,
                        &mut self.pending_action,
                        false,
                        &mut scroll_state,
                    );
                    if req.is_some() {
                        download_req = req;
                    }
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label(i18n::t("no_document_selected"));
                    });
                }
            }
        });

        if let Some(req) = download_req {
            self.start_download(req);
        }
    }
}

fn open_folder_dialog() -> Option<std::path::PathBuf> {
    rfd::FileDialog::new().pick_folder()
}

#[cfg(not(target_os = "macos"))]
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

#[cfg(not(target_os = "macos"))]
fn render_file_menu(ui: &mut egui::Ui, state: &AppState, action: &mut AppAction) {
    if ui.button(crate::i18n::t("menu_open_workspace")).clicked() {
        if let Some(path) = open_folder_dialog() {
            *action = AppAction::OpenWorkspace(path);
        }
        ui.close_menu();
    }
    ui.separator();
    if ui
        .add_enabled(
            state.is_dirty(),
            egui::Button::new(crate::i18n::t("menu_save")),
        )
        .clicked()
    {
        *action = AppAction::SaveDocument;
        ui.close_menu();
    }
}

#[cfg(not(target_os = "macos"))]
fn render_settings_menu(ui: &mut egui::Ui, _state: &AppState, action: &mut AppAction) {
    ui.menu_button(crate::i18n::t("menu_language"), |ui| {
        let mut reset_layout = false;
        if ui.button("English").clicked() {
            *action = AppAction::ChangeLanguage("en".to_string());
            reset_layout = true;
        }
        if ui.button("日本語").clicked() {
            *action = AppAction::ChangeLanguage("ja".to_string());
            reset_layout = true;
        }
        if reset_layout {
            ui.close_menu();
        }
    });
}

#[cfg(not(target_os = "macos"))]
fn render_header_right(ui: &mut egui::Ui, state: &AppState) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if state.is_dirty() {
            ui.label("*");
        }
        if !state.ai_available() {
            ui.label(crate::i18n::t("ai_unconfigured"));
        }
    });
}

fn render_status_bar(ctx: &egui::Context, state: &AppState) {
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let ready = crate::i18n::t("status_ready");
            let msg = state.status_message.as_deref().unwrap_or(&ready);
            ui.label(msg);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if state.is_dirty() {
                    ui.label("●");
                }
            });
        });
    });
}

fn render_workspace_panel(ctx: &egui::Context, state: &mut AppState, action: &mut AppAction) {
    egui::SidePanel::left("workspace_tree")
        .resizable(true)
        .min_width(120.0)
        .default_width(220.0)
        .show(ctx, |ui| {
            // パネル幅をコンテンツが押し広げないようにする。
            // available_width をコンテンツの最大幅として固定することで、
            // indent による深いネストでもパネル幅はユーザーのドラッグ操作でのみ変わる。
            let panel_width = ui.available_width();
            ui.set_max_width(panel_width);
            ui.set_min_width(panel_width);
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
            ui.horizontal(|ui| {
                ui.heading(crate::i18n::t("workspace_title"));
                // 折りたたみボタン（VSCode の「エクスプローラーを閉じる」に相当）。
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .small_button("‹")
                        .on_hover_text(crate::i18n::t("collapse_sidebar"))
                        .clicked()
                    {
                        state.show_workspace = false;
                    }
                });
            });
            // 全展開 / 全折畳ボタン
            if state.workspace.is_some() {
                ui.horizontal(|ui| {
                    if ui
                        .small_button("+")
                        .on_hover_text(crate::i18n::t("expand_all"))
                        .clicked()
                    {
                        state.force_tree_open = Some(true);
                    }
                    if ui
                        .small_button("-")
                        .on_hover_text(crate::i18n::t("collapse_all"))
                        .clicked()
                    {
                        state.force_tree_open = Some(false);
                    }
                });
            }
            ui.separator();
            render_workspace_content(ui, state, action);
        });
}

fn render_workspace_content(ui: &mut egui::Ui, state: &mut AppState, action: &mut AppAction) {
    if let Some(ws) = &state.workspace {
        let entries = ws.tree.clone();
        let mut selected: Option<std::path::PathBuf> = None;
        let force = state.force_tree_open;
        let active_path = state.active_path().map(|p| p.to_path_buf());
        egui::ScrollArea::vertical()
            .id_salt("workspace_tree_scroll")
            .show(ui, |ui| {
                // ScrollArea 内でもコンテンツが横幅を押し広げないようにする。
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                for entry in &entries {
                    render_tree_entry(ui, entry, &mut selected, force, 0, active_path.as_deref());
                }
            });
        state.force_tree_open = None; // フレームごとに消費してリセット
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

fn render_preview_content(
    ui: &mut egui::Ui,
    preview: &mut PreviewPane,
    state: &AppState,
    action: &mut AppAction,
    scroll_sync: bool,
    scroll_state: &mut (f32, ScrollSource, f32),
) -> Option<DownloadRequest> {
    let mut download_req = None;
    render_preview_header(ui, state, action);
    ui.separator();

    let (fraction, source, prev_max_scroll) = scroll_state;
    let mut scroll_area = egui::ScrollArea::both().id_salt("preview_scroll");

    // エディタからスクロールが来た場合、プレビューを追従させる
    // オフセット = fraction * preview側のmax_scroll (前フレームの値)
    let consuming_editor = scroll_sync && *source == ScrollSource::Editor;
    if consuming_editor {
        scroll_area = scroll_area.vertical_scroll_offset(*fraction * (*prev_max_scroll).max(1.0));
    }

    let output = scroll_area.show(ui, |ui| {
        download_req = preview.show_content(ui);
    });

    if scroll_sync {
        // 今フレームのmax_scrollを記録（次フレームで使用）
        let max_scroll = (output.content_size.y - output.inner_rect.height()).max(0.0);
        *prev_max_scroll = max_scroll;

        if consuming_editor {
            // エディタからの同期を消費済み → Neither にリセット
            *source = ScrollSource::Neither;
            // fraction をプレビューの実際の位置に更新（次フレームの誤検知を防ぐ）
            if max_scroll > 0.0 {
                *fraction = (output.state.offset.y / max_scroll).clamp(0.0, 1.0);
            }
        } else {
            // ユーザーがプレビューをスクロールした場合のみ更新
            if max_scroll > 0.0 {
                let current_fraction = (output.state.offset.y / max_scroll).clamp(0.0, 1.0);
                let diff = (current_fraction - *fraction).abs();
                if diff > 0.002 {
                    *fraction = current_fraction;
                    *source = ScrollSource::Preview;
                }
            }
        }
    }

    download_req
}

fn render_preview_header(ui: &mut egui::Ui, state: &AppState, action: &mut AppAction) {
    ui.horizontal(|ui| {
        ui.heading(crate::i18n::t("preview_title"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let has_doc = state.active_document().is_some();
            if ui
                .add_enabled(has_doc, egui::Button::new("\u{1F504}"))
                .on_hover_text(crate::i18n::t("refresh_diagrams"))
                .clicked()
            {
                *action = AppAction::RefreshDiagrams;
            }
        });
    });
}

/// タブ行: 開いているドキュメントのタブを横並びに表示する。
/// ← → ナビゲーションボタン付き。横スクロールバーは非表示。
fn render_tab_bar(ui: &mut egui::Ui, state: &mut AppState, action: &mut AppAction) {
    const MAX_TAB_WIDTH: f32 = 200.0;

    let mut close_idx: Option<usize> = None;
    let mut tab_action: Option<AppAction> = None;

    // ワークスペースルートを取得（相対パス計算用）
    let ws_root = state.workspace.as_ref().map(|ws| ws.root.clone());
    let doc_count = state.open_documents.len();

    // ツールチップの表示遅延を 0.25 秒に設定
    ui.style_mut().interaction.tooltip_delay = 0.25;

    ui.horizontal(|ui| {
        // 右端の ◀ ▶ ボタン分の幅を確保（ボタン2つ + セパレータ + マージン ≈ 80px）
        let nav_button_width = 80.0;
        let scroll_width = ui.available_width() - nav_button_width;

        // タブ一覧（横スクロール可能）
        egui::ScrollArea::horizontal()
            .max_width(scroll_width)
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .id_salt("tab_scroll")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (idx, doc) in state.open_documents.iter().enumerate() {
                        let is_active = state.active_doc_idx == Some(idx);
                        let filename = doc.file_name().unwrap_or("untitled").to_string();
                        let dirty_suffix = if doc.is_dirty { " *" } else { "" };
                        let title = format!("{filename}{dirty_suffix}");
                        let tooltip_path = relative_full_path(&doc.path, ws_root.as_deref());

                        // タブ描画
                        let resp = ui
                            .push_id(format!("tab_{idx}"), |ui| {
                                ui.set_max_width(MAX_TAB_WIDTH);
                                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                                ui.selectable_label(is_active, &title)
                            })
                            .inner;

                        // ツールチップ：相対パスを表示
                        let clicked = resp.clicked();
                        resp.on_hover_text(&tooltip_path);
                        if clicked && !is_active {
                            tab_action = Some(AppAction::SelectDocument(doc.path.clone()));
                        }

                        // 閉じるボタン
                        if ui.small_button("x").clicked() {
                            close_idx = Some(idx);
                        }
                        ui.add_space(4.0);
                    }
                });
            });

        // セパレータ + ◀ ▶ ボタン（右端に配置）
        ui.separator();

        let nav_enabled = doc_count > 1;
        if ui
            .add_enabled(nav_enabled, egui::Button::new("◀").small())
            .clicked()
        {
            if let Some(idx) = state.active_doc_idx {
                let new_idx = crate::shell_logic::prev_tab_index(idx, doc_count);
                tab_action = Some(AppAction::SelectDocument(
                    state.open_documents[new_idx].path.clone(),
                ));
            }
        }
        if ui
            .add_enabled(nav_enabled, egui::Button::new("▶").small())
            .clicked()
        {
            if let Some(idx) = state.active_doc_idx {
                let new_idx = crate::shell_logic::next_tab_index(idx, doc_count);
                tab_action = Some(AppAction::SelectDocument(
                    state.open_documents[new_idx].path.clone(),
                ));
            }
        }
    });

    if let Some(action_val) = tab_action {
        *action = action_val;
    } else if let Some(idx) = close_idx {
        *action = AppAction::CloseDocument(idx);
    }
}

/// ワークスペースルートからの相対フルパスを返す（ツールチップ用）。
/// 例: /workspace/specs/auth/spec.md → "specs/auth/spec.md"
fn relative_full_path(path: &std::path::Path, ws_root: Option<&std::path::Path>) -> String {
    crate::shell_logic::relative_full_path(path, ws_root)
}

/// 表示モード行: Preview / Code / Split トグルボタン（タブごとに記憶）。右寄せ配置。
fn render_view_mode_bar(ui: &mut egui::Ui, state: &mut AppState) {
    let mut mode = state.active_view_mode();
    let prev = mode;
    // allocate_ui_with_layout で高さを1行に固定して右寄せにする。
    let bar_height = ui.spacing().interact_size.y;
    let available_width = ui.available_width();
    ui.allocate_ui_with_layout(
        egui::vec2(available_width, bar_height),
        egui::Layout::right_to_left(egui::Align::Center),
        |ui| {
            // right_to_left なので逆順に追加して左→右の視覚順にする。
            ui.selectable_value(&mut mode, ViewMode::Split, i18n::t("view_mode_split"));
            ui.selectable_value(&mut mode, ViewMode::CodeOnly, i18n::t("view_mode_code"));
            ui.selectable_value(
                &mut mode,
                ViewMode::PreviewOnly,
                i18n::t("view_mode_preview"),
            );
        },
    );
    if mode != prev {
        state.set_active_view_mode(mode);
    }
}

fn render_editor_content(
    ui: &mut egui::Ui,
    state: &mut AppState,
    action: &mut AppAction,
    sync_scroll: bool,
) {
    if let Some(doc) = state.active_document() {
        let mut buffer = doc.buffer.clone();

        let mut scroll_area = egui::ScrollArea::vertical().id_salt("editor_scroll");

        // プレビューからスクロールが来た場合、エディタを追従させる
        // オフセット = fraction * editor側のmax_scroll (前フレームの値)
        let consuming_preview = sync_scroll && state.scroll_source == ScrollSource::Preview;
        if consuming_preview {
            scroll_area = scroll_area
                .vertical_scroll_offset(state.scroll_fraction * state.editor_max_scroll.max(1.0));
        }

        let output = scroll_area.show(ui, |ui| {
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

        if sync_scroll {
            // 今フレームのmax_scrollを記録（次フレームで使用）
            let max_scroll = (output.content_size.y - output.inner_rect.height()).max(0.0);
            state.editor_max_scroll = max_scroll;

            if consuming_preview {
                // プレビューからの同期を消費済み → Neither にリセット
                state.scroll_source = ScrollSource::Neither;
                // fraction をエディタの実際の位置に更新（次フレームの誤検知を防ぐ）
                if max_scroll > 0.0 {
                    state.scroll_fraction = (output.state.offset.y / max_scroll).clamp(0.0, 1.0);
                }
            } else {
                // ユーザーがエディタをスクロールした場合のみ更新
                if max_scroll > 0.0 {
                    let current_fraction = (output.state.offset.y / max_scroll).clamp(0.0, 1.0);
                    let diff = (current_fraction - state.scroll_fraction).abs();
                    if diff > 0.002 {
                        state.scroll_fraction = current_fraction;
                        state.scroll_source = ScrollSource::Editor;
                    }
                }
            }
        }
    }
}

fn render_tree_entry(
    ui: &mut egui::Ui,
    entry: &katana_core::workspace::TreeEntry,
    selected: &mut Option<std::path::PathBuf>,
    force: Option<bool>,
    depth: usize,
    active_path: Option<&std::path::Path>,
) {
    use katana_core::workspace::TreeEntry;
    match entry {
        TreeEntry::Directory { path, children } => {
            render_directory_entry(ui, path, children, selected, force, depth, active_path);
        }
        TreeEntry::File { path } => {
            render_file_entry(ui, entry, path, selected, depth, active_path);
        }
    }
}

/// インデントプレフィックスを生成する。ui.indent() を使わずフラットなレイアウトにすることで
/// SidePanel の最小幅が累積しないようにする。
fn indent_prefix(depth: usize) -> String {
    "  ".repeat(depth)
}

fn render_directory_entry(
    ui: &mut egui::Ui,
    path: &std::path::Path,
    children: &[katana_core::workspace::TreeEntry],
    selected: &mut Option<std::path::PathBuf>,
    force: Option<bool>,
    depth: usize,
    active_path: Option<&std::path::Path>,
) {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
    // フルパスで ID を生成して同名ディレクトリの ID 衝突を防ぐ。
    let id = ui.make_persistent_id(format!("dir:{}", path.display()));
    let mut state =
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);
    if let Some(open) = force {
        state.set_open(open);
    }
    let is_open = state.is_open();

    let arrow = if is_open { "▼" } else { "▶" };
    let dir_icon = if is_open { "📂" } else { "📁" };
    let prefix = indent_prefix(depth);
    let label_text = format!("{prefix}{arrow} {dir_icon} {name}");
    let resp = ui.add(
        egui::Label::new(egui::RichText::new(label_text).color(egui::Color32::from_gray(220)))
            .truncate()
            .sense(egui::Sense::click()),
    );
    if resp.clicked() {
        state.set_open(!is_open);
    }
    state.store(ui.ctx());

    if state.is_open() {
        for child in children {
            render_tree_entry(ui, child, selected, force, depth + 1, active_path);
        }
    }
}

fn render_file_entry(
    ui: &mut egui::Ui,
    entry: &katana_core::workspace::TreeEntry,
    path: &std::path::Path,
    selected: &mut Option<std::path::PathBuf>,
    depth: usize,
    active_path: Option<&std::path::Path>,
) {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
    let prefix = indent_prefix(depth);
    let icon = if entry.is_markdown() { "📄" } else { "  " };
    let label = format!("{prefix}{icon} {name}");

    let is_active = active_path.is_some_and(|ap| ap == path);

    // アクティブファイルは背景色とテキスト色でハイライト表示
    let text_color = if is_active {
        egui::Color32::WHITE
    } else {
        egui::Color32::from_gray(220)
    };
    let rich = egui::RichText::new(&label).color(text_color);
    let rich = if is_active { rich.strong() } else { rich };

    let resp = ui.add(
        egui::Label::new(rich)
            .truncate()
            .sense(egui::Sense::click()),
    );

    // アクティブ行に背景を付ける（ラベル描画後にその rect を使って全幅背景を描画）
    if is_active {
        let full_rect = egui::Rect::from_min_max(
            egui::pos2(ui.min_rect().min.x, resp.rect.min.y),
            egui::pos2(ui.min_rect().max.x, resp.rect.max.y),
        );
        // VSCode 風のアクティブアイテム背景色（ただし文字の下に描画）
        let bg_color = egui::Color32::from_rgba_premultiplied(40, 80, 160, 100);
        ui.painter().rect_filled(full_rect, 3.0, bg_color);
    }

    if resp.clicked() && entry.is_markdown() {
        *selected = Some(path.to_path_buf());
    }
}
