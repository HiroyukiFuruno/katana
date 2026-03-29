#![allow(unused_imports)]
#![allow(dead_code)]
use crate::app::*;
use crate::shell::*;

use crate::preview_pane::{DownloadRequest, PreviewPane};
use crate::shell_logic::hash_str;
use katana_platform::FilesystemService;

use crate::app_state::*;
use std::ffi::OsStr;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

pub(crate) trait ActionOps {
    fn take_action(&mut self) -> AppAction;
    fn handle_toggle_task_list(&mut self, global_index: usize, new_state: char);
    fn cleanup_closed_tab_previews(&mut self);
    fn cancel_inactive_renders(&mut self);
    fn process_action(&mut self, ctx: &egui::Context, action: AppAction);
    fn handle_show_release_notes(&mut self);
    fn poll_changelog(&mut self, _ctx: &egui::Context);
    fn trigger_action(&mut self, action: AppAction);
    fn app_state_mut(&mut self) -> &mut AppState;
    fn new(state: AppState) -> Self;
    fn skip_splash(&mut self);
}

impl ActionOps for KatanaApp {
    fn take_action(&mut self) -> AppAction {
        std::mem::replace(&mut self.pending_action, AppAction::None)
    }
    fn handle_toggle_task_list(&mut self, global_index: usize, new_state: char) {
        let (path, content) = if let Some(doc) = self.state.active_document_mut() {
            let spans = egui_commonmark::extract_task_list_spans(&doc.buffer);
            if let Some(span) = spans.get(global_index) {
                let replacement = format!("[{}]", new_state);
                if span.start <= span.end && span.end <= doc.buffer.len() {
                    doc.buffer.replace_range(span.clone(), &replacement);
                    doc.is_dirty = true;
                }
            } else {
                tracing::warn!(
                    "Interactive Task List out of bounds: global_index {} vs {}",
                    global_index,
                    spans.len()
                );
            }
            (doc.path.clone(), doc.buffer.clone())
        } else {
            return;
        };
        self.refresh_preview(&path, &content);
    }
    fn cleanup_closed_tab_previews(&mut self) {
        let open_paths: std::collections::HashSet<_> =
            self.state.open_documents.iter().map(|d| &d.path).collect();
        self.tab_previews.retain(|t| open_paths.contains(&t.path));
    }
    fn cancel_inactive_renders(&mut self) {
        let active_path = self.state.active_document().map(|d| d.path.clone());
        for pane in &mut self.tab_previews {
            if Some(&pane.path) != active_path.as_ref() {
                pane.pane.abort_renders();
            }
        }
    }
    fn process_action(&mut self, ctx: &egui::Context, action: AppAction) {
        match action {
            AppAction::OpenWorkspace(p) => self.handle_open_workspace(p),
            AppAction::RefreshWorkspace => self.handle_refresh_workspace(),
            AppAction::SelectDocument(p) => self.handle_select_document(p, true),
            AppAction::OpenMultipleDocuments(paths) => {
                // When recursively opening a directory, activate the first file
                // and open the rest lazily (no load, no activate) in subsequent frames
                // to prevent UI freezing and show progressive tab increase.
                let mut iter = paths.into_iter();
                if let Some(first_path) = iter.next() {
                    self.handle_select_document(first_path, true);
                }
                for path in iter {
                    self.pending_document_loads.push_back(path);
                }
            }
            AppAction::RemoveWorkspace(path) => self.handle_remove_workspace(path),
            AppAction::CloseDocument(idx) => {
                // A1: If confirm_close_dirty_tab is enabled and the doc is dirty,
                // show a confirmation dialog instead of closing immediately.
                let should_confirm = self
                    .state
                    .settings
                    .settings()
                    .behavior
                    .confirm_close_dirty_tab
                    && idx < self.state.open_documents.len()
                    && self.state.open_documents[idx].is_dirty;

                if should_confirm {
                    self.state.pending_close_confirm = Some(idx);
                } else {
                    self.force_close_document(idx);
                }
            }
            AppAction::ForceCloseDocument(idx) => {
                self.state.pending_close_confirm = None;
                self.force_close_document(idx);
            }
            AppAction::UpdateBuffer(c) => self.handle_update_buffer(c),
            AppAction::ReplaceText { span, replacement } => {
                self.handle_replace_text(span, replacement)
            }
            AppAction::ToggleTaskList {
                global_index,
                new_state,
            } => self.handle_toggle_task_list(global_index, new_state),
            AppAction::SaveDocument => self.handle_save_document(),
            AppAction::RefreshDiagrams => {
                // Clear all egui texture caches (e.g. diagrams, network images)
                ctx.forget_all_images();
                // Reinstall UI icon SVGs so they aren't lost to the cache clear
                crate::icon::IconRegistry::install(ctx);

                // Invalidate hashes so non-active tabs re-render on next switch
                for tab in &mut self.tab_previews {
                    tab.hash = 0;
                    for viewer in tab.pane.viewer_states.iter_mut() {
                        viewer.texture = None;
                    }
                    tab.pane.fullscreen_viewer_state.texture = None;
                }

                // Re-render the active tab without reading from disk, to avoid wiping unsaved changes!
                if let Some(doc) = self.state.active_document_mut() {
                    let path = doc.path.clone();
                    let src = doc.buffer.clone();
                    let concurrency = self
                        .state
                        .settings
                        .settings()
                        .performance
                        .diagram_concurrency;
                    self.full_refresh_preview(&path, &src, true, concurrency);
                }
            }
            AppAction::ChangeLanguage(lang) => {
                crate::i18n::set_language(&lang);
                crate::shell_ui::update_native_menu_strings_from_i18n();
                self.state.settings.settings_mut().language = lang;
                if let Err(e) = self.state.settings.save() {
                    tracing::warn!("Failed to save settings: {e}");
                }
            }
            AppAction::ToggleSettings => {
                self.state.show_settings = !self.state.show_settings;
            }
            AppAction::ToggleAbout => {
                self.show_about = !self.show_about;
            }
            AppAction::ToggleToc => {
                self.state.show_toc = !self.state.show_toc;
            }
            AppAction::SetSplitDirection(dir) => {
                // Keep toolbar toggles temporary and scoped to the active tab.
                self.state.set_active_split_direction(dir);
            }
            AppAction::SetPaneOrder(order) => {
                // Keep toolbar toggles temporary and scoped to the active tab.
                self.state.set_active_pane_order(order);
            }
            AppAction::CloseOtherDocuments(idx) => {
                if idx < self.state.open_documents.len() {
                    let mut keep = Vec::new();
                    let old_docs = std::mem::take(&mut self.state.open_documents);
                    for (i, doc) in old_docs.into_iter().enumerate() {
                        if i == idx {
                            keep.push(doc);
                        } else {
                            self.state.push_recently_closed(doc.path);
                        }
                    }
                    self.state.open_documents = keep;
                    self.state.active_doc_idx = Some(0);
                }
                self.save_workspace_state();
            }
            AppAction::CloseAllDocuments => {
                let old_docs = std::mem::take(&mut self.state.open_documents);
                for doc in old_docs.into_iter() {
                    self.state.push_recently_closed(doc.path);
                }
                self.state.active_doc_idx = None;
                self.save_workspace_state();
                self.cleanup_closed_tab_previews();
            }
            AppAction::CloseDocumentsToRight(idx) => {
                let mut keep = Vec::new();
                let old_docs = std::mem::take(&mut self.state.open_documents);
                for (i, doc) in old_docs.into_iter().enumerate() {
                    if i <= idx {
                        keep.push(doc);
                    } else {
                        self.state.push_recently_closed(doc.path);
                    }
                }
                self.state.open_documents = keep;
                if let Some(a_idx) = self.state.active_doc_idx {
                    if a_idx > idx {
                        self.state.active_doc_idx = Some(idx);
                    }
                }
                self.save_workspace_state();
                self.cleanup_closed_tab_previews();
            }
            AppAction::CloseDocumentsToLeft(idx) => {
                let mut keep = Vec::new();
                let new_active_idx = self.state.active_doc_idx;
                let old_docs = std::mem::take(&mut self.state.open_documents);
                for (i, doc) in old_docs.into_iter().enumerate() {
                    if i >= idx {
                        keep.push(doc);
                    } else {
                        self.state.push_recently_closed(doc.path);
                    }
                }
                self.state.open_documents = keep;
                if let Some(a_idx) = new_active_idx {
                    if a_idx < idx {
                        self.state.active_doc_idx = Some(0);
                    } else {
                        self.state.active_doc_idx = Some(a_idx - idx);
                    }
                }
                self.save_workspace_state();
                self.cleanup_closed_tab_previews();
            }
            AppAction::TogglePinDocument(idx) => {
                if idx < self.state.open_documents.len() {
                    let active_path = self.state.active_document().map(|d| d.path.clone());
                    let doc = &mut self.state.open_documents[idx];
                    doc.is_pinned = !doc.is_pinned;
                    // Stable sort to move pinned tabs to the front
                    self.state.open_documents.sort_by_key(|d| !d.is_pinned);
                    if let Some(path) = active_path {
                        if let Some(new_idx) = self
                            .state
                            .open_documents
                            .iter()
                            .position(|d| d.path == path)
                        {
                            self.state.active_doc_idx = Some(new_idx);
                        }
                    }
                }
                self.save_workspace_state();
            }
            AppAction::RestoreClosedDocument => {
                if let Some(path) = self.state.recently_closed_tabs.pop_back() {
                    self.handle_select_document(path, true);
                }
            }
            AppAction::ReorderDocument { from, to } => {
                let len = self.state.open_documents.len();
                if from < len && to <= len && from != to {
                    let active_path = self.state.active_document().map(|d| d.path.clone());
                    let doc = self.state.open_documents.remove(from);
                    let actual_to = if to > from { to - 1 } else { to };
                    self.state.open_documents.insert(actual_to, doc);
                    if let Some(path) = active_path {
                        if let Some(new_idx) = self
                            .state
                            .open_documents
                            .iter()
                            .position(|d| d.path == path)
                        {
                            self.state.active_doc_idx = Some(new_idx);
                        }
                    }
                }
                self.save_workspace_state();
            }
            AppAction::CheckForUpdates => {
                self.start_update_check(true);
            }
            AppAction::ExportDocument(fmt) => {
                self.handle_export_document(ctx, fmt);
            }
            AppAction::AcceptTerms(version) => {
                self.state.settings.settings_mut().terms_accepted_version = Some(version);
                if let Err(e) = self.state.settings.save() {
                    tracing::warn!("Failed to save terms acceptance: {e}");
                }
            }
            AppAction::DeclineTerms => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            AppAction::ShowMetaInfo(path) => {
                self.show_meta_info_for = Some(path);
            }
            AppAction::SkipVersion(version) => {
                self.state.settings.settings_mut().updates.skipped_version = Some(version);
                let _ = self.state.settings.save();
                self.show_update_dialog = false;
            }
            AppAction::DismissUpdate => {
                self.show_update_dialog = false;
            }
            AppAction::ConfirmRelaunch => {
                if let Some(_prep) = self.pending_relaunch.take() {
                    #[cfg(all(not(test), not(coverage)))]
                    {
                        let _ = katana_core::update::execute_relauncher(_prep);
                        std::process::exit(0);
                    }
                }
            }
            AppAction::ShowReleaseNotes => {
                self.handle_show_release_notes();
            }
            AppAction::ClearAllCaches => {
                use egui::load::BytesLoader;
                katana_platform::cache::DefaultCacheService::clear_all_directories();
                crate::http_cache_loader::PersistentHttpLoader::default().forget_all();
                ctx.forget_all_images();
                crate::icon::IconRegistry::install(ctx);

                // Assuming `clear_http_cache` is a decent message for "Successfully cleared" or "Clear cache" button label. Use it as Status msg.
                self.state.status_message = Some((
                    crate::i18n::get()
                        .settings
                        .behavior
                        .clear_http_cache
                        .clone(),
                    crate::app_state::StatusType::Success,
                ));
            }
            AppAction::RequestNewFile(path) => {
                let ext = self
                    .state
                    .settings
                    .settings()
                    .workspace
                    .visible_extensions
                    .first()
                    .cloned();
                self.state.create_fs_node_modal_state = Some((path, String::new(), ext, false));
            }
            AppAction::RequestNewDirectory(path) => {
                self.state.create_fs_node_modal_state = Some((path, String::new(), None, true));
            }
            AppAction::RequestRename(path) => {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                self.state.rename_modal_state = Some((path, name));
            }
            AppAction::RequestDelete(path) => {
                self.state.delete_modal_state = Some(path);
            }
            AppAction::CopyPathToClipboard(path) => {
                ctx.copy_text(path.to_string_lossy().to_string());
            }
            AppAction::CopyRelativePathToClipboard(path) => {
                let rel_path = if let Some(ws) = &self.state.workspace {
                    path.strip_prefix(&ws.root).unwrap_or(&path).to_path_buf()
                } else {
                    path.clone()
                };
                ctx.copy_text(rel_path.to_string_lossy().to_string());
            }
            AppAction::RevealInOs(path) => {
                #[cfg(target_os = "macos")]
                {
                    let _ = std::process::Command::new("open")
                        .arg("-R")
                        .arg(&path)
                        .spawn();
                }
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("explorer")
                        .arg("/select,")
                        .arg(&path)
                        .spawn();
                }
                #[cfg(target_os = "linux")]
                {
                    let dir = if path.is_file() {
                        path.parent().unwrap_or(&path)
                    } else {
                        &path
                    };
                    let _ = std::process::Command::new("xdg-open").arg(dir).spawn();
                }
            }
            AppAction::None => {}
            AppAction::InstallUpdate => {
                if let Some(release) = &self.state.update_available {
                    self.state.checking_for_updates = true;
                    self.state.update_phase =
                        Some(crate::app_state::UpdatePhase::Downloading { progress: 0.0 });
                    let exe_path = std::env::current_exe().unwrap();
                    let target_app_path = if exe_path.to_string_lossy().contains("MacOS") {
                        const MACOS_BUNDLE_LEVELS: usize = 3;
                        exe_path
                            .ancestors()
                            .nth(MACOS_BUNDLE_LEVELS)
                            .unwrap()
                            .to_path_buf()
                    } else {
                        exe_path.clone()
                    };

                    let download_url = release.download_url.clone();
                    let (tx, rx) = std::sync::mpsc::channel();
                    self.update_install_rx = Some(rx);

                    std::thread::spawn(move || {
                        let tx_clone = tx.clone();
                        let res = katana_core::update::prepare_update(
                            &download_url,
                            &target_app_path,
                            move |progress| {
                                let _ = tx_clone.send(UpdateInstallEvent::Progress(progress));
                            },
                        )
                        .map_err(|e| e.to_string());
                        let _ = tx.send(UpdateInstallEvent::Finished(res));
                    });
                }
            }
        }

        // Clean up resources whenever an action completes.
        // Specifically, ensure inactive tabs give up their background CPU,
        // and closed tabs drop their previews entirely.
        self.cleanup_closed_tab_previews();

        // Ensure the newly active document is loaded and has a preview.
        // This handles cases where tabs are closed and the active tab silently shifts.
        let mut inactive_but_focused_path = None;
        if let Some(active_idx) = self.state.active_doc_idx {
            if let Some(doc) = self.state.open_documents.get(active_idx) {
                let has_preview = self.tab_previews.iter().any(|t| t.path == doc.path);
                if !doc.is_loaded || !has_preview {
                    inactive_but_focused_path = Some(doc.path.clone());
                }
            }
        }
        if let Some(path) = inactive_but_focused_path {
            self.handle_select_document(path, true);
        }

        self.cancel_inactive_renders();
    }
    fn handle_show_release_notes(&mut self) {
        let current_version = env!("CARGO_PKG_VERSION").to_string();
        let previous = self.old_app_version.clone().or_else(|| {
            self.state
                .settings
                .settings()
                .updates
                .previous_app_version
                .clone()
        });
        let lang = self.state.settings.settings().language.clone();

        let (tx, rx) = std::sync::mpsc::channel();
        self.changelog_rx = Some(rx);

        crate::changelog::fetch_changelog(&lang, current_version, previous, tx);
        tracing::info!("Triggered ShowReleaseNotes background fetch.");

        // Open/select the changelog tab immediately (it will show a loading state until data arrives)
        let virtual_path =
            std::path::PathBuf::from(format!("Katana://ChangeLog v{}", env!("CARGO_PKG_VERSION")));
        if !self
            .state
            .open_documents
            .iter()
            .any(|d| d.path == virtual_path)
        {
            self.state
                .open_documents
                .push(katana_core::document::Document::new_empty(
                    virtual_path.clone(),
                ));
        }
        self.handle_select_document(virtual_path, true);
    }
    fn poll_changelog(&mut self, _ctx: &egui::Context) {
        if let Some(rx) = &self.changelog_rx {
            if let Ok(event) = rx.try_recv() {
                self.changelog_rx = None;
                match event {
                    crate::changelog::ChangelogEvent::Success(sections) => {
                        self.changelog_sections = sections;
                        let virtual_path = std::path::PathBuf::from(format!(
                            "Katana://ChangeLog v{}",
                            env!("CARGO_PKG_VERSION")
                        ));
                        if let Some(pos) = self
                            .state
                            .open_documents
                            .iter()
                            .position(|d| d.path == virtual_path)
                        {
                            self.state.active_doc_idx = Some(pos);
                        } else {
                            self.state
                                .open_documents
                                .push(katana_core::document::Document::new_empty(virtual_path));
                            self.state.active_doc_idx = Some(self.state.open_documents.len() - 1);
                        }
                    }
                    crate::changelog::ChangelogEvent::Error(err) => {
                        tracing::error!("Failed to fetch changelog: {}", err);
                        self.state.status_message = Some((
                            format!("Failed to fetch release notes: {err}"),
                            crate::app_state::StatusType::Error,
                        ));
                    }
                }
            }
        }
    }
    fn trigger_action(&mut self, action: AppAction) {
        self.pending_action = action;
    }
    fn app_state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }
    fn new(state: AppState) -> Self {
        let mut app = Self {
            state,
            fs: FilesystemService::new(),
            pending_action: AppAction::None,
            tab_previews: Vec::new(),
            download_rx: None,
            workspace_rx: None,
            update_rx: None,
            changelog_rx: None,
            update_install_rx: None,
            export_tasks: Vec::new(),
            pending_document_loads: std::collections::VecDeque::new(),
            show_about: false,
            show_update_dialog: false,
            update_markdown_cache: egui_commonmark::CommonMarkCache::default(),
            update_notified: false,
            about_icon: None,
            cached_theme: None,
            cached_font_size: None,
            cached_font_family: None,
            settings_preview: PreviewPane::default(),
            needs_splash: !cfg!(test),
            splash_start: None,
            show_meta_info_for: None,
            pending_relaunch: None,
            changelog_sections: Vec::new(),
            needs_changelog_display: false,
            old_app_version: None,
        };
        let current_version = env!("CARGO_PKG_VERSION");
        let mut show_changelog = false;

        {
            let settings_mut = app.state.settings.settings_mut();
            if let Some(prev) = &settings_mut.updates.previous_app_version {
                app.old_app_version = Some(prev.clone());
                if prev != current_version {
                    show_changelog = true;
                }
            } else {
                // First launch ever or first launch since v0.8.0
                show_changelog = true;
            }
            if show_changelog {
                settings_mut.updates.previous_app_version = Some(current_version.to_string());
            }
        }

        if show_changelog {
            if let Err(e) = app.state.settings.save() {
                tracing::warn!("Failed to save previous_app_version: {e}");
            }
            app.needs_changelog_display = true;
        }

        app.start_update_check(false);
        app
    }
    fn skip_splash(&mut self) {
        self.needs_splash = false;
        self.splash_start = None;
    }
}
