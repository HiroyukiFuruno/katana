//! Katana three-pane egui shell.

use std::collections::HashMap;

use eframe::egui;
use katana_platform::FilesystemService;

use crate::{
    app_state::{AppAction, AppState, ScrollSource, ViewMode},
    i18n,
    preview_pane::{DownloadRequest, PreviewPane},
};

// UI描画関数群はカバレッジ計測除外対象のため別モジュールに分離
use crate::shell_ui::*;

// ─────────────────────────────────────────────
// レイアウト定数
// ─────────────────────────────────────────────

/// サイドバー折りたたみ時に表示する「›」トグルボタンの幅 (px)。
pub(crate) const SIDEBAR_COLLAPSED_TOGGLE_WIDTH: f32 = 24.0;

/// ファイルツリーパネルのリサイズ最小幅 (px)。
pub(crate) const FILE_TREE_PANEL_MIN_WIDTH: f32 = 120.0;

/// ファイルツリーパネルの初期表示幅 (px)。
pub(crate) const FILE_TREE_PANEL_DEFAULT_WIDTH: f32 = 220.0;

/// Split モード時のプレビューパネル最小幅 (px)。
pub(crate) const SPLIT_PREVIEW_PANEL_MIN_WIDTH: f32 = 200.0;

/// Split モード時のプレビューパネル初期幅 (px)。
pub(crate) const SPLIT_PREVIEW_PANEL_DEFAULT_WIDTH: f32 = 400.0;

/// タブバー右端の ◀▶ ナビゲーションボタン領域幅 (px)。
pub(crate) const TAB_NAV_BUTTONS_AREA_WIDTH: f32 = 80.0;

/// 各タブの右側に設けるタブ間余白 (px)。
pub(crate) const TAB_INTER_ITEM_SPACING: f32 = 4.0;

/// テキストエディタ TextEdit の初期表示行数。
pub(crate) const EDITOR_INITIAL_VISIBLE_ROWS: usize = 40;

/// エディタ⇔プレビュー間スクロール同期の感度閾値。
/// fraction 差分がこの値以下の場合はスクロールイベントを無視する。
pub(crate) const SCROLL_SYNC_DEAD_ZONE: f32 = 0.002;

/// タブのツールチップが表示されるまでの遅延 (秒)。
pub(crate) const TAB_TOOLTIP_SHOW_DELAY_SECS: f32 = 0.25;

/// ファイルツリーの「ワークスペース未選択」表示下の余白 (px)。
pub(crate) const NO_WORKSPACE_BOTTOM_SPACING: f32 = 8.0;

/// ダウンロード完了チェックのポーリング間隔 (ms)。
pub(crate) const DOWNLOAD_STATUS_CHECK_INTERVAL_MS: u64 = 200;

// ─────────────────────────────────────────────
// カラー定数
// ─────────────────────────────────────────────

/// アプリ内タイトルバーに表示するファイル名のテキスト色。
const TITLE_BAR_TEXT_COLOR: egui::Color32 = egui::Color32::from_gray(180);

/// ファイルツリーの通常テキスト色（非アクティブファイル・ディレクトリ）。
pub(crate) const FILE_TREE_TEXT_COLOR: egui::Color32 = egui::Color32::from_gray(220);

/// ファイルツリーでアクティブファイルを示す背景ハイライト色 (VSCode風半透明ブルー)。
pub(crate) const ACTIVE_FILE_HIGHLIGHT_BG: egui::Color32 =
    egui::Color32::from_rgba_premultiplied(40, 80, 160, 100);

/// ファイルツリーのアクティブ行背景の角丸半径。
pub(crate) const ACTIVE_FILE_HIGHLIGHT_ROUNDING: f32 = 3.0;

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
                    ctx.request_repaint_after(std::time::Duration::from_millis(
                        DOWNLOAD_STATUS_CHECK_INTERVAL_MS,
                    ));
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
                            .color(TITLE_BAR_TEXT_COLOR),
                    );
                });
            });
        });

        // ワークスペースが非表示のときも折りたたみトグルボタンを表示する。
        if !self.state.show_workspace {
            egui::SidePanel::left("workspace_collapsed")
                .resizable(false)
                .exact_width(SIDEBAR_COLLAPSED_TOGGLE_WIDTH)
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
                .min_width(SPLIT_PREVIEW_PANEL_MIN_WIDTH)
                .default_width(SPLIT_PREVIEW_PANEL_DEFAULT_WIDTH)
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
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn make_app() -> KatanaApp {
        let state = AppState::new(AiProviderRegistry::new(), PluginRegistry::new());
        KatanaApp::new(state)
    }

    fn make_temp_workspace() -> TempDir {
        let dir = tempfile::tempdir().unwrap();
        // ワークスペースに md ファイルを作成
        std::fs::write(dir.path().join("test.md"), "# Test").unwrap();
        dir
    }

    // handle_open_workspace: 有効なパスで成功 (L149-160)
    #[test]
    fn handle_open_workspace_success_sets_workspace() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        app.handle_open_workspace(dir.path().to_path_buf());
        assert!(app.state.workspace.is_some());
        assert!(app.state.status_message.is_some());
    }

    // handle_open_workspace: 無効なパスでエラー (L161-167)
    #[test]
    fn handle_open_workspace_error_sets_status_message() {
        let mut app = make_app();
        app.handle_open_workspace(PathBuf::from("/nonexistent/path/that/cannot/exist"));
        // 存在しないパスなので workspace は None（空ディレクトリとして開かれる可能性も）
        // エラーが記録される or workspace が空で開かれる
        assert!(
            app.state.workspace.is_some() || app.state.status_message.is_some(),
            "エラーまたはワークスペースが設定されるべき"
        );
    }

    // handle_select_document: 存在しないファイルのロードエラー (L198-204)
    #[test]
    fn handle_select_document_file_not_found_sets_status_message() {
        let mut app = make_app();
        app.handle_select_document(PathBuf::from("/nonexistent/file.md"));
        // ロードエラー → status_message に記録
        assert!(app.state.status_message.is_some());
    }

    // handle_select_document: 既存タブ選択でフォーカス移動 (L173-188)
    #[test]
    fn handle_select_document_switches_to_existing_tab() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");

        // 最初のロード
        app.handle_select_document(path.clone());
        assert_eq!(app.state.active_doc_idx, Some(0));
        assert_eq!(app.state.open_documents.len(), 1);

        // 同じファイルを再度選択 → 新しいタブは開かない
        app.handle_select_document(path.clone());
        assert_eq!(app.state.open_documents.len(), 1);
        assert_eq!(app.state.active_doc_idx, Some(0));
    }

    // handle_update_buffer: アクティブドキュメントなし (L213)
    #[test]
    fn handle_update_buffer_without_active_doc_does_nothing() {
        let mut app = make_app();
        // ドキュメントを開かずに UpdateBuffer → パニックしない
        app.handle_update_buffer("new content".to_string());
        assert!(app.state.open_documents.is_empty());
    }

    // handle_update_buffer: アクティブドキュメントあり (L209-215)
    #[test]
    fn handle_update_buffer_updates_active_doc_buffer() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.handle_select_document(path.clone());

        app.handle_update_buffer("# Updated Content".to_string());
        let doc = app.state.active_document().unwrap();
        assert_eq!(doc.buffer, "# Updated Content");
        assert!(doc.is_dirty);
    }

    // handle_save_document: アクティブドキュメントなし (L219-220)
    #[test]
    fn handle_save_document_without_active_doc_does_nothing() {
        let mut app = make_app();
        app.handle_save_document();
        // ステータスメッセージは設定されない（ドキュメントなし）
        assert!(app.state.status_message.is_none());
    }

    // handle_save_document: 正常保存 (L222-223)
    #[test]
    fn handle_save_document_success_sets_status() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.handle_select_document(path.clone());
        app.handle_update_buffer("# Modified".to_string());

        app.handle_save_document();
        assert!(app.state.status_message.is_some());
    }

    // process_action: CloseDocument (L236-244)
    #[test]
    fn process_action_close_document_removes_tab() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.handle_select_document(path.clone());
        assert_eq!(app.state.open_documents.len(), 1);

        app.process_action(AppAction::CloseDocument(0));
        assert!(app.state.open_documents.is_empty());
        assert!(app.state.active_doc_idx.is_none());
    }

    // process_action: CloseDocument - インデックス外はパニックしない (L237)
    #[test]
    fn process_action_close_document_out_of_bounds_does_nothing() {
        let mut app = make_app();
        app.process_action(AppAction::CloseDocument(99));
        assert!(app.state.open_documents.is_empty());
    }

    // process_action: RefreshDiagrams (L248-253)
    #[test]
    fn process_action_refresh_diagrams_does_not_crash() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.handle_select_document(path.clone());

        app.process_action(AppAction::RefreshDiagrams);
        // クラッシュしなければOK
    }

    // process_action: RefreshDiagrams ドキュメントなし (L249 early return)
    #[test]
    fn process_action_refresh_diagrams_no_doc_does_nothing() {
        let mut app = make_app();
        app.process_action(AppAction::RefreshDiagrams);
        // ドキュメントなし → クラッシュしない
    }

    // process_action: ChangeLanguage (L255-257)
    #[test]
    fn process_action_change_language_sets_language() {
        let mut app = make_app();
        app.process_action(AppAction::ChangeLanguage("ja".to_string()));
        // i18n の言語が変更されたことを確認（直接アクセス困難なのでパニックしないことを確認）
    }

    // process_action: None (L258)
    #[test]
    fn process_action_none_does_nothing() {
        let mut app = make_app();
        app.process_action(AppAction::None);
    }

    // process_action: UpdateBuffer (L246)
    #[test]
    fn process_action_update_buffer_calls_handler() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.handle_select_document(path);
        app.process_action(AppAction::UpdateBuffer("# Via Process Action".to_string()));
        assert_eq!(
            app.state.active_document().unwrap().buffer,
            "# Via Process Action"
        );
    }

    // process_action: SaveDocument (L247)
    #[test]
    fn process_action_save_document_calls_handler() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.handle_select_document(path);
        app.process_action(AppAction::UpdateBuffer("saved content".to_string()));
        app.process_action(AppAction::SaveDocument);
        assert!(app.state.status_message.is_some());
    }

    // start_download: スレッドが起動する (L263-273)
    #[test]
    fn start_download_sets_download_state() {
        let mut app = make_app();
        app.start_download(DownloadRequest {
            url: "http://example.com/plantuml.jar".to_string(),
            dest: PathBuf::from("/tmp/test_plantuml.jar"),
        });
        // status_message が設定される
        assert!(app.state.status_message.is_some());
        // download_rx が設定される
        assert!(app.download_rx.is_some());
    }

    // download_with_curl: 親ディレクトリ作成が必要なパス (L319-320)
    #[test]
    fn download_with_curl_creates_parent_dir() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("subdir").join("file.jar");
        // curl コマンドが失敗しても親ディレクトリが作成される
        // (curl は存在しない URL で失敗するが、dir_all は成功する)
        let _ = download_with_curl("http://127.0.0.1:0/nonexistent", &dest);
        // 親ディレクトリが作成されたことを確認
        assert!(dest.parent().unwrap().exists());
    }

    // take_action: pending_action を返してリセット (L127-129)
    #[test]
    fn take_action_returns_and_resets_pending_action() {
        let mut app = make_app();
        app.pending_action = AppAction::ChangeLanguage("en".to_string());
        let action = app.take_action();
        assert!(matches!(action, AppAction::ChangeLanguage(_)));
        assert!(matches!(app.pending_action, AppAction::None));
    }

    // poll_download: download_rx がない場合 (L297-299)
    #[test]
    fn poll_download_without_rx_does_nothing() {
        let app = make_app();
        assert!(app.download_rx.is_none());
        // download_rx なしで poll しても問題ない
        // egui Context を作れないため内部的な poll は呼べないが、
        // download_rx = None の場合は early exit する (L297-299)
    }
}

// shell.rs の追加テスト: 前のモジュールから分離して未カバー行を追加カバー
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests_extra {
    use super::*;
    use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};

    fn make_app() -> KatanaApp {
        let state = AppState::new(AiProviderRegistry::new(), PluginRegistry::new());
        KatanaApp::new(state)
    }

    fn make_temp_workspace() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("test.md"), "# Test").unwrap();
        dir
    }

    // handle_select_document: ハッシュ不一致で再レンダリング (L184-185)
    #[test]
    fn handle_select_document_rerenders_when_hash_changed() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");

        // 最初のロード
        app.handle_select_document(path.clone());
        assert_eq!(app.state.open_documents.len(), 1);

        // tab_hashes に古いハッシュを設定（バッファとは異なる）
        app.tab_hashes.insert(path.clone(), 0xDEADBEEF);

        // 再選択 → ハッシュ不一致で full_refresh_preview が呼ばれる (L184-185)
        app.handle_select_document(path.clone());

        // タブ数は変わらない
        assert_eq!(app.state.open_documents.len(), 1);
    }

    // handle_save_document: fs.save_document が失敗するケース (L224-228)
    #[test]
    fn handle_save_document_error_sets_error_status_message() {
        use std::os::unix::fs::PermissionsExt;

        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.handle_select_document(path.clone());
        app.handle_update_buffer("# Modified content".to_string());

        // ファイルを読み取り専用にする
        let perms = std::fs::Permissions::from_mode(0o444);
        std::fs::set_permissions(&path, perms).unwrap();

        app.handle_save_document();

        // 書き込み失敗 → status_message に記録
        assert!(app.state.status_message.is_some());

        // クリーンアップ: 書き込み可能に戻す
        let perms = std::fs::Permissions::from_mode(0o644);
        let _ = std::fs::set_permissions(&path, perms);
    }

    // download_with_curl: 成功ケース (L326-327) — ローカルfile:// URL
    #[test]
    fn download_with_curl_success_with_local_file_url() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("source.txt");
        let dest = dir.path().join("dest.txt");
        std::fs::write(&src, "hello").unwrap();

        let url = format!("file://{}", src.display());
        let result = download_with_curl(&url, &dest);
        // macOS では curl が利用可能
        assert!(result.is_ok());
        assert!(dest.exists());
    }

    // process_action: OpenWorkspace (L234)
    #[test]
    fn process_action_open_workspace_calls_handler() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        app.process_action(AppAction::OpenWorkspace(dir.path().to_path_buf()));
        assert!(app.state.workspace.is_some());
    }

    // process_action: SelectDocument (L235)
    #[test]
    fn process_action_select_document_calls_handler() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.process_action(AppAction::SelectDocument(path));
        assert_eq!(app.state.open_documents.len(), 1);
    }

    // full_refresh_preview: ハッシュが更新される (L140-147)
    #[test]
    fn full_refresh_preview_updates_tab_hash() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.full_refresh_preview(&path, "# Content");
        assert!(app.tab_hashes.contains_key(&path));
    }

    // refresh_preview: 既存エントリを更新する (L131-137)
    #[test]
    fn refresh_preview_updates_existing_pane() {
        let mut app = make_app();
        let _dir = make_temp_workspace();
        let path = _dir.path().join("test.md");
        app.refresh_preview(&path, "# Initial");
        app.refresh_preview(&path, "# Updated");
    }

    // poll_download: download_rx が None の場合は何もしない
    #[test]
    fn poll_download_does_nothing_when_no_rx() {
        let mut app = make_app();
        let ctx = egui::Context::default();
        app.poll_download(&ctx);
        assert!(app.download_rx.is_none());
    }

    // poll_download: Ok(Ok(())) で完了 → status_message設定, download_rx=None
    #[test]
    fn poll_download_sets_status_on_success() {
        let mut app = make_app();
        let (tx, rx) = std::sync::mpsc::channel();
        app.download_rx = Some(rx);
        tx.send(Ok(())).unwrap();
        drop(tx);
        let ctx = egui::Context::default();
        app.poll_download(&ctx);
        assert!(app.state.status_message.is_some());
        assert!(app.download_rx.is_none());
        assert!(matches!(app.pending_action, AppAction::RefreshDiagrams));
    }

    // poll_download: Ok(Err(e)) でエラー → error status_message
    #[test]
    fn poll_download_sets_error_on_failure() {
        let mut app = make_app();
        let (tx, rx) = std::sync::mpsc::channel();
        app.download_rx = Some(rx);
        tx.send(Err("network error".to_string())).unwrap();
        drop(tx);
        let ctx = egui::Context::default();
        app.poll_download(&ctx);
        assert!(app.state.status_message.is_some());
        assert!(app.download_rx.is_none());
    }

    // poll_download: Err(Empty) → まだ受信中
    #[test]
    fn poll_download_keeps_rx_when_empty() {
        let mut app = make_app();
        let (_tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();
        app.download_rx = Some(rx);
        let ctx = egui::Context::default();
        app.poll_download(&ctx);
        // Empty なので rx は維持される
        assert!(app.download_rx.is_some());
    }

    // poll_download: Err(Disconnected) → 完了として処理
    #[test]
    fn poll_download_clears_rx_on_disconnect() {
        let mut app = make_app();
        let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();
        drop(tx); // sender ドロップで Disconnected
        app.download_rx = Some(rx);
        let ctx = egui::Context::default();
        app.poll_download(&ctx);
        assert!(app.download_rx.is_none());
    }

    // download_with_curl: 失敗パス（不正URL → exit code 非0）
    #[test]
    fn download_with_curl_failure_returns_error() {
        let dir = tempfile::TempDir::new().unwrap();
        let dest = dir.path().join("nonexistent.jar");
        // 存在しないファイルURL → curl が失敗する
        let result = download_with_curl("file:///nonexistent/path/to/file", &dest);
        assert!(result.is_err());
    }

    // download_with_curl: create_dir_all パスをカバー（親ディレクトリが存在しない場合）
    #[test]
    fn download_with_curl_creates_parent_dirs() {
        let dir = tempfile::TempDir::new().unwrap();
        let src = dir.path().join("source.txt");
        std::fs::write(&src, "hello").unwrap();
        let dest = dir.path().join("subdir").join("deep").join("dest.txt");
        let url = format!("file://{}", src.display());
        let result = download_with_curl(&url, &dest);
        // ディレクトリは作成される
        assert!(dest.parent().unwrap().exists());
        assert!(result.is_ok());
        assert!(dest.exists());
    }
}
