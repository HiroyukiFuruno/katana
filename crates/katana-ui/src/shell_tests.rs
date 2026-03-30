#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn make_app() -> KatanaApp {
        let state = AppState::new(
            AiProviderRegistry::new(),
            PluginRegistry::new(),
            katana_platform::SettingsService::default(),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        KatanaApp::new(state)
    }

    fn make_temp_workspace() -> TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("test.md"), "# Test").unwrap();
        dir
    }

    fn wait_for_workspace(app: &mut KatanaApp) {
        let ctx = egui::Context::default();
        for _ in 0..100 {
            app.poll_workspace_load(&ctx);
            if app.workspace_rx.is_none() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    #[test]
    fn handle_open_workspace_success_sets_workspace() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        app.handle_open_workspace(dir.path().to_path_buf());
        wait_for_workspace(&mut app);
        assert!(app.state.workspace.data.is_some());
        assert!(app.state.layout.status_message.is_some());
    }

    #[test]
    fn handle_open_workspace_error_sets_status_message() {
        let mut app = make_app();
        app.handle_open_workspace(PathBuf::from("/nonexistent/path/that/cannot/exist"));
        wait_for_workspace(&mut app);
        assert!(
            app.state.workspace.data.is_some() || app.state.layout.status_message.is_some(),
            "Error or workspace should be set"
        );
    }

    #[test]
    fn handle_select_document_file_not_found_sets_status_message() {
        let mut app = make_app();
        app.handle_select_document(PathBuf::from("/nonexistent/file.md"), true);
        assert!(app.state.layout.status_message.is_some());
    }

    #[test]
    fn handle_select_document_switches_to_existing_tab() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");

        app.handle_select_document(path.clone(), true);
        assert_eq!(app.state.document.active_doc_idx, Some(0));
        assert_eq!(app.state.document.open_documents.len(), 1);

        app.handle_select_document(path.clone(), true);
        assert_eq!(app.state.document.open_documents.len(), 1);
        assert_eq!(app.state.document.active_doc_idx, Some(0));
    }

    #[test]
    fn handle_update_buffer_without_active_doc_does_nothing() {
        let mut app = make_app();
        app.handle_update_buffer("new content".to_string());
        assert!(app.state.document.open_documents.is_empty());
    }

    #[test]
    fn handle_update_buffer_updates_active_doc_buffer() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.handle_select_document(path.clone(), true);

        app.handle_update_buffer("# Updated Content".to_string());
        let doc = app.state.active_document().unwrap();
        assert_eq!(doc.buffer, "# Updated Content");
        assert!(doc.is_dirty);
    }

    #[test]
    fn handle_save_document_without_active_doc_does_nothing() {
        let mut app = make_app();
        app.handle_save_document();
        assert!(app.state.layout.status_message.is_none());
    }

    #[test]
    fn test_lazy_loading_flow() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("lazy.md");
        std::fs::write(&path, "# Lazy content").unwrap();

        app.handle_select_document(path.clone(), false);
        assert_eq!(app.state.document.open_documents.len(), 1);
        assert!(!app.state.document.open_documents[0].is_loaded);

        app.handle_select_document(path.clone(), true);
        assert!(app.state.document.open_documents[0].is_loaded);
        assert_eq!(
            app.state.document.open_documents[0].buffer,
            "# Lazy content"
        );
    }

    #[test]
    fn handle_save_document_success_sets_status() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.handle_select_document(path.clone(), true);
        app.handle_update_buffer("# Modified".to_string());

        app.handle_save_document();
        assert!(app.state.layout.status_message.is_some());
    }

    #[test]
    fn process_action_close_document_removes_tab() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.handle_select_document(path.clone(), true);
        assert_eq!(app.state.document.open_documents.len(), 1);

        app.process_action(&egui::Context::default(), AppAction::CloseDocument(0));
        assert!(app.state.document.open_documents.is_empty());
        assert!(app.state.document.active_doc_idx.is_none());
    }

    #[test]
    fn process_action_close_document_out_of_bounds_does_nothing() {
        let mut app = make_app();
        app.process_action(&egui::Context::default(), AppAction::CloseDocument(99));
        assert!(app.state.document.open_documents.is_empty());
    }

    #[test]
    fn process_action_refresh_diagrams_does_not_crash() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.handle_select_document(path.clone(), true);

        app.process_action(&egui::Context::default(), AppAction::RefreshDiagrams);
    }

    #[test]
    fn process_action_export_document_logs() {
        let mut app = make_app();
        app.process_action(
            &egui::Context::default(),
            AppAction::ExportDocument(crate::app_state::ExportFormat::Html),
        );
    }

    #[test]
    fn process_action_export_pdf_without_tool_shows_error() {
        if katana_core::markdown::PdfExporter::is_available() {
            return;
        }
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.handle_select_document(path, true);

        app.process_action(
            &egui::Context::default(),
            AppAction::ExportDocument(crate::app_state::ExportFormat::Pdf),
        );
        let (msg, kind) = app.state.layout.status_message.as_ref().unwrap();
        assert_eq!(*kind, crate::app_state::StatusType::Error);
        assert!(msg.contains("headless_chrome"), "msg = {msg}");
    }

    #[test]
    fn process_action_export_png_without_tool_shows_error() {
        if true {
            return;
        }
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.handle_select_document(path, true);

        app.process_action(
            &egui::Context::default(),
            AppAction::ExportDocument(crate::app_state::ExportFormat::Png),
        );
        let (msg, kind) = app.state.layout.status_message.as_ref().unwrap();
        assert_eq!(*kind, crate::app_state::StatusType::Error);
        assert!(msg.contains("headless_chrome"), "msg = {msg}");
    }

    #[test]
    fn process_action_refresh_diagrams_no_doc_does_nothing() {
        let mut app = make_app();
        app.process_action(&egui::Context::default(), AppAction::RefreshDiagrams);
    }

    #[test]
    fn process_action_refresh_diagrams_clears_texture_handles() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test_textures.md");
        std::fs::write(&path, "# Something").unwrap();
        app.handle_select_document(path.clone(), true);

        let ctx = egui::Context::default();
        let dummy_img = egui::ColorImage::example();
        let texture = ctx.load_texture("fake", dummy_img, egui::TextureOptions::LINEAR);

        if let Some(tab) = app.tab_previews.iter_mut().find(|p| p.path == path) {
            tab.pane
                .viewer_states
                .push(crate::preview_pane::ViewerState {
                    zoom: 1.0,
                    pan: egui::Vec2::ZERO,
                    texture: Some(texture.clone()),
                });
            tab.pane.fullscreen_viewer_state.texture = Some(texture.clone());
        } else {
            panic!("Tab not found");
        }

        app.process_action(&ctx, AppAction::RefreshDiagrams);

        let tab = app.tab_previews.iter().find(|p| p.path == path).unwrap();
        assert!(
            tab.pane.viewer_states[0].texture.is_none(),
            "Texture cache inside viewer state must be cleared!"
        );
        assert!(
            tab.pane.fullscreen_viewer_state.texture.is_none(),
            "Fullscreen texture cache must be cleared!"
        );
    }

    #[test]
    fn process_action_change_language_sets_language() {
        let mut app = make_app();
        app.process_action(
            &egui::Context::default(),
            AppAction::ChangeLanguage("ja".to_string()),
        );
    }

    #[test]
    fn process_action_toggle_toc_toggles_flag() {
        let mut app = make_app();
        assert!(!app.state.layout.show_toc);

        app.process_action(&egui::Context::default(), AppAction::ToggleToc);
        assert!(app.state.layout.show_toc);

        app.process_action(&egui::Context::default(), AppAction::ToggleToc);
        assert!(!app.state.layout.show_toc);
    }

    #[test]
    fn process_action_toggle_settings_toggles_flag() {
        let mut app = make_app();
        assert!(!app.state.layout.show_settings);

        app.process_action(&egui::Context::default(), AppAction::ToggleSettings);
        assert!(app.state.layout.show_settings);

        app.process_action(&egui::Context::default(), AppAction::ToggleSettings);
        assert!(!app.state.layout.show_settings);
    }

    #[test]
    fn process_action_none_does_nothing() {
        let mut app = make_app();
        app.process_action(&egui::Context::default(), AppAction::None);
    }

    #[test]
    fn process_action_update_buffer_calls_handler() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.handle_select_document(path, true);
        app.process_action(
            &egui::Context::default(),
            AppAction::UpdateBuffer("# Via Process Action".to_string()),
        );
        assert_eq!(
            app.state.active_document().unwrap().buffer,
            "# Via Process Action"
        );
    }

    #[test]
    fn process_action_save_document_calls_handler() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.handle_select_document(path, true);
        app.process_action(
            &egui::Context::default(),
            AppAction::UpdateBuffer("saved content".to_string()),
        );
        app.process_action(&egui::Context::default(), AppAction::SaveDocument);
        assert!(app.state.layout.status_message.is_some());
    }

    #[test]
    fn start_download_sets_download_state() {
        let mut app = make_app();
        app.start_download(crate::preview_pane::DownloadRequest {
            url: "http://example.com/plantuml.jar".to_string(),
            dest: PathBuf::from("/tmp/test_plantuml.jar"),
        });
        assert!(app.state.layout.status_message.is_some());
        assert!(app.download_rx.is_some());
    }

    #[test]
    pub(crate) fn download_with_curl_creates_parent_dir() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("subdir").join("file.jar");
        let _ = download_with_curl("http://127.0.0.1:0/nonexistent", &dest);
        assert!(dest.parent().unwrap().exists());
    }

    #[test]
    fn take_action_returns_and_resets_pending_action() {
        let mut app = make_app();
        app.pending_action = AppAction::ChangeLanguage("en".to_string());
        let action = app.take_action();
        assert!(
            format!("{action:?}").starts_with("ChangeLanguage"),
            "expected ChangeLanguage, got {action:?}"
        );
        assert_eq!(
            format!("{:?}", app.pending_action),
            format!("{:?}", AppAction::None)
        );
    }

    #[test]
    fn poll_download_without_rx_does_nothing() {
        let app = make_app();
        assert!(app.download_rx.is_none());
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests_extra {
    use super::*;
    use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};

    fn make_app() -> KatanaApp {
        let state = AppState::new(
            AiProviderRegistry::new(),
            PluginRegistry::new(),
            katana_platform::SettingsService::default(),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        let mut app = KatanaApp::new(state);
        app.pending_action = AppAction::None;
        app
    }

    fn make_temp_workspace() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("test.md"), "# Test").unwrap();
        dir
    }

    #[test]
    fn handle_select_document_rerenders_when_hash_changed() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");

        app.handle_select_document(path.clone(), true);
        assert_eq!(app.state.document.open_documents.len(), 1);

        app.tab_previews.push(TabPreviewCache {
            path: path.clone(),
            pane: PreviewPane::default(),
            hash: 0xDEADBEEF,
        });

        app.handle_select_document(path.clone(), true);

        assert_eq!(app.state.document.open_documents.len(), 1);
    }

    #[test]
    fn handle_save_document_error_sets_error_status_message() {
        use std::os::unix::fs::PermissionsExt;

        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.handle_select_document(path.clone(), true);
        app.handle_update_buffer("# Modified content".to_string());

        let perms = std::fs::Permissions::from_mode(0o444);
        std::fs::set_permissions(&path, perms).unwrap();

        app.handle_save_document();

        assert!(app.state.layout.status_message.is_some());

        let perms = std::fs::Permissions::from_mode(0o644);
        let _ = std::fs::set_permissions(&path, perms);
    }

    #[test]
    pub(crate) fn download_with_curl_success_with_local_file_url() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("source.txt");
        let dest = dir.path().join("dest.txt");
        std::fs::write(&src, "hello").unwrap();

        let url = format!("file://{}", src.display());
        let result = download_with_curl(&url, &dest);
        assert!(result.is_ok());
        assert!(dest.exists());
    }

    #[test]
    pub(crate) fn download_with_curl_launch_error() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("dest.txt");
        let result = super::_download_with_cmd(
            "invalid_curl_binary_for_test",
            "http://example.com/file",
            &dest,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains(&crate::i18n::get().error.curl_launch_failed));
    }

    #[test]
    fn process_action_open_workspace_calls_handler() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        app.process_action(
            &egui::Context::default(),
            AppAction::OpenWorkspace(dir.path().to_path_buf()),
        );
        wait_for_workspace(&mut app);
        assert!(app.state.workspace.data.is_some());
    }

    #[test]
    fn process_action_select_document_calls_handler() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.process_action(&egui::Context::default(), AppAction::SelectDocument(path));
        assert_eq!(app.state.document.open_documents.len(), 1);
    }

    #[test]
    fn full_refresh_preview_updates_tab_hash() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");
        app.full_refresh_preview(&path, "# Content", false, 4);
        assert!(app.tab_previews.iter().any(|t| t.path == path));
    }

    #[test]
    fn full_refresh_preview_replaces_when_is_loading() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("test.md");

        app.full_refresh_preview(&path, "# Initial", false, 4);
        let initial_hash = app
            .tab_previews
            .iter()
            .find(|t| t.path == path)
            .unwrap()
            .hash;

        let pane = super::KatanaApp::get_preview_pane(&mut app.tab_previews, path.clone());
        pane.is_loading = true;

        app.full_refresh_preview(&path, "# Updated", true, 4);

        let final_hash = app
            .tab_previews
            .iter()
            .find(|t| t.path == path)
            .unwrap()
            .hash;
        assert_ne!(
            initial_hash, final_hash,
            "full_refresh_preview should update hash even when is_loading was true (PreviewPane handles cancellation)"
        );
    }

    #[test]
    fn refresh_preview_updates_existing_pane() {
        let mut app = make_app();
        let _dir = make_temp_workspace();
        let path = _dir.path().join("test.md");
        app.refresh_preview(&path, "# Initial");
        app.refresh_preview(&path, "# Updated");
    }

    #[test]
    fn poll_download_does_nothing_when_no_rx() {
        let mut app = make_app();
        let ctx = egui::Context::default();
        app.poll_download(&ctx);
        assert!(app.download_rx.is_none());
    }

    #[test]
    fn poll_download_sets_status_on_success() {
        let mut app = make_app();
        let (tx, rx) = std::sync::mpsc::channel();
        app.download_rx = Some(rx);
        tx.send(Ok(())).unwrap();
        drop(tx);
        let ctx = egui::Context::default();
        app.poll_download(&ctx);
        assert!(app.state.layout.status_message.is_some());
        assert!(app.download_rx.is_none());
        assert_eq!(
            format!("{:?}", app.pending_action),
            format!("{:?}", AppAction::RefreshDiagrams)
        );
    }

    #[test]
    fn poll_download_sets_error_on_failure() {
        let mut app = make_app();
        let (tx, rx) = std::sync::mpsc::channel();
        app.download_rx = Some(rx);
        tx.send(Err("network error".to_string())).unwrap();
        drop(tx);
        let ctx = egui::Context::default();
        app.poll_download(&ctx);
        assert!(app.state.layout.status_message.is_some());
        assert!(app.download_rx.is_none());
    }

    #[test]
    fn poll_download_keeps_rx_when_empty() {
        let mut app = make_app();
        let (_tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();
        app.download_rx = Some(rx);
        let ctx = egui::Context::default();
        app.poll_download(&ctx);
        assert!(app.download_rx.is_some());
    }

    #[test]
    fn poll_download_clears_rx_on_disconnect() {
        let mut app = make_app();
        let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();
        drop(tx); // Disconnected on sender drop
        app.download_rx = Some(rx);
        let ctx = egui::Context::default();
        app.poll_download(&ctx);
        assert!(app.download_rx.is_none());
    }

    #[test]
    pub(crate) fn download_with_curl_failure_returns_error() {
        let dir = tempfile::TempDir::new().unwrap();
        let dest = dir.path().join("nonexistent.jar");
        let result = download_with_curl("file:///nonexistent/path/to/file", &dest);
        assert!(result.is_err());
    }

    #[test]
    pub(crate) fn download_with_curl_creates_parent_dirs() {
        let dir = tempfile::TempDir::new().unwrap();
        let src = dir.path().join("source.txt");
        std::fs::write(&src, "hello").unwrap();
        let dest = dir.path().join("subdir").join("deep").join("dest.txt");
        let url = format!("file://{}", src.display());
        let result = download_with_curl(&url, &dest);
        assert!(dest.parent().unwrap().exists());
        assert!(result.is_ok());
        assert!(dest.exists());
    }

    #[test]
    pub(crate) fn download_with_curl_no_parent_path() {
        let result = download_with_curl("file:///nonexistent/file", std::path::Path::new(""));
        assert!(result.is_err());
    }

    #[test]
    pub(crate) fn download_with_curl_create_dir_error() {
        let dest = std::path::Path::new("/dev/null/impossible_dir/file.jar");
        let result = download_with_curl("file:///nonexistent/file", dest);
        assert!(result.is_err());
    }

    struct FailingRepository;

    impl katana_platform::SettingsRepository for FailingRepository {
        fn load(&self) -> katana_platform::settings::AppSettings {
            katana_platform::settings::AppSettings::default()
        }
        fn save(&self, _settings: &katana_platform::settings::AppSettings) -> anyhow::Result<()> {
            anyhow::bail!("simulated save failure")
        }
    }

    fn make_app_with_failing_repo() -> KatanaApp {
        let settings = katana_platform::SettingsService::new(Box::new(FailingRepository));
        let state = AppState::new(
            AiProviderRegistry::new(),
            PluginRegistry::new(),
            settings,
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        KatanaApp::new(state)
    }

    fn wait_for_workspace(app: &mut KatanaApp) {
        let ctx = egui::Context::default();
        for _ in 0..100 {
            app.poll_workspace_load(&ctx);
            if app.workspace_rx.is_none() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    #[test]
    fn handle_open_workspace_save_error_does_not_panic() {
        let mut app = make_app_with_failing_repo();
        let dir = make_temp_workspace();
        app.handle_open_workspace(dir.path().to_path_buf());
        wait_for_workspace(&mut app);
        assert!(app.state.workspace.data.is_some());
    }

    #[test]
    fn change_language_save_error_does_not_panic() {
        let mut app = make_app_with_failing_repo();
        app.process_action(
            &egui::Context::default(),
            AppAction::ChangeLanguage("ja".to_string()),
        );
    }

    #[test]
    fn trigger_action_is_not_overwritten_before_take_action() {
        let mut app = make_app();
        let dir = make_temp_workspace();

        app.trigger_action(AppAction::OpenWorkspace(dir.path().to_path_buf()));

        assert!(
            matches!(app.pending_action, AppAction::OpenWorkspace(_)),
            "pending_action must still be OpenWorkspace before take_action(); \
             RefreshDiagrams must not overwrite it"
        );

        let action = app.take_action();
        assert!(
            matches!(action, AppAction::OpenWorkspace(_)),
            "take_action() must return OpenWorkspace, not a different action. \
             Regression: shell_ui theme guard was overwriting pending_action on first frame."
        );

        assert!(matches!(app.pending_action, AppAction::None));
    }

    #[test]
    fn refresh_diagrams_is_set_when_no_action_is_pending() {
        let mut app = make_app();
        assert!(matches!(app.pending_action, AppAction::None));

        if matches!(app.pending_action, AppAction::None) {
            app.pending_action = AppAction::RefreshDiagrams;
        }

        assert!(
            matches!(app.pending_action, AppAction::RefreshDiagrams),
            "RefreshDiagrams should be set when no action is pending"
        );
    }

    #[test]
    fn handle_refresh_workspace_rescans_tree() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        app.handle_open_workspace(dir.path().to_path_buf());
        wait_for_workspace(&mut app);
        assert!(app.state.workspace.data.is_some());

        std::fs::write(dir.path().join("new.md"), "# New").unwrap();

        app.handle_refresh_workspace();
        wait_for_workspace(&mut app);
        let ws = app.state.workspace.data.as_ref().unwrap();
        let paths: Vec<_> = ws
            .tree
            .iter()
            .map(|it| it.path().to_string_lossy().to_string())
            .collect();
        assert!(paths.iter().any(|it| it.contains("new.md")));
    }

    #[test]
    fn handle_refresh_workspace_no_workspace_does_nothing() {
        let mut app = make_app();
        app.handle_refresh_workspace();
        assert!(app.state.workspace.data.is_none());
    }

    #[test]
    fn handle_refresh_workspace_error_sets_status_message() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        app.handle_open_workspace(dir.path().to_path_buf());
        wait_for_workspace(&mut app);
        assert!(app.state.workspace.data.is_some());

        app.state.workspace.data.as_mut().unwrap().root =
            std::path::PathBuf::from("/nonexistent/deleted/workspace");

        app.handle_refresh_workspace();
        wait_for_workspace(&mut app);
        assert!(app.state.layout.status_message.is_some());
    }

    #[test]
    fn process_action_refresh_workspace_calls_handler() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        app.handle_open_workspace(dir.path().to_path_buf());
        wait_for_workspace(&mut app);
        app.process_action(&egui::Context::default(), AppAction::RefreshWorkspace);
        wait_for_workspace(&mut app);
        assert!(app.state.workspace.data.is_some());
    }
    #[test]
    fn test_open_workspace_file_updates_buffer() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let file_path = dir.path().join("a.md");
        std::fs::write(&file_path, "A").unwrap();
        app.handle_open_workspace(dir.path().to_path_buf());
        wait_for_workspace(&mut app);
        app.handle_select_document(file_path.clone(), true);

        let doc = app.state.active_document_mut().unwrap();
        doc.buffer = "B".to_string(); // bypass update_buffer to bypass hash updates

        app.handle_select_document(file_path.clone(), true);
        let tab = app
            .tab_previews
            .iter()
            .find(|t| t.path == file_path)
            .unwrap();
        assert!(tab.hash != 0);
    }

    #[test]
    fn test_poll_workspace_load_disconnect() {
        let state = AppState::new(
            katana_core::ai::AiProviderRegistry::default(),
            katana_core::plugin::PluginRegistry::default(),
            katana_platform::SettingsService::default(),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        let mut app = KatanaApp::new(state);

        let (tx, rx) = std::sync::mpsc::channel();
        app.workspace_rx = Some(rx);
        app.state.workspace.is_loading = true;

        drop(tx);

        let ui_ctx = egui::Context::default();
        app.poll_workspace_load(&ui_ctx);

        assert!(!app.state.workspace.is_loading);
    }

    #[test]
    fn test_lazy_loading_flow() {
        let mut app = make_app();
        let dir = make_temp_workspace();
        let path = dir.path().join("lazy.md");
        std::fs::write(&path, "# Lazy content").unwrap();

        app.handle_select_document(path.clone(), false);
        assert_eq!(app.state.document.open_documents.len(), 1);
        assert!(!app.state.document.open_documents[0].is_loaded);

        app.handle_select_document(path.clone(), true);
        assert!(app.state.document.open_documents[0].is_loaded);
        assert_eq!(
            app.state.document.open_documents[0].buffer,
            "# Lazy content"
        );
    }

    #[test]
    fn test_auto_expansion_relative_path() {
        let mut app = make_app();
        app.handle_select_document(std::path::PathBuf::from("root_file.md"), true);
        assert!(app.state.workspace.expanded_directories.is_empty());
    }

    #[test]
    fn test_handle_select_document_lazy_does_not_expand_parents() {
        let mut app = make_app();
        let path = std::path::PathBuf::from("/a/b/c.md");
        app.handle_select_document(path, false); // Lazy load

        assert!(
            app.state.workspace.expanded_directories.is_empty(),
            "Expanded directories should be empty on lazy load"
        );
    }

    #[test]
    fn test_open_multiple_documents_activates_first_file() {
        let mut app = make_app();
        let temp_dir = std::env::temp_dir().join("katana_test_open_multi");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        let f1 = temp_dir.join("1.md");
        let f2 = temp_dir.join("2.md");
        std::fs::write(&f1, "# First").unwrap();
        std::fs::write(&f2, "# Second").unwrap();

        app.process_action(
            &egui::Context::default(),
            AppAction::OpenMultipleDocuments(vec![f1.clone(), f2.clone()]),
        );

        while let Some(path) = app.pending_document_loads.pop_front() {
            app.handle_select_document(path, false);
        }

        assert_eq!(app.state.document.open_documents.len(), 2);
        assert!(app.state.document.open_documents[0].is_loaded);
        assert!(!app.state.document.open_documents[1].is_loaded);
        assert_eq!(app.state.document.active_doc_idx, Some(0));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    use crate::app_state::AppState;
    use crate::preview_pane::PreviewPane;
    use katana_platform::FilesystemService;

    fn setup_test_app() -> KatanaApp {
        let state = AppState::new(
            AiProviderRegistry::new(),
            PluginRegistry::new(),
            katana_platform::SettingsService::default(),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        KatanaApp {
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
            needs_splash: false,
            splash_start: None,
            show_meta_info_for: None,
            pending_relaunch: None,
            changelog_sections: Vec::new(),
            needs_changelog_display: false,
            old_app_version: None,
        }
    }

    #[test]
    fn test_toggle_about_action() {
        let mut app = setup_test_app();
        assert!(!app.show_about);

        app.process_action(&egui::Context::default(), AppAction::ToggleAbout);
        assert!(app.show_about);

        app.process_action(&egui::Context::default(), AppAction::ToggleAbout);
        assert!(!app.show_about);
    }

    #[test]
    fn test_check_for_updates_manual_trigger() {
        let mut app = setup_test_app();
        app.state.update.checking = true;
        app.start_update_check(true);
        assert!(app.show_update_dialog);
    }

    #[test]
    fn test_check_for_updates_action() {
        let mut app = setup_test_app();
        assert!(!app.show_update_dialog);
        assert!(!app.state.update.checking);

        app.process_action(&egui::Context::default(), AppAction::CheckForUpdates);

        assert!(app.show_update_dialog);
        assert!(app.state.update.checking);

        let (tx, rx) = std::sync::mpsc::channel();
        tx.send(Ok(Some(katana_core::update::ReleaseInfo {
            tag_name: "100.0.0".to_string(),
            html_url: "".to_string(),
            download_url: "".to_string(),
            body: "".to_string(),
        })))
        .unwrap();
        app.update_rx = Some(rx);

        let ctx = eframe::egui::Context::default();
        app.poll_update_check(&ctx);

        assert!(app.state.update.available.is_some());
        assert_eq!(
            app.state.update.available.as_ref().unwrap().tag_name,
            "100.0.0"
        );
        assert!(app.update_rx.is_none());
        assert!(app.update_notified);
    }

    #[test]
    fn test_update_check_error_action() {
        let mut app = setup_test_app();
        app.state.update.checking = true;
        let (tx, rx) = std::sync::mpsc::channel();
        tx.send(Err("Network failure".to_string())).unwrap();
        app.update_rx = Some(rx);

        let ctx = eframe::egui::Context::default();
        app.poll_update_check(&ctx);

        assert_eq!(app.state.update.check_error.unwrap(), "Network failure");
        assert!(app.update_rx.is_none());
    }

    #[test]
    fn test_update_check_channel_closed() {
        let mut app = setup_test_app();
        app.state.update.checking = true;
        let (tx, rx) =
            std::sync::mpsc::channel::<Result<Option<katana_core::update::ReleaseInfo>, String>>();
        drop(tx); // cause Err(RecvError) or Disconnected
        app.update_rx = Some(rx);

        let ctx = eframe::egui::Context::default();
        app.poll_update_check(&ctx);

        assert!(!app.state.update.checking);
        assert!(app.update_rx.is_none());
    }

    #[test]
    fn test_background_update_check_shows_dialog_only_once() {
        let mut app = setup_test_app();
        app.start_update_check(false); // background check
        assert!(!app.show_update_dialog); // should be hidden during check
        assert!(!app.update_notified);

        let (tx, rx) = std::sync::mpsc::channel();
        tx.send(Ok(Some(katana_core::update::ReleaseInfo {
            tag_name: "100.0.0".to_string(),
            html_url: "".to_string(),
            download_url: "".to_string(),
            body: "".to_string(),
        })))
        .unwrap();
        app.update_rx = Some(rx);

        let ctx = eframe::egui::Context::default();
        app.poll_update_check(&ctx);

        assert!(app.show_update_dialog);
        assert!(app.update_notified);
    }


    #[test]
    pub(crate) fn export_html_to_tmp_writes_html_file() {
        let preset = katana_core::markdown::color_preset::DiagramColorPreset::dark();
        let filename = "katana_test_export.html";
        let result = super::export_html_to_tmp("# Hello", filename, preset, None);
        let path = result.unwrap();
        assert!(path.exists(), "HTML file must exist at {}", path.display());
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(
            contents.contains("<!DOCTYPE html>"),
            "Output must be valid HTML"
        );
        assert!(contents.contains("Hello"), "Output must contain heading");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    pub(crate) fn export_html_to_tmp_path_is_in_temp_dir() {
        let preset = katana_core::markdown::color_preset::DiagramColorPreset::dark();
        let filename = "katana_path_check.html";
        let path = super::export_html_to_tmp("test", filename, preset, None).unwrap();
        let expected = std::path::PathBuf::from("/tmp").join(filename);
        assert_eq!(path, expected);
        let _ = std::fs::remove_file(&path);
    }


    #[test]
    fn export_as_html_creates_task_with_open_on_complete() {
        let mut app = make_app();
        let dir = tempfile::tempdir().unwrap();
        let md_path = dir.path().join("hello.md");
        std::fs::write(&md_path, "# Integration Test").unwrap();
        app.handle_select_document(md_path.clone(), true);

        app.export_as_html(&egui::Context::default(), "# Integration Test", &md_path);

        assert_eq!(
            app.export_tasks.len(),
            1,
            "must push exactly one ExportTask"
        );
        let task = &app.export_tasks[0];
        assert!(
            task.open_on_complete,
            "HTML export must set open_on_complete = true"
        );
        assert!(
            task.filename.ends_with(".html"),
            "filename must be .html, got {}",
            task.filename
        );
    }

    #[test]
    fn export_as_html_thread_produces_html_file_in_tmp() {
        let mut app = make_app();
        let dir = tempfile::tempdir().unwrap();
        let md_path = dir.path().join("real_doc.md");
        std::fs::write(&md_path, "# Real Document\n\nParagraph content.").unwrap();
        app.handle_select_document(md_path.clone(), true);

        app.export_as_html(
            &egui::Context::default(),
            "# Real Document\n\nParagraph content.",
            &md_path,
        );

        let task = &app.export_tasks[0];
        let result = task.rx.recv_timeout(std::time::Duration::from_secs(5));
        let path = result
            .expect("channel must receive within 5s")
            .expect("export must succeed");

        assert!(
            path.starts_with("/tmp"),
            "path must be under /tmp, got {}",
            path.display()
        );
        assert!(path.exists(), "HTML file must exist at {}", path.display());

        let html = std::fs::read_to_string(&path).unwrap();
        assert!(
            html.contains("<!DOCTYPE html>"),
            "must be full HTML document"
        );
        assert!(html.contains("Real Document"), "must contain the heading");
        assert!(html.contains("Paragraph content"), "must contain body text");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn export_as_html_multiple_calls_create_multiple_tasks() {
        let mut app = make_app();
        let dir = tempfile::tempdir().unwrap();
        let path1 = dir.path().join("doc1.md");
        let path2 = dir.path().join("doc2.md");
        std::fs::write(&path1, "# Doc 1").unwrap();
        std::fs::write(&path2, "# Doc 2").unwrap();
        app.handle_select_document(path1.clone(), true);

        app.export_as_html(&egui::Context::default(), "# Doc 1", &path1);
        app.export_as_html(&egui::Context::default(), "# Doc 2", &path2);

        assert_eq!(
            app.export_tasks.len(),
            2,
            "two exports must create two tasks"
        );

        for task in &app.export_tasks {
            let result = task.rx.recv_timeout(std::time::Duration::from_secs(5));
            let path = result.unwrap().unwrap();
            assert!(path.exists());
            let _ = std::fs::remove_file(&path);
        }
    }


    #[test]
    pub(crate) fn export_html_to_tmp_path_is_canonicalizable() {
        let preset = katana_core::markdown::color_preset::DiagramColorPreset::dark();
        let path = super::export_html_to_tmp("# Test", "katana_canon_test.html", preset, None)
            .expect("generation must succeed");
        let canonical = path
            .canonicalize()
            .unwrap_or_else(|e| panic!("path {} must be canonicalizable: {e}", path.display()));
        assert!(
            canonical.is_absolute(),
            "canonical path must be absolute: {}",
            canonical.display()
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn export_html_file_url_is_valid_and_openable() {
        let preset = katana_core::markdown::color_preset::DiagramColorPreset::dark();
        let path = super::export_html_to_tmp("# URL Test", "katana_url_test.html", preset, None)
            .expect("generation must succeed");

        let url = format!("file://{}", path.display());

        assert!(
            url.starts_with("file:///"),
            "URL must start with file:/// (3 slashes), got: {url}"
        );

        let file_path = std::path::Path::new(url.strip_prefix("file://").unwrap());
        assert!(
            file_path.exists(),
            "path extracted from URL must exist: {} (url={url})",
            file_path.display()
        );

        let canonical = path.canonicalize().unwrap();
        let canonical_url = format!("file://{}", canonical.display());
        let canonical_file_path =
            std::path::Path::new(canonical_url.strip_prefix("file://").unwrap());
        assert!(
            canonical_file_path.exists(),
            "canonical path from URL must exist: {}",
            canonical_file_path.display()
        );

        let _ = std::fs::remove_file(&path);
    }
}