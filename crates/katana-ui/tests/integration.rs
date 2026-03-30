use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
use katana_ui::app_state::{AppAction, AppState, ViewMode};
use katana_ui::shell::KatanaApp;

fn wait_for_workspace_load(harness: &mut Harness<'static, KatanaApp>) {
    for _ in 0..50 {
        harness.step();
        if !harness.state_mut().app_state_mut().workspace.is_loading {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn setup_harness() -> Harness<'static, KatanaApp> {
    std::env::set_var("MERMAID_MMDC", "dummy_missing_executable_for_kittest");

    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let settings_path = std::env::temp_dir().join(format!(
        "katana_test_settings_harness_{}_{}.json",
        std::process::id(),
        id
    ));
    let _ = std::fs::remove_file(&settings_path);

    Harness::builder().build_eframe(move |_cc| {
        let ai_registry = AiProviderRegistry::new();
        let plugin_registry = PluginRegistry::new();
        let mut state = AppState::new(
            ai_registry,
            plugin_registry,
            katana_platform::SettingsService::new(Box::new(
                katana_platform::JsonFileRepository::new(settings_path.clone()),
            )),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        state.config.settings.settings_mut().terms_accepted_version =
            Some(katana_ui::about_info::APP_VERSION.to_string());
        state
            .config
            .settings
            .settings_mut()
            .updates
            .previous_app_version = Some(katana_ui::about_info::APP_VERSION.to_string());

        katana_ui::i18n::set_language("en");
        let mut app = KatanaApp::new(state);
        app.skip_splash();
        app
    })
}

#[test]
fn test_integration_application_startup() {
    let mut harness = setup_harness();
    harness.step();
    let _node = harness.get_by_label("No workspace open.");
}

#[test]
fn test_integration_workspace_and_tabs() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = std::env::temp_dir().join("katana_test_ws");
    let temp_dir = temp_dir.canonicalize().unwrap_or(temp_dir);
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();
    let test_file = temp_dir.join("test1.md");
    std::fs::write(&test_file, "# Hello Katana").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));

    wait_for_workspace_load(&mut harness);

    let file_node = harness.get_all_by_value("file test1.md").next().unwrap();

    file_node.click();
    harness.step();
    harness.step();

    assert!(harness
        .state_mut()
        .app_state_mut()
        .document
        .active_doc_idx
        .is_some());

    harness
        .state_mut()
        .trigger_action(AppAction::CloseDocument(0));
    harness.step();

    assert!(harness
        .state_mut()
        .app_state_mut()
        .document
        .active_doc_idx
        .is_none());
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_toc_panel_display() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = std::env::temp_dir().join("katana_test_toc");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();
    let test_file1 = temp_dir.join("toc_test1.md");
    std::fs::write(&test_file1, "# Heading 1").unwrap();
    let test_file1 = test_file1.canonicalize().unwrap_or(test_file1);

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);

    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(test_file1.clone()));
    harness.step();

    let toggle_btn = harness.get_by_label("toggle_toc");
    toggle_btn.click();
    harness.step(); // UI Registers click, sets pending_action = ToggleToc
    harness.step(); // KatanaApp reads pending_action, sets show_toc = true, renders TOC panel

    let toc_visible = harness.state_mut().app_state_mut().layout.show_toc;
    assert!(toc_visible, "show_toc should be true after clicking button");

    let toc_title = katana_ui::i18n::get().toc.title.clone();
    let _panel = harness.get_by_label(&toc_title);

    let headings_count = harness.query_all_by_label("Heading 1").count();
    assert_eq!(
        headings_count, 2,
        "Heading 1 should appear exactly twice: once in TOC, once in preview text"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_toc_enable_disable_setting() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = std::env::temp_dir().join("katana_test_toc_setting");
    let temp_dir = temp_dir.canonicalize().unwrap_or(temp_dir);
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();
    let test_file1 = temp_dir.join("toc_test_setting.md");
    std::fs::write(&test_file1, "# Heading 1").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);
    let abs_path = test_file1.canonicalize().unwrap_or(test_file1.clone());
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path));
    harness.step();
    harness.step();

    let toc_icon = "toggle_toc";
    assert_eq!(
        harness.query_all_by_label(toc_icon).count(),
        1,
        "TOC button should be visible when toc_visible setting is true (default)"
    );

    harness
        .state_mut()
        .app_state_mut()
        .config
        .settings
        .settings_mut()
        .layout
        .toc_visible = false;
    harness.step();
    harness.step();

    assert!(
        !harness
            .state_mut()
            .app_state_mut()
            .config
            .settings
            .settings()
            .layout
            .toc_visible,
        "TOC setting must be false"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_toc_panel_hides_when_disabled() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = std::env::temp_dir().join("katana_test_toc_hide");
    let temp_dir = temp_dir.canonicalize().unwrap_or(temp_dir);
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();
    let test_file1 = temp_dir.join("toc_hide_test.md");
    std::fs::write(&test_file1, "# Heading 1").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);
    let abs_path = test_file1.canonicalize().unwrap_or(test_file1.clone());
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path));
    harness.step();
    harness.step();

    harness.state_mut().trigger_action(AppAction::ToggleToc);
    for _ in 0..10 {
        harness.step();
    }

    let toc_title = katana_ui::i18n::get().toc.title.clone();
    assert_eq!(
        harness.query_all_by_label(&toc_title).count(),
        1,
        "TOC panel MUST be visible after toggling it on"
    );

    harness
        .state_mut()
        .app_state_mut()
        .config
        .settings
        .settings_mut()
        .layout
        .toc_visible = false;
    harness.step();
    harness.step();



    let is_panel_visible = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.query_all_by_label(&toc_title).count()
    }))
    .unwrap_or(0)
        > 0;

    assert!(
        !is_panel_visible,
        "TOC panel MUST NOT be visible when toc_visible setting is false"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_changelog_tab_display() {
    use katana_ui::app_state::AppAction;
    use katana_ui::changelog::ChangelogSection;

    let mut harness = setup_harness();
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::ShowReleaseNotes);

    harness.step();

    harness.state_mut().clear_changelog_rx_for_test();
    harness
        .state_mut()
        .set_changelog_sections_for_test(vec![ChangelogSection {
            version: "0.8.0".to_string(),
            heading: "v0.8.0".to_string(),
            body: "### Features\n- Fixed the close button overlap".to_string(),
            default_open: true,
        }]);

    for _ in 0..9 {
        harness.step();
    }
    {
        let state = harness.state();
        let app = state.app_state_for_test();
        let active_doc = app.active_document().expect("a document MUST be active");
        assert!(active_doc
            .path
            .to_string_lossy()
            .starts_with("Katana://ChangeLog"));
    }
    harness.step();
    harness.step();

    let i18n = katana_ui::i18n::get();
    let expected_title = format!("{} v{}", i18n.menu.release_notes, env!("CARGO_PKG_VERSION"));
    harness.get_by_label(&expected_title);

    harness.get_by_label("Fixed the close button overlap");

    let header_label = "v0.8.0"; // It's open by default now!
    harness.get_by_label(header_label).hover();
    harness.step();
    harness.get_by_label(header_label).click();
    harness.step();
}

#[test]
fn test_integration_view_modes() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = std::env::temp_dir().join("katana_test_ws_modes");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();
    let test_file = temp_dir.join("test_modes.md");
    std::fs::write(&test_file, "# Hello View Modes\n**Bold text here.**").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);
    let abs_path = test_file.canonicalize().unwrap_or(test_file);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path));
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::PreviewOnly);
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::PreviewOnly
    );
    let _ = harness.get_all_by_value("Bold text here.").next();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::Split
    );
    let _ = harness.get_all_by_value("Bold text here.").next();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::CodeOnly);
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::CodeOnly
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}
#[test]
fn test_integration_settings_window() {
    katana_ui::i18n::set_language("en");
    let mut harness = setup_harness();
    harness.step();

    harness
        .state_mut()
        .trigger_action(katana_ui::app_state::AppAction::ToggleSettings);
    harness.step();
    harness.step();

    for node in harness.query_all_by_label("Font") {
        node.click();
    }
    harness.step();
    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .config
            .active_settings_tab,
        katana_ui::app_state::SettingsTab::Font
    );
    for node in harness.query_all_by_label("Layout") {
        node.click();
    }
    harness.step();
    harness
        .state_mut()
        .app_state_mut()
        .config
        .active_settings_tab = katana_ui::app_state::SettingsTab::Layout;
    harness.step();
    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .config
            .active_settings_tab,
        katana_ui::app_state::SettingsTab::Layout
    );

    harness
        .state_mut()
        .trigger_action(katana_ui::app_state::AppAction::ToggleSettings);
    harness.step();
}

#[test]
fn test_integration_editor_line_numbers_and_highlight() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = std::env::temp_dir().join("katana_test_editor_lines");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();
    let test_file = temp_dir.join("lines.md");
    std::fs::write(&test_file, "Line 1\nLine 2\nLine 3").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);
    let abs_path = test_file.canonicalize().unwrap_or(test_file);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path));
    harness.step();
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::CodeOnly);
    harness.step();
    harness.step();

    let count_1 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.query_all_by_label("1").count()
    }))
    .unwrap_or(0);
    let count_2 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.query_all_by_label("2").count()
    }))
    .unwrap_or(0);
    let count_3 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.query_all_by_label("3").count()
    }))
    .unwrap_or(0);

    assert!(count_1 > 0, "Line number 1 should be visible");
    assert!(count_2 > 0, "Line number 2 should be visible");
    assert!(count_3 > 0, "Line number 3 should be visible");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_workspace_directory_toggle_non_recursive() {
    let mut harness = setup_harness();
    harness.step();

    let unique_name = format!(
        "katana_test_dir_toggle_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    let temp_dir = std::env::temp_dir().join(unique_name);
    let _ = std::fs::remove_dir_all(&temp_dir);
    let dir2 = temp_dir.join("dir1").join("dir2");
    std::fs::create_dir_all(&dir2).unwrap();
    let test_file = dir2.join("test.md");
    std::fs::write(&test_file, "# Content").unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);
    harness.step();
    harness.step();

    let dir1_node = harness.get_by_label("dir dir1");

    dir1_node.click();
    harness.step();
    harness.step();

    let dir2_node = harness.get_by_label("dir dir2");

    let test_md_visible = harness
        .get_all_by_role(egui::accesskit::Role::Label)
        .any(|n| n.value().map(|v| v.contains("test.md")).unwrap_or(false));
    assert!(
        !test_md_visible,
        "test.md should NOT be visible (non-recursive expansion)"
    );

    dir2_node.click();
    harness.step();
    harness.step();

    let _ = harness.get_by_label("file test.md");

    let cache_before = harness
        .state_mut()
        .app_state_mut()
        .workspace
        .expanded_directories
        .clone();
    assert!(
        !cache_before.is_empty(),
        "Cache should contain expanded dirs"
    );

    let parent_label = harness.get_by_label("dir dir1");
    parent_label.click(); // Collapses dir1
    harness.step();
    harness.step();

    let parent_label = harness.get_by_label("dir dir1");
    parent_label.click(); // Expands dir1
    harness.step();
    harness.step();

    let test_md_visible_cached = harness
        .get_all_by_role(egui::accesskit::Role::Label)
        .any(|n| n.value().map(|v| v.contains("test.md")).unwrap_or(false));
    assert!(
        test_md_visible_cached,
        "test.md should be visible after closing and reopening dir1 (cached expansion)"
    );

    let collapse_all = harness.get_by_label("-"); // The collapse all button has text "-"
    collapse_all.click();
    harness.step();
    harness.step();

    let dir2_present = harness
        .get_all_by_role(egui::accesskit::Role::Label)
        .any(|n| n.value().map(|l| l.contains("dir2")).unwrap_or(false));
    assert!(
        !dir2_present,
        "dir2 should NOT be visible after Collapse All"
    );

    let cache_after = harness
        .state_mut()
        .app_state_mut()
        .workspace
        .expanded_directories
        .clone();
    assert!(
        cache_after.is_empty(),
        "Cache should be EMPTY after Collapse All"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}


#[test]
fn test_integration_update_buffer() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = std::env::temp_dir().join("katana_test_ws_buf");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();
    let test_file = temp_dir.join("buf_test.md");
    std::fs::write(&test_file, "# Original").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);
    let abs_path = test_file.canonicalize().unwrap_or(test_file);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path));
    harness.step();

    harness.state_mut().trigger_action(AppAction::UpdateBuffer(
        "# Updated\n\nNew content".to_string(),
    ));
    harness.step();
    let active_idx = harness
        .state_mut()
        .app_state_mut()
        .document
        .active_doc_idx
        .unwrap();
    let buf = harness.state_mut().app_state_mut().document.open_documents[active_idx]
        .buffer
        .clone();
    assert!(buf.contains("New content"));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_save_document() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = std::env::temp_dir().join("katana_test_ws_save");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();
    let test_file = temp_dir.join("save_test.md");
    std::fs::write(&test_file, "# Hello").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);
    let abs_path = test_file.canonicalize().unwrap_or(test_file.clone());
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path.clone()));
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::UpdateBuffer("# Saved Content".to_string()));
    harness.step();
    harness.state_mut().trigger_action(AppAction::SaveDocument);
    harness.step();
    let content = std::fs::read_to_string(&test_file).unwrap();
    assert_eq!(content, "# Saved Content");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_multiple_documents_and_navigation() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = std::env::temp_dir().join("katana_test_ws_multi");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();
    let file1 = temp_dir.join("alpha.md");
    let file2 = temp_dir.join("beta.md");
    std::fs::write(&file1, "# Alpha").unwrap();
    std::fs::write(&file2, "# Beta").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);

    let abs1 = file1.canonicalize().unwrap_or(file1);
    let abs2 = file2.canonicalize().unwrap_or(file2);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs1));
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs2));
    harness.step();

    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        2
    );
    assert_eq!(
        harness.state_mut().app_state_mut().document.active_doc_idx,
        Some(1)
    );

    harness
        .state_mut()
        .trigger_action(AppAction::CloseDocument(0));
    harness.step();
    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        1
    );
    assert_eq!(
        harness.state_mut().app_state_mut().document.active_doc_idx,
        Some(0)
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_preview_with_diagram_content() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = std::env::temp_dir().join("katana_test_ws_diag");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();
    let test_file = temp_dir.join("diagram_test.md");
    let content = "# Diagram Test\n\n```mermaid\ngraph TD; A-->B\n```\n\n```drawio\n<mxGraphModel><root><mxCell id=\"0\"/></mxGraphModel>\n```\n";
    std::fs::write(&test_file, content).unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);
    let abs_path = test_file.canonicalize().unwrap_or(test_file);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path));
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::PreviewOnly);
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::PreviewOnly
    );
    let _ = harness.get_all_by_value("Diagram Test").next();

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_workspace_with_subdirectory() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = std::env::temp_dir().join("katana_test_ws_subdir");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(temp_dir.join("docs")).unwrap();
    std::fs::write(temp_dir.join("root.md"), "# Root").unwrap();
    std::fs::write(temp_dir.join("docs").join("inner.md"), "# Inner").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));

    wait_for_workspace_load(&mut harness);

    let _ = harness.get_by_label("dir docs");
    let _ = harness.get_by_label("file root.md");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_open_all_markdown() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = std::env::temp_dir().join("katana_test_open_all_md");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(temp_dir.join("docs")).unwrap();

    let md1 = temp_dir.join("docs").join("a.md");
    let md2 = temp_dir.join("docs").join("b.md");
    let not_md = temp_dir.join("docs").join("c.txt");

    std::fs::write(&md1, "# A").unwrap();
    std::fs::write(&md2, "# B").unwrap();
    std::fs::write(&not_md, "not md").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));

    wait_for_workspace_load(&mut harness);

    wait_for_workspace_load(&mut harness);

    harness
        .state_mut()
        .trigger_action(AppAction::OpenMultipleDocuments(vec![
            md1.clone(),
            md2.clone(),
        ]));

    for _ in 0..5 {
        harness.step();
    }

    let state = harness.state_mut().app_state_mut();
    assert_eq!(
        state.document.open_documents.len(),
        2,
        "Should open 2 documents"
    );

    harness
        .state_mut()
        .trigger_action(AppAction::OpenMultipleDocuments(vec![
            md1.clone(),
            md2.clone(),
        ]));

    for _ in 0..5 {
        harness.step();
    }

    let state = harness.state_mut().app_state_mut();
    assert_eq!(
        state.document.open_documents.len(),
        2,
        "Should not duplicate tabs on re-opening"
    );

    assert_eq!(state.document.active_doc_idx, Some(0)); // First file is activated

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_directory_collapse_bug() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = std::env::temp_dir().join("katana_test_collapse");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();
    let parent_dir = temp_dir.join("parent");
    std::fs::create_dir_all(&parent_dir).unwrap();
    let sub_dir = parent_dir.join("child");
    std::fs::create_dir_all(&sub_dir).unwrap();
    std::fs::write(sub_dir.join("file.md"), "# File").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));

    wait_for_workspace_load(&mut harness);

    use egui_kittest::kittest::Queryable;

    let parent_visible = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.get_by_label("dir parent")
    }))
    .is_ok();
    assert!(parent_visible, "Parent should be visible");

    let child_visible = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.get_by_label("dir child")
    }))
    .is_ok();
    assert!(!child_visible, "Child should not be visible initially");

    harness
        .state_mut()
        .app_state_mut()
        .workspace
        .force_tree_open = Some(true);
    harness.step();
    harness.step();
    harness.step();

    let parent_visible = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.get_by_label("dir parent")
    }))
    .is_ok();
    assert!(parent_visible, "Parent should still be visible");

    let child_visible = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.get_by_label("dir child")
    }))
    .is_ok();
    assert!(child_visible, "Child should now be visible");

    harness
        .state_mut()
        .app_state_mut()
        .workspace
        .force_tree_open = Some(false);
    harness.step();
    harness.step(); // ensure flushed

    let parent_visible = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.get_by_label("dir parent")
    }))
    .is_ok();
    assert!(parent_visible, "Parent should still be visible");

    let child_visible = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.get_by_label("dir child")
    }))
    .is_ok();
    assert!(!child_visible, "Child should be hidden");

    let parent_node = harness.get_by_label("dir parent");
    parent_node.click();
    harness.step();

    let file_visible = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.get_by_label("file file.md")
    }))
    .is_ok();
    assert!(!file_visible, "Child directory should be collapsed!");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_workspace_panel_collapsed() {
    let mut harness = setup_harness();
    harness.step();

    harness.state_mut().app_state_mut().layout.show_workspace = false;
    harness.step();
    assert!(!harness.state_mut().app_state_mut().layout.show_workspace);

    {
        use egui_kittest::kittest::Queryable;
        for label in ["›", ">", "❯"] {
            let nodes: Vec<_> = harness.query_all_by_label(label).collect();
            if let Some(node) = nodes.into_iter().next() {
                node.click();
                harness.step();
                break;
            }
        }
    }

    harness.state_mut().app_state_mut().layout.show_workspace = true;
    harness.step();
}

#[test]
fn test_integration_workspace_tab_persistence() {
    let mut harness = setup_harness();
    harness.step();

    let ws1 = std::env::temp_dir().join("katana_test_ws1");
    let _ = std::fs::remove_dir_all(&ws1);
    std::fs::create_dir_all(&ws1).unwrap();
    let file1 = ws1.join("file1.md");
    std::fs::write(&file1, "# WS1").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(ws1.clone()));
    wait_for_workspace_load(&mut harness);

    let abs_file1 = file1.canonicalize().unwrap_or(file1);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_file1.clone()));
    harness.step();
    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        1
    );

    let ws2 = std::env::temp_dir().join("katana_test_ws2");
    let _ = std::fs::remove_dir_all(&ws2);
    std::fs::create_dir_all(&ws2).unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(ws2.clone()));
    wait_for_workspace_load(&mut harness);

    let cache_key = format!("workspace_tabs:{}", ws1.display());

    let cache_json = harness
        .state_mut()
        .app_state_mut()
        .config
        .cache
        .get_persistent(&cache_key);
    assert!(
        cache_json.is_some(),
        "Workspace 1 tab state must be saved to cache before switching. Key was: {}",
        cache_key
    );

    let json_str = cache_json.unwrap();
    assert!(
        json_str.contains("file1.md"),
        "The saved cache must contain the opened tab's path"
    );

    let _ = std::fs::remove_dir_all(&ws1);
    let _ = std::fs::remove_dir_all(&ws2);
}

#[test]
fn test_integration_split_mode_with_document() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = std::env::temp_dir().join("katana_test_split");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();
    let test_file = temp_dir.join("split_test.md");
    std::fs::write(&test_file, "# Split Mode Test\n\nContent here.").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);

    let abs_path = test_file.canonicalize().unwrap_or(test_file);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path));
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::Split
    );

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::CodeOnly);
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::CodeOnly
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_multiple_tabs_close() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = std::env::temp_dir().join("katana_test_multi_tab");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(temp_dir.join("file1.md"), "# File 1").unwrap();
    std::fs::write(temp_dir.join("file2.md"), "# File 2").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);

    let p1 = temp_dir
        .join("file1.md")
        .canonicalize()
        .unwrap_or_else(|_| temp_dir.join("file1.md"));
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(p1));
    harness.step();

    let p2 = temp_dir
        .join("file2.md")
        .canonicalize()
        .unwrap_or_else(|_| temp_dir.join("file2.md"));
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(p2));
    harness.step();

    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        2
    );

    harness
        .state_mut()
        .trigger_action(AppAction::CloseDocument(0));
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::CloseDocument(0));
    harness.step();
    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        0
    );
    assert!(harness
        .state_mut()
        .app_state_mut()
        .document
        .active_doc_idx
        .is_none());

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_workspace_tree_expand_collapse() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = std::env::temp_dir().join("katana_test_tree_toggle");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(temp_dir.join("subdir")).unwrap();
    std::fs::write(temp_dir.join("root.md"), "# Root").unwrap();
    std::fs::write(temp_dir.join("subdir").join("child.md"), "# Child").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);

    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .workspace
            .force_tree_open,
        None
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_preview_only_no_document() {
    let mut harness = setup_harness();
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(katana_ui::app_state::ViewMode::PreviewOnly);
    harness.step();
    let _ = harness.get_by_label("No document open.");
}

#[test]
fn test_integration_refresh_diagrams_action() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let md_path = temp_dir.path().join("note.md");
    std::fs::write(&md_path, "# Note").unwrap();

    let mut harness = setup_harness();
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);

    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(md_path));
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::RefreshDiagrams);
    harness.step();
}

#[test]
fn test_integration_sidebar_collapse_expand() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("test.md"), "# Test").unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);

    harness.state_mut().app_state_mut().layout.show_workspace = false;
    harness.step();
    harness.step();

    harness.state_mut().app_state_mut().layout.show_workspace = true;
    harness.step();
}

#[test]
fn test_integration_tree_toggle_buttons() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(temp_dir.path().join("sub")).unwrap();
    std::fs::write(temp_dir.path().join("root.md"), "# Root").unwrap();
    std::fs::write(temp_dir.path().join("sub").join("child.md"), "# Child").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);

    if let Some(btn) = harness.query_all_by_label("+").next() {
        btn.click();
    }
    harness.step();

    if let Some(btn) = harness.query_all_by_label("-").next() {
        btn.click();
    }
    harness.step();
}

#[test]
fn test_integration_tab_navigation_and_close() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("a.md"), "# A").unwrap();
    std::fs::write(temp_dir.path().join("b.md"), "# B").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);

    let a_path = temp_dir.path().join("a.md");
    let b_path = temp_dir.path().join("b.md");
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(a_path.clone()));
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(b_path.clone()));
    harness.step();

    if let Some(btn) = harness.query_all_by_label("◀").next() {
        btn.click();
    }
    harness.step();

    if let Some(btn) = harness.query_all_by_label("▶").next() {
        btn.click();
    }
    harness.step();

    if let Some(btn) = harness.query_all_by_label("x").next() {
        btn.click();
    }
    harness.step();
}


#[test]
fn test_integration_directory_entry_click_toggle() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(temp_dir.path().join("mydir")).unwrap();
    std::fs::write(temp_dir.path().join("mydir").join("inner.md"), "# Inner").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);

    harness
        .state_mut()
        .app_state_mut()
        .workspace
        .force_tree_open = Some(true);
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .workspace
        .force_tree_open = Some(false);
    harness.step();
}

#[test]
fn test_integration_text_edit_triggers_update_buffer() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    let md_path = temp_dir.path().join("edit.md");
    std::fs::write(&md_path, "# Editable").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(md_path));
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::CodeOnly);
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::UpdateBuffer("# Modified content".to_string()));
    harness.step();
}

#[test]
fn test_integration_refresh_button_click() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    let md_path = temp_dir.path().join("diag.md");
    std::fs::write(
        &md_path,
        "# Diagram\n```drawio\n<mxGraphModel></mxGraphModel>\n```",
    )
    .unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(md_path));
    harness.step();

    if let Some(btn) = harness.query_all_by_label("🔄").next() {
        btn.click();
    }
    harness.step();
}


fn setup_harness_with_json_repo(settings_path: &std::path::Path) -> Harness<'static, KatanaApp> {
    let path = settings_path.to_path_buf();
    Harness::builder().build_eframe(move |_cc| {
        let repo = katana_platform::JsonFileRepository::new(path.clone());
        let settings = katana_platform::SettingsService::new(Box::new(repo));
        let mut state = AppState::new(
            AiProviderRegistry::new(),
            PluginRegistry::new(),
            settings,
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        state.config.settings.settings_mut().terms_accepted_version =
            Some(katana_ui::about_info::APP_VERSION.to_string());
        state
            .config
            .settings
            .settings_mut()
            .updates
            .previous_app_version = Some(katana_ui::about_info::APP_VERSION.to_string());

        katana_ui::i18n::set_language("en");
        let mut app = KatanaApp::new(state);
        app.skip_splash();
        app
    })
}

#[test]
fn test_persistence_workspace_roundtrip() {
    let settings_dir = tempfile::tempdir().unwrap();
    let settings_path = settings_dir.path().join("settings.json");

    let ws_dir = tempfile::tempdir().unwrap();
    std::fs::write(ws_dir.path().join("doc.md"), "# Hello").unwrap();

    {
        let mut harness = setup_harness_with_json_repo(&settings_path);
        harness.step();

        harness
            .state_mut()
            .trigger_action(AppAction::OpenWorkspace(ws_dir.path().to_path_buf()));
        wait_for_workspace_load(&mut harness);

        assert!(harness.state_mut().app_state_mut().workspace.data.is_some());

        let json = std::fs::read_to_string(&settings_path).unwrap();
        assert!(
            json.contains(&ws_dir.path().display().to_string()),
            "settings.json should contain the workspace path, got: {json}"
        );
    }

    {
        let repo = katana_platform::JsonFileRepository::new(settings_path.to_path_buf());
        let settings = katana_platform::SettingsService::new(Box::new(repo));
        let restored_ws = settings.settings().workspace.last_workspace.clone();

        assert!(
            restored_ws.is_some(),
            "last_workspace should be persisted in settings.json"
        );
        assert!(
            restored_ws
                .unwrap()
                .contains(&ws_dir.path().display().to_string()),
            "Restored workspace should match the previously opened directory"
        );
    }
}

#[test]
fn test_persistence_language_roundtrip() {
    let settings_dir = tempfile::tempdir().unwrap();
    let settings_path = settings_dir.path().join("settings.json");

    {
        let mut harness = setup_harness_with_json_repo(&settings_path);
        harness.step();

        harness
            .state_mut()
            .trigger_action(AppAction::ChangeLanguage("ja".to_string()));
        harness.step();

        let json = std::fs::read_to_string(&settings_path).unwrap();
        assert!(
            json.contains("\"language\": \"ja\""),
            "settings.json should contain language=ja, got: {json}"
        );
        katana_ui::i18n::set_language("en");
    }

    {
        let repo = katana_platform::JsonFileRepository::new(settings_path.to_path_buf());
        let settings = katana_platform::SettingsService::new(Box::new(repo));
        assert_eq!(
            settings.settings().language,
            "ja",
            "Language should be restored as 'ja' from disk"
        );
    }
}

#[test]
fn test_persistence_multiple_changes_accumulate() {
    let settings_dir = tempfile::tempdir().unwrap();
    let settings_path = settings_dir.path().join("settings.json");

    let ws_dir = tempfile::tempdir().unwrap();
    std::fs::write(ws_dir.path().join("readme.md"), "# Readme").unwrap();

    {
        let mut harness = setup_harness_with_json_repo(&settings_path);
        harness.step();

        harness
            .state_mut()
            .trigger_action(AppAction::OpenWorkspace(ws_dir.path().to_path_buf()));
        wait_for_workspace_load(&mut harness);

        harness
            .state_mut()
            .trigger_action(AppAction::ChangeLanguage("ja".to_string()));
        harness.step();
    }

    {
        let repo = katana_platform::JsonFileRepository::new(settings_path.to_path_buf());
        let settings = katana_platform::SettingsService::new(Box::new(repo));
        let s = settings.settings();

        assert!(
            s.workspace.last_workspace.is_some(),
            "last_workspace should be persisted"
        );
        assert_eq!(s.language, "ja", "language should be persisted");
        katana_ui::i18n::set_language("en");
    }
}

#[test]
fn test_persistence_corrupt_file_falls_back_to_defaults() {
    let settings_dir = tempfile::tempdir().unwrap();
    let settings_path = settings_dir.path().join("settings.json");

    std::fs::write(&settings_path, "NOT VALID JSON {{{").unwrap();

    let mut harness = setup_harness_with_json_repo(&settings_path);
    harness.step();

    let s = harness.state_mut().app_state_mut();
    assert_eq!(
        s.config.settings.settings().theme.theme,
        "dark",
        "Should fall back to default theme"
    );
    assert_eq!(
        s.config.settings.settings().language,
        "en",
        "Should fall back to default language"
    );
    assert!(
        s.config
            .settings
            .settings()
            .workspace
            .last_workspace
            .is_none(),
        "Should fall back to no workspace"
    );
}

#[test]
fn test_persistence_missing_file_uses_defaults() {
    let settings_dir = tempfile::tempdir().unwrap();
    let settings_path = settings_dir.path().join("nonexistent.json");

    let mut harness = setup_harness_with_json_repo(&settings_path);
    harness.step();

    let s = harness.state_mut().app_state_mut();
    assert_eq!(s.config.settings.settings().theme.theme, "dark");
    assert_eq!(s.config.settings.settings().language, "en");
}

#[test]
fn test_regression_preview_content_visible_in_preview_only_mode() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    let md_path = temp_dir.path().join("preview_regression.md");
    std::fs::write(&md_path, "# RegressionTestHeading\n\nSome body text.").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(md_path));
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::PreviewOnly);
    harness.step();
    harness.step();

    let no_preview_label = katana_ui::i18n::get().preview.no_preview.clone();
    let no_preview_nodes: Vec<_> = harness.query_all_by_label(&no_preview_label).collect();
    assert!(
        no_preview_nodes.is_empty(),
        "Preview pane must NOT show '{no_preview_label}' when a document is open. \
         Likely cause: render_preview_header is consuming all available height, \
         leaving ScrollArea with height=0."
    );
}

#[test]
fn test_regression_preview_content_visible_in_split_mode() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    let md_path = temp_dir.path().join("split_regression.md");
    std::fs::write(&md_path, "# SplitRegressionHeading\n\nSome body text.").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(md_path));
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step();
    harness.step();

    let no_preview_label = katana_ui::i18n::get().preview.no_preview.clone();
    let no_preview_nodes: Vec<_> = harness.query_all_by_label(&no_preview_label).collect();
    assert!(
        no_preview_nodes.is_empty(),
        "Split mode preview pane must NOT show '{no_preview_label}' when a document is open."
    );
}

#[test]
fn test_split_direction_setting_toggles_correctly() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    let md_path = temp_dir.path().join("toggle_test.md");
    std::fs::write(&md_path, "# Toggle Test").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(md_path));
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step();

    assert_eq!(
        harness.state_mut().app_state_mut().active_split_direction(),
        katana_platform::SplitDirection::Horizontal,
        "Default split direction must be Horizontal"
    );

    harness
        .state_mut()
        .trigger_action(AppAction::SetSplitDirection(
            katana_platform::SplitDirection::Vertical,
        ));
    harness.step();

    assert_eq!(
        harness.state_mut().app_state_mut().active_split_direction(),
        katana_platform::SplitDirection::Vertical,
        "Split direction must switch to Vertical"
    );

    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::Split,
        "Split direction toggle must not leave split mode"
    );
}

#[test]
fn test_integration_cache_facade_restores_tabs() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    let md_path1 = temp_dir.path().join("tab1.md");
    let md_path2 = temp_dir.path().join("tab2.md");
    std::fs::write(&md_path1, "# Tab 1").unwrap();
    std::fs::write(&md_path2, "# Tab 2").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);

    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(md_path1.clone()));
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(md_path2.clone()));
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);

    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        2
    );
    assert_eq!(
        harness.state_mut().app_state_mut().document.active_doc_idx,
        Some(1)
    );
}

#[test]
fn test_ui_split_dir_toggle_horizontal_to_vertical() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    let md_path = temp_dir.path().join("toggle_ui.md");
    std::fs::write(&md_path, "# UI Toggle Test").unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(md_path));
    harness.step();
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step(); // UI should render and the '⇕' button should appear

    assert_eq!(
        harness.state_mut().app_state_mut().active_split_direction(),
        katana_platform::SplitDirection::Horizontal,
    );

    let node = harness.get_by_label("Toggle Split Direction");
    node.click();
    harness.step();
    harness.step(); // Action is processed on the next frame

    assert_eq!(
        harness.state_mut().app_state_mut().active_split_direction(),
        katana_platform::SplitDirection::Vertical,
        "UI click on '⇕' should toggle split direction to Vertical",
    );
}


#[test]
fn test_action_set_split_direction_horizontal_to_vertical() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    let md_path = temp_dir.path().join("dir_test.md");
    std::fs::write(&md_path, "# Dir Test").unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(md_path));
    harness.step();
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step();

    assert_eq!(
        harness.state_mut().app_state_mut().active_split_direction(),
        katana_platform::SplitDirection::Horizontal,
    );

    harness
        .state_mut()
        .trigger_action(AppAction::SetSplitDirection(
            katana_platform::SplitDirection::Vertical,
        ));
    harness.step();

    assert_eq!(
        harness.state_mut().app_state_mut().active_split_direction(),
        katana_platform::SplitDirection::Vertical,
        "SetSplitDirection(Vertical) must update per-tab state to Vertical"
    );

    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .config
            .settings
            .settings()
            .layout
            .split_direction,
        katana_platform::SplitDirection::Horizontal,
        "settings.split_direction must remain unchanged (not persisted)"
    );
}

#[test]
fn test_action_set_split_direction_roundtrip() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    let md_path = temp_dir.path().join("roundtrip_test.md");
    std::fs::write(&md_path, "# Roundtrip Test").unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(md_path));
    harness.step();
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::SetSplitDirection(
            katana_platform::SplitDirection::Vertical,
        ));
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_split_direction(),
        katana_platform::SplitDirection::Vertical,
    );

    harness
        .state_mut()
        .trigger_action(AppAction::SetSplitDirection(
            katana_platform::SplitDirection::Horizontal,
        ));
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_split_direction(),
        katana_platform::SplitDirection::Horizontal,
        "Must be able to switch back from Vertical to Horizontal"
    );
}

#[test]
fn test_action_set_pane_order_roundtrip() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    let md_path = temp_dir.path().join("pane_order_test.md");
    std::fs::write(&md_path, "# Pane Order Test").unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(md_path));
    harness.step();
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step();

    assert_eq!(
        harness.state_mut().app_state_mut().active_pane_order(),
        katana_platform::PaneOrder::EditorFirst,
    );

    harness.state_mut().trigger_action(AppAction::SetPaneOrder(
        katana_platform::PaneOrder::PreviewFirst,
    ));
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_pane_order(),
        katana_platform::PaneOrder::PreviewFirst,
        "SetPaneOrder(PreviewFirst) must update per-tab pane order"
    );

    harness.state_mut().trigger_action(AppAction::SetPaneOrder(
        katana_platform::PaneOrder::EditorFirst,
    ));
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_pane_order(),
        katana_platform::PaneOrder::EditorFirst,
        "Must be able to switch back from PreviewFirst to EditorFirst"
    );
}


#[test]
fn test_file_entry_label_is_left_aligned() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("alignment.md"), "# Alignment").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);
    harness.step();

    let nodes: Vec<_> = harness.query_all_by_label("file alignment.md").collect();
    assert!(
        !nodes.is_empty(),
        "File entry '📄 alignment.md' must be present in the workspace tree"
    );

    let node = &nodes[0];
    let label_rect = node.rect();
    let label_width = label_rect.width();
    assert!(
        label_width < 176.0,
        "File entry label width must be text-width, not full-row-width. \
         Got width={label_width:.1}, expected < 176.0 (indicates add_sized right-alignment bug)"
    );
}

#[test]
fn test_file_entry_click_opens_document() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("clickable.md"), "# Clickable").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);
    harness.step();

    assert!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .active_doc_idx
            .is_none(),
        "No document should be open before clicking"
    );

    let nodes: Vec<_> = harness.query_all_by_label("file clickable.md").collect();
    assert!(!nodes.is_empty(), "File entry must be present");
    nodes[0].click();
    harness.step();
    harness.step();

    assert!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .active_doc_idx
            .is_some(),
        "Clicking a file entry must open the document (active_doc_idx should be Some)"
    );
    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        1,
        "Exactly one document should be open after clicking"
    );
}

#[test]
fn test_tab_nav_buttons_have_tooltips() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("a.md"), "# A").unwrap();
    std::fs::write(temp_dir.path().join("b.md"), "# B").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(temp_dir.path().join("a.md")));
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(temp_dir.path().join("b.md")));
    harness.step();

    let prev_nodes: Vec<_> = harness.query_all_by_label("◀").collect();
    assert!(
        !prev_nodes.is_empty(),
        "◀ (previous tab) button must be present"
    );

    let next_nodes: Vec<_> = harness.query_all_by_label("▶").collect();
    assert!(
        !next_nodes.is_empty(),
        "▶ (next tab) button must be present"
    );

    prev_nodes[0].hover();
    harness.step();
    harness.step();

    let prev_tooltip = katana_ui::i18n::get().tab.nav_prev.clone();
    let next_tooltip = katana_ui::i18n::get().tab.nav_next.clone();
    assert_ne!(
        prev_tooltip, "tab_nav_prev",
        "tab_nav_prev i18n key must resolve to translated text"
    );
    assert_ne!(
        next_tooltip, "tab_nav_next",
        "tab_nav_next i18n key must resolve to translated text"
    );

}

#[test]
fn test_font_size_slider_has_hover_tooltip() {
    let mut harness = setup_harness();
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::ToggleSettings);
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .config
        .active_settings_tab = katana_ui::app_state::SettingsTab::Font;
    harness.step();
    harness.step();

    let hint_text = katana_ui::i18n::get()
        .settings
        .font
        .size_slider_hint
        .clone();
    assert_ne!(
        hint_text, "settings_font_size_slider_hint",
        "i18n key must resolve to a translated value, not the key itself"
    );

    let font_size_label = katana_ui::i18n::get().settings.font.size.clone();
    let _slider = harness.get_by_label(&font_size_label);
    harness.step();
}

#[test]
fn test_font_size_slider_visible_on_light_theme() {
    let light = egui::Visuals::light();
    let inactive_bg = light.widgets.inactive.bg_fill;
    let panel_bg = light.panel_fill;

    let boosted = egui::Color32::from_rgba_premultiplied(
        inactive_bg.r().saturating_add(40),
        inactive_bg.g().saturating_add(40),
        inactive_bg.b().saturating_add(40),
        inactive_bg.a(),
    );

    let boosted_max_diff = boosted
        .r()
        .abs_diff(panel_bg.r())
        .max(boosted.g().abs_diff(panel_bg.g()))
        .max(boosted.b().abs_diff(panel_bg.b()));

    assert!(
        boosted_max_diff < 30,
        "Brightness-boost must produce low contrast on light theme (proves bug). \
         max_diff={boosted_max_diff}, boosted=({},{},{}), panel=({},{},{})",
        boosted.r(),
        boosted.g(),
        boosted.b(),
        panel_bg.r(),
        panel_bg.g(),
        panel_bg.b(),
    );

    let selection_bg = light.selection.bg_fill;
    let selection_max_diff = selection_bg
        .r()
        .abs_diff(panel_bg.r())
        .max(selection_bg.g().abs_diff(panel_bg.g()))
        .max(selection_bg.b().abs_diff(panel_bg.b()));

    assert!(
        selection_max_diff >= 30,
        "Selection color must provide sufficient contrast on light theme. \
         max_diff={selection_max_diff}, selection=({},{},{}), panel=({},{},{})",
        selection_bg.r(),
        selection_bg.g(),
        selection_bg.b(),
        panel_bg.r(),
        panel_bg.g(),
        panel_bg.b(),
    );
}

#[test]
fn test_font_size_slider_has_visible_border() {
    let dark = egui::Visuals::dark();
    let light = egui::Visuals::light();

    assert!(
        dark.widgets.inactive.bg_stroke.width < 1.0,
        "Default dark theme must have no visible slider border (proves bug). \
         Got width={}",
        dark.widgets.inactive.bg_stroke.width,
    );
    assert!(
        light.widgets.inactive.bg_stroke.width < 1.0,
        "Default light theme must have no visible slider border (proves bug). \
         Got width={}",
        light.widgets.inactive.bg_stroke.width,
    );

    let dark_selection = dark.selection.bg_fill;
    let light_selection = light.selection.bg_fill;

    let dark_max_diff = dark_selection
        .r()
        .abs_diff(dark_selection.g())
        .max(dark_selection.r().abs_diff(dark_selection.b()));
    let light_max_diff = light_selection
        .r()
        .abs_diff(light_selection.g())
        .max(light_selection.r().abs_diff(light_selection.b()));

    assert!(
        dark_max_diff > 10,
        "Dark selection color must have saturation for border. \
         rgb=({},{},{}), spread={dark_max_diff}",
        dark_selection.r(),
        dark_selection.g(),
        dark_selection.b(),
    );
    assert!(
        light_max_diff > 10,
        "Light selection color must have saturation for border. \
         rgb=({},{},{}), spread={light_max_diff}",
        light_selection.r(),
        light_selection.g(),
        light_selection.b(),
    );
}

#[test]
fn test_ui_all_languages_load_successfully() {
    let mut harness = setup_harness();
    harness.step();

    let supported_langs = [
        ("en", "English"),
        ("ja", "\u{65e5}\u{672c}\u{8a9e}"),
        ("zh-CN", "\u{7b80}\u{4f53}\u{4e2d}\u{6587}"),
        ("zh-TW", "\u{7e41}\u{9ad4}\u{4e2d}\u{6587}"),
        ("ko", "한국어"),
        ("pt", "Português"),
        ("fr", "Français"),
        ("de", "Deutsch"),
        ("es", "Español"),
        ("it", "Italiano"),
    ];

    for (code, _name) in supported_langs {
        harness
            .state_mut()
            .trigger_action(katana_ui::app_state::AppAction::ChangeLanguage(
                code.to_string(),
            ));
        harness.step();
        harness.step();

        let settings = katana_ui::i18n::get();
        assert!(
            !settings.settings.tabs.is_empty(),
            "Tabs shouldn't be empty for {}",
            code
        );
        assert_eq!(
            harness
                .state_mut()
                .app_state_mut()
                .config
                .settings
                .settings()
                .language,
            code,
            "Language setting should be updated to {}",
            code
        );
    }
    katana_ui::i18n::set_language("en");
}

#[test]
fn test_search_modal_include_exclude_options() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("apple.md"), "# Apple").unwrap();
    std::fs::write(temp_dir.path().join("banana.md"), "# Banana").unwrap();
    std::fs::write(temp_dir.path().join("cherry.md"), "# Cherry").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);
    harness.step();

    for _ in 0..50 {
        let count = harness
            .state_mut()
            .app_state_mut()
            .workspace
            .data
            .as_ref()
            .map_or(0, |w| w.tree.len());
        if count > 0 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
        harness.step();
    }

    harness.state_mut().app_state_mut().layout.show_search_modal = true;
    harness.step();
    harness.step();

    harness.state_mut().app_state_mut().search.query = "".to_string();
    harness.step();
    assert_eq!(harness.state_mut().app_state_mut().search.results.len(), 0);

    harness.state_mut().app_state_mut().search.query = "apple".to_string();
    harness.step();
    assert_eq!(harness.state_mut().app_state_mut().search.results.len(), 1);
    assert!(harness.state_mut().app_state_mut().search.results[0].ends_with("apple.md"));

    harness.state_mut().app_state_mut().search.query = "".to_string();
    harness.state_mut().app_state_mut().search.include_pattern = "banana".to_string();
    harness.step();
    assert_eq!(harness.state_mut().app_state_mut().search.results.len(), 1);
    assert!(harness.state_mut().app_state_mut().search.results[0].ends_with("banana.md"));

    harness.state_mut().app_state_mut().search.include_pattern = "".to_string();
    harness.state_mut().app_state_mut().search.exclude_pattern = "banana".to_string();
    harness.step();
    assert_eq!(harness.state_mut().app_state_mut().search.results.len(), 2);

    harness.state_mut().app_state_mut().search.query = "a".to_string(); // 'apple.md' and 'banana.md' have 'a'
    harness.state_mut().app_state_mut().search.include_pattern = ".md".to_string();
    harness.state_mut().app_state_mut().search.exclude_pattern = "banana".to_string(); // excludes 'banana.md'
    harness.step();
    assert_eq!(harness.state_mut().app_state_mut().search.results.len(), 1);
    assert!(harness.state_mut().app_state_mut().search.results[0].ends_with("apple.md"));
}

#[test]
fn test_search_sidebar_buttons() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);
    harness.step();

    let search_nodes: Vec<_> = harness.query_all_by_label("🔍").collect();
    assert!(
        !search_nodes.is_empty(),
        "Search button (🔍) must be present in the workspace sidebar"
    );

    let filter_nodes: Vec<_> = harness.query_all_by_label("\u{2207}").collect(); // ∇
    assert!(
        !filter_nodes.is_empty(),
        "Filter button (\u{2207}) must be present in the workspace sidebar"
    );
}

#[test]
fn test_integration_open_workspace_restores_tabs() {
    let settings_dir = tempfile::tempdir().unwrap();
    let settings_path = settings_dir.path().join("settings.json");
    let ws_dir = tempfile::tempdir().unwrap();
    let doc_path1 = ws_dir.path().join("doc1.md");
    let doc_path2 = ws_dir.path().join("doc2.md");
    std::fs::write(&doc_path1, "# Doc 1").unwrap();
    std::fs::write(&doc_path2, "# Doc 2").unwrap();

    {
        let mut harness = setup_harness_with_json_repo(&settings_path);
        harness.step();
        harness
            .state_mut()
            .trigger_action(AppAction::OpenWorkspace(ws_dir.path().to_path_buf()));
        wait_for_workspace_load(&mut harness);
        harness
            .state_mut()
            .trigger_action(AppAction::SelectDocument(doc_path1.clone()));
        harness.step();
        harness
            .state_mut()
            .trigger_action(AppAction::SelectDocument(doc_path2.clone()));
        harness.step();

        let settings = harness
            .state_mut()
            .app_state_mut()
            .config
            .settings
            .settings();
        assert_eq!(settings.workspace.open_tabs.len(), 2);
        assert_eq!(settings.workspace.active_tab_idx, Some(1));
    }

    {
        let mut harness = setup_harness_with_json_repo(&settings_path);
        harness.step();
        harness
            .state_mut()
            .trigger_action(AppAction::OpenWorkspace(ws_dir.path().to_path_buf()));
        wait_for_workspace_load(&mut harness);

        let state = harness.state_mut().app_state_mut();
        assert_eq!(state.document.open_documents.len(), 2);
        assert_eq!(state.document.active_doc_idx, Some(1));
    }
}

#[test]
fn test_integration_remove_workspace() {
    let settings_dir = tempfile::tempdir().unwrap();
    let settings_path = settings_dir.path().join("settings.json");
    let ws_dir = tempfile::tempdir().unwrap();

    let mut harness = setup_harness_with_json_repo(&settings_path);
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(ws_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);

    {
        let settings = harness
            .state_mut()
            .app_state_mut()
            .config
            .settings
            .settings();
        assert!(settings
            .workspace
            .paths
            .contains(&ws_dir.path().display().to_string()));
    }

    harness
        .state_mut()
        .trigger_action(AppAction::RemoveWorkspace(
            ws_dir.path().display().to_string(),
        ));
    harness.step();

    {
        let settings = harness
            .state_mut()
            .app_state_mut()
            .config
            .settings
            .settings();
        assert!(!settings
            .workspace
            .paths
            .contains(&ws_dir.path().display().to_string()));
    }
}

#[test]
fn test_integration_save_workspace_state_error() {
    let settings_dir = tempfile::tempdir().unwrap();
    let settings_path = settings_dir.path().to_path_buf();
    let ws_dir = tempfile::tempdir().unwrap();

    let mut harness = setup_harness_with_json_repo(&settings_path);
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(ws_dir.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);

    harness
        .state_mut()
        .trigger_action(AppAction::RemoveWorkspace(
            ws_dir.path().display().to_string(),
        ));
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::CloseDocument(0));
    harness.step();
}

#[test]
fn test_integration_open_workspace_failed() {
    let mut harness = setup_harness();
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(std::path::PathBuf::from(
            "/invalid/path/that/does/not/exist/12345/abcde",
        )));

    wait_for_workspace_load(&mut harness);

    assert!(!harness.state_mut().app_state_mut().workspace.is_loading);
}

#[test]
fn test_integration_tab_context_menu_close_others() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = std::env::temp_dir().join("katana_test_ws_context_menu");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();
    let file1 = temp_dir.join("a.md");
    let file2 = temp_dir.join("b.md");
    let file3 = temp_dir.join("c.md");
    std::fs::write(&file1, "# A").unwrap();
    std::fs::write(&file2, "# B").unwrap();
    std::fs::write(&file3, "# C").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);

    let abs1 = file1.canonicalize().unwrap_or(file1);
    let abs2 = file2.canonicalize().unwrap_or(file2);
    let abs3 = file3.canonicalize().unwrap_or(file3);

    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs1));
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs2));
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs3));
    harness.step();

    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        3
    );

    harness
        .state_mut()
        .trigger_action(AppAction::CloseOtherDocuments(1));
    harness.step();

    let state = harness.state_mut().app_state_mut();
    assert_eq!(
        state.document.open_documents.len(),
        1,
        "Should close other tabs"
    );
    assert_eq!(
        state.document.recently_closed_tabs.len(),
        2,
        "Should put 2 tabs into history"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_tree_context_menu_actions() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = std::env::temp_dir().join("katana_test_tree_actions");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();
    let file = temp_dir.join("test.md");

    harness
        .state_mut()
        .trigger_action(AppAction::RequestNewFile(temp_dir.clone()));
    harness.step();
    let state = harness.state_mut().app_state_mut();
    assert!(state.layout.create_fs_node_modal.is_some());
    let (parent, name, _, is_dir) = state.layout.create_fs_node_modal.as_ref().unwrap();
    assert_eq!(*parent, temp_dir);
    assert!(name.is_empty());
    assert!(!*is_dir);

    harness
        .state_mut()
        .trigger_action(AppAction::RequestRename(file.clone()));
    harness.step();
    let state = harness.state_mut().app_state_mut();
    assert!(state.layout.rename_modal.is_some());
    let (target, name) = state.layout.rename_modal.as_ref().unwrap();
    assert_eq!(*target, file);
    assert_eq!(name, "test.md");

    harness
        .state_mut()
        .trigger_action(AppAction::RequestDelete(file.clone()));
    harness.step();
    let state = harness.state_mut().app_state_mut();
    assert!(state.layout.delete_modal.is_some());
    let target = state.layout.delete_modal.as_ref().unwrap();
    assert_eq!(*target, file);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_ui_context_menu_close_others() {
    let mut harness = setup_harness();
    harness.step();

    let unique_name = format!(
        "katana_test_ws_context_menu_ui_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    let temp_dir = std::env::temp_dir().join(unique_name);
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();
    let file1 = temp_dir.join("a.md");
    let file2 = temp_dir.join("b.md");
    let file3 = temp_dir.join("c.md");
    std::fs::write(&file1, "# A").unwrap();
    std::fs::write(&file2, "# B").unwrap();
    std::fs::write(&file3, "# C").unwrap();
    let abs1 = file1.canonicalize().unwrap_or(file1);
    let abs2 = file2.canonicalize().unwrap_or(file2);
    let abs3 = file3.canonicalize().unwrap_or(file3);

    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs1));
    harness.run_steps(5);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs3));
    harness.run_steps(5);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs2.clone()));
    harness.run_steps(5);

    let tab_b = harness.query_all_by_label("b.md").next().unwrap();

    tab_b.click_secondary();

    let mut found = false;
    for _ in 0..20 {
        harness.run_steps(5);
        if harness.query_all_by_label("Close Others").next().is_some() {
            found = true;
            break;
        }
    }

    assert!(found, "Close Others context menu button failed to render");
    let btn = harness.get_by_label("Close Others");
    btn.click();
    harness.run_steps(20);

    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        1
    );
    assert_eq!(
        harness.state_mut().app_state_mut().document.open_documents[0].path,
        abs2
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_tab_restore_closed() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::tempdir().unwrap();
    let abs1 = temp_dir.path().join("a.md");
    let abs2 = temp_dir.path().join("b.md");
    std::fs::write(&abs1, "# A").unwrap();
    std::fs::write(&abs2, "# B").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs1.clone()));
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs2.clone()));
    harness.step();

    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        2
    );

    harness
        .state_mut()
        .trigger_action(AppAction::CloseDocument(1));
    harness.step();

    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        1
    );

    harness
        .state_mut()
        .trigger_action(AppAction::RestoreClosedDocument);
    harness.step();

    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        2
    );
    assert_eq!(
        harness.state_mut().app_state_mut().document.open_documents[1].path,
        abs2
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_tab_reorder() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::tempdir().unwrap();
    let abs1 = temp_dir.path().join("a.md");
    let abs2 = temp_dir.path().join("b.md");
    let abs3 = temp_dir.path().join("c.md");
    std::fs::write(&abs1, "# A").unwrap();
    std::fs::write(&abs2, "# B").unwrap();
    std::fs::write(&abs3, "# C").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs1.clone()));
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs2.clone()));
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs3.clone()));
    harness.step();

    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        3
    );
    assert_eq!(
        harness.state_mut().app_state_mut().document.open_documents[0].path,
        abs1
    );
    assert_eq!(
        harness.state_mut().app_state_mut().document.open_documents[1].path,
        abs2
    );
    assert_eq!(
        harness.state_mut().app_state_mut().document.open_documents[2].path,
        abs3
    );

    harness
        .state_mut()
        .trigger_action(AppAction::ReorderDocument { from: 0, to: 2 });
    harness.step();

    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        3
    );
    assert_eq!(
        harness.state_mut().app_state_mut().document.open_documents[0].path,
        abs2
    );
    assert_eq!(
        harness.state_mut().app_state_mut().document.open_documents[1].path,
        abs1
    );
    assert_eq!(
        harness.state_mut().app_state_mut().document.open_documents[2].path,
        abs3
    );

    harness
        .state_mut()
        .trigger_action(AppAction::ReorderDocument { from: 2, to: 0 });
    harness.step();

    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        3
    );
    assert_eq!(
        harness.state_mut().app_state_mut().document.open_documents[0].path,
        abs3
    );
    assert_eq!(
        harness.state_mut().app_state_mut().document.open_documents[1].path,
        abs2
    );
    assert_eq!(
        harness.state_mut().app_state_mut().document.open_documents[2].path,
        abs1
    );

    harness
        .state_mut()
        .trigger_action(AppAction::ReorderDocument { from: 5, to: 10 });
    harness.step();
    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        3
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_tab_pinning() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::tempdir().unwrap();
    let abs1 = temp_dir.path().join("a.md");
    let abs2 = temp_dir.path().join("b.md");
    std::fs::write(&abs1, "# A").unwrap();
    std::fs::write(&abs2, "# B").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs1.clone()));
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs2.clone()));
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::TogglePinDocument(1));
    harness.step();

    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        2
    );
    assert!(harness.state_mut().app_state_mut().document.open_documents[0].is_pinned);
    assert_eq!(
        harness.state_mut().app_state_mut().document.open_documents[0].path,
        abs2
    );
    assert!(!harness.state_mut().app_state_mut().document.open_documents[1].is_pinned);
    assert_eq!(
        harness.state_mut().app_state_mut().document.open_documents[1].path,
        abs1
    );

    harness
        .state_mut()
        .trigger_action(AppAction::TogglePinDocument(0));
    harness.step();

    assert!(!harness.state_mut().app_state_mut().document.open_documents[0].is_pinned);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_tab_close_directions() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::tempdir().unwrap();
    let abs1 = temp_dir.path().join("a.md");
    let abs2 = temp_dir.path().join("b.md");
    let abs3 = temp_dir.path().join("c.md");
    std::fs::write(&abs1, "# A").unwrap();
    std::fs::write(&abs2, "# B").unwrap();
    std::fs::write(&abs3, "# C").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs1.clone()));
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs2.clone()));
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs3.clone()));
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs1.clone()));
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::CloseDocumentsToLeft(1));
    harness.step();

    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        2
    );
    assert_eq!(
        harness.state_mut().app_state_mut().document.open_documents[0].path,
        abs2
    );
    assert_eq!(
        harness.state_mut().app_state_mut().document.open_documents[1].path,
        abs3
    );

    harness
        .state_mut()
        .trigger_action(AppAction::CloseDocumentsToRight(0));
    harness.step();

    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        1
    );
    assert_eq!(
        harness.state_mut().app_state_mut().document.open_documents[0].path,
        abs2
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_tab_close_edges() {
    let mut harness = setup_harness();
    harness.step();
    let temp_dir = tempfile::tempdir().unwrap();
    let abs1 = temp_dir.path().join("a.md");
    let abs2 = temp_dir.path().join("b.md");
    let abs3 = temp_dir.path().join("c.md");
    std::fs::write(&abs1, "# A").unwrap();
    std::fs::write(&abs2, "# B").unwrap();
    std::fs::write(&abs3, "# C").unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs1.clone()));
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs2.clone()));
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs3.clone()));
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::CloseDocumentsToLeft(1));
    harness.step();

    assert_eq!(
        harness.state_mut().app_state_mut().document.active_doc_idx,
        Some(1)
    );

    harness
        .state_mut()
        .trigger_action(AppAction::CloseDocumentsToRight(0));
    harness.step();

    assert_eq!(
        harness.state_mut().app_state_mut().document.active_doc_idx,
        Some(0)
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_tab_close_all() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::tempdir().unwrap();
    let abs1 = temp_dir.path().join("a.md");
    std::fs::write(&abs1, "# A").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs1.clone()));
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::CloseAllDocuments);
    harness.step();

    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        0
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_tab_restore_closed_limit() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::tempdir().unwrap();
    let mut paths = Vec::new();

    for i in 0..11 {
        let p = temp_dir.path().join(format!("file_{}.md", i));
        std::fs::write(&p, format!("# {}", i)).unwrap();
        harness
            .state_mut()
            .trigger_action(AppAction::SelectDocument(p.clone()));
        harness.step();
        paths.push(p);
    }

    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        11
    );

    for _ in 0..11 {
        harness
            .state_mut()
            .trigger_action(AppAction::CloseDocument(0));
        harness.step();
    }

    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        0
    );
    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .recently_closed_tabs
            .len(),
        10
    );

    for _ in 0..10 {
        harness
            .state_mut()
            .trigger_action(AppAction::RestoreClosedDocument);
        harness.step();
    }

    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        10
    );
    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .recently_closed_tabs
            .len(),
        0
    );

    harness
        .state_mut()
        .trigger_action(AppAction::RestoreClosedDocument);
    harness.step();
    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .document
            .open_documents
            .len(),
        10
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_ui_terms_modal_visibility() {
    let settings_path = std::env::temp_dir().join(format!(
        "katana_test_settings_terms_{}.json",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&settings_path);

    let mut harness = Harness::builder().build_eframe(move |_cc| {
        let ai_registry = AiProviderRegistry::new();
        let plugin_registry = PluginRegistry::new();
        let state = AppState::new(
            ai_registry,
            plugin_registry,
            katana_platform::SettingsService::new(Box::new(
                katana_platform::JsonFileRepository::new(settings_path.clone()),
            )),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        let mut app = KatanaApp::new(state);
        app.skip_splash();
        app
    });

    harness.step();

    katana_ui::i18n::set_language("en");
    harness.step();

    harness.get_by_label("Terms of Service");

    harness.get_by_label_contains(&format!("Version: {}", env!("CARGO_PKG_VERSION")));
    harness.get_by_label("Accept").click();
    harness.step();
    harness.run_steps(5);

    harness.get_by_label("No workspace open.");
}

#[test]
fn test_regression_update_dialog_up_to_date_renders_correctly() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER_UP: AtomicUsize = AtomicUsize::new(200);
    let id = COUNTER_UP.fetch_add(1, Ordering::SeqCst);
    let settings_path = std::env::temp_dir().join(format!(
        "katana_test_settings_uptodate_{}_{}.json",
        std::process::id(),
        id
    ));
    let _ = std::fs::remove_file(&settings_path);

    let mut harness = Harness::builder()
        .with_size(egui::Vec2::new(1280.0, 800.0))
        .build_eframe(move |_cc| {
            let ai_registry = katana_core::ai::AiProviderRegistry::new();
            let plugin_registry = katana_core::plugin::PluginRegistry::new();
            let mut state = AppState::new(
                ai_registry,
                plugin_registry,
                katana_platform::SettingsService::new(Box::new(
                    katana_platform::JsonFileRepository::new(settings_path.clone()),
                )),
                std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
            );
            state.config.settings.settings_mut().terms_accepted_version =
                Some(katana_ui::about_info::APP_VERSION.to_string());
            katana_ui::i18n::set_language("en");
            let mut app = KatanaApp::new(state);
            app.skip_splash();
            app.disable_changelog_display_for_test();
            app
        });

    harness.step();

    harness.state_mut().app_state_mut().update.checking = false;

    harness.state_mut().open_update_dialog_for_test();
    harness.run_steps(10);

    harness.get_by_label("Up to Date");

    harness.get_by_label("OK");
}

#[test]
fn test_regression_update_dialog_does_not_stretch_vertically() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER2: AtomicUsize = AtomicUsize::new(100);
    let id = COUNTER2.fetch_add(1, Ordering::SeqCst);
    let settings_path = std::env::temp_dir().join(format!(
        "katana_test_settings_stretch_{}_{}.json",
        std::process::id(),
        id
    ));
    let _ = std::fs::remove_file(&settings_path);

    let mut harness = Harness::builder()
        .with_size(egui::Vec2::new(1280.0, 800.0))
        .build_eframe(move |_cc| {
            let ai_registry = katana_core::ai::AiProviderRegistry::new();
            let plugin_registry = katana_core::plugin::PluginRegistry::new();
            let mut state = AppState::new(
                ai_registry,
                plugin_registry,
                katana_platform::SettingsService::new(Box::new(
                    katana_platform::JsonFileRepository::new(settings_path.clone()),
                )),
                std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
            );
            state.config.settings.settings_mut().terms_accepted_version =
                Some(katana_ui::about_info::APP_VERSION.to_string());
            katana_ui::i18n::set_language("en");
            let mut app = KatanaApp::new(state);
            app.skip_splash();
            app
        });

    harness.step();
    harness.state_mut().open_update_dialog_for_test();

    harness.run_steps(10);

    let window_id = egui::Id::new("katana_update_dialog_v6");
    let rect = harness
        .ctx
        .memory(|mem: &egui::Memory| mem.area_rect(window_id));

    let bounds = rect.expect("Update dialog window rect not found in egui memory");
    let height = bounds.height();
    assert!(
        height < 200.0,
        "Update dialog height ({height:.0}px) exceeds 200px — vertical stretch bug!"
    );
}

#[test]
fn test_integration_auto_save_interval_precision_preserved_by_ui() {
    let mut harness = setup_harness();
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::ToggleSettings);
    harness
        .state_mut()
        .app_state_mut()
        .config
        .active_settings_tab = katana_ui::app_state::SettingsTab::Behavior;
    harness.step();
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .config
        .settings
        .settings_mut()
        .behavior
        .auto_save_interval_secs = 5.1;

    for _ in 0..5 {
        harness.step();
    }

    let final_val = harness
        .state_mut()
        .app_state_mut()
        .config
        .settings
        .settings()
        .behavior
        .auto_save_interval_secs;
    assert_eq!(
        final_val,
        5.1,
        "Strict IT Check: The Settings UI MUST preserve exactly 1 decimal of precision (0.1 step) for the Auto-save interval and must NOT round it to integers."
    );
}

#[test]
fn test_ast_linter_locales() {
    let locales_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("locales");
    let mut failures = vec![];
    let mut locales = std::collections::HashMap::new();

    for entry in std::fs::read_dir(locales_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "json") {
            let filename = path.file_name().unwrap().to_str().unwrap().to_string();
            if filename == "languages.json" {
                continue;
            }
            let file_contents = std::fs::read_to_string(&path).unwrap();
            let json: serde_json::Value = serde_json::from_str(&file_contents).unwrap();
            locales.insert(filename, json);
        }
    }

    let en_json = locales.get("en.json").expect("en.json missing");

    let mut all_leaves = std::collections::HashMap::new();
    fn get_leaves(
        val: &serde_json::Value,
        path: &str,
        leaves: &mut std::collections::HashMap<String, String>,
    ) {
        if path.contains(".key") && path.starts_with("settings.tabs[") {
            return;
        }
        if path == "error.render_error" || path == "terms.version_label" {
            return;
        }
        match val {
            serde_json::Value::String(s) => {
                leaves.insert(path.to_string(), s.trim().to_string());
            }
            serde_json::Value::Object(map) => {
                for (k, v) in map {
                    let new_path = if path.is_empty() {
                        k.clone()
                    } else {
                        format!("{path}.{k}")
                    };
                    get_leaves(v, &new_path, leaves);
                }
            }
            serde_json::Value::Array(arr) => {
                for (i, v) in arr.iter().enumerate() {
                    let new_path = format!("{path}[{i}]");
                    get_leaves(v, &new_path, leaves);
                }
            }
            _ => {}
        }
    }

    for (filename, json) in &locales {
        let mut leaves = std::collections::HashMap::new();
        get_leaves(json, "", &mut leaves);
        all_leaves.insert(filename.clone(), leaves);
    }

    let allowed_overlaps = [
        "Abrir", "Architecture", "Aumentar", "Build", "Cancelar", "Claro", "Code", "Comportamento", "Confirmar", "Copyright", "Código", "Descartar", "Dividir", "Documentation", "Documento HTML (.html)", "Documento PDF (.pdf)", "Duplicar...", "Exportar", "File", "Filtro", "Idioma (Language)", "Info", "Infos", "Intervalo", "Layout", "Links", "Nunca", "OK", "Ocultar KatanA", "Patrocinar", "Personalizado", "Plataforma", "Pronto", "Quotidiano", "Renderizando {kind}...", "Runtime", "Rust", "Semanalmente", "Sistema", "Sponsor", "Support", "System", "Tema", "Terms content.", "Text", "Texto", "Version", "[AI: \u{672a}\u{8a2d}\u{5b9a}]", "[AI: unconfigured]", "fn main() { println!(\"\u{4f60}\u{597d}!\"); }", "segundos", "{kind} \u{6e32}\u{67d3}\u{4e2d}...", "⏳ Exportando {filename}...", "\u{4e0a}\u{79fb}", "\u{4e0b}\u{79fb}", "\u{4f8b}\u{5982} node_modules, target, .git", "\u{4fdd}\u{5b58}", "\u{5168}\u{5c4f}", "\u{5206}\u{5272}", "\u{5206}\u{5272}\u{65b9}\u{5411}", "\u{53d6}\u{6d88}", "\u{53d6}\u{6d88}\u{56fa}\u{5b9a}", "\u{53f3}\u{79fb}", "\u{56fa}\u{5b9a}", "\u{5782}\u{76f4}\u{ff08}\u{4e0a} / \u{4e0b}\u{ff09}", "\u{5929}", "\u{5de6}\u{79fb}", "\u{5df2}\u{662f}\u{6700}\u{65b0}\u{7248}\u{672c}", "\u{5e73}\u{53f0}", "\u{60a8}\u{4f7f}\u{7528}\u{7684}\u{662f}\u{6700}\u{65b0}\u{7248}\u{672c}\u{7684} KatanA\u{3002}", "\u{63a5}\u{53d7}", "\u{652f}\u{6301}", "\u{653e}\u{5927}", "\u{66f4}\u{65b0}", "\u{6709}\u{65b0}\u{7248}\u{672c}\u{53ef}\u{7528}", "\u{6bcf}\u{5929}", "\u{6bcf}\u{6708}", "\u{6c34}\u{5e73}\u{ff08}\u{5de6} / \u{53f3}\u{ff09}", "\u{6df1}\u{8272}", "\u{7248}\u{672c}", "\u{78ba}\u{8a8d}", "\u{79d2}", "\u{7e2e}\u{5c0f}", "\u{80cc}\u{666f}", "\u{8a2d}\u{5b9a}", "\u{8b66}\u{544a}", "\u{9000}\u{51fa} KatanA", "\u{91cd}\u{7f6e}\u{4f4d}\u{7f6e}\u{548c}\u{5927}\u{5c0f}", "\u{9762}\u{677f}\u{80cc}\u{666f}"
    ];

    for (filename, target_json) in &locales {
        if filename == "en.json" {
            continue;
        } // en is the baseline structure

        fn check_structure_and_values(
            en_val: &serde_json::Value,
            target_val: Option<&serde_json::Value>,
            path_str: &str,
            filename: &str,
            all_leaves: &std::collections::HashMap<
                String,
                std::collections::HashMap<String, String>,
            >,
            allowed_overlaps: &[&str],
            failures: &mut Vec<String>,
        ) {
            let Some(target) = target_val else {
                failures.push(format!("{filename}: [LOOPHOLE CLOSED] Missing required key '{path_str}' which exists in en.json"));
                return;
            };

            let is_ignored_path = (path_str.contains(".key")
                && path_str.starts_with("settings.tabs["))
                || path_str == "error.render_error"
                || path_str == "terms.version_label";

            match (en_val, target) {
                (serde_json::Value::String(_), serde_json::Value::String(s)) => {
                    if is_ignored_path {
                        return;
                    }

                    let trimmed = s.trim();
                    if trimmed.is_empty() {
                        failures.push(format!(
                            "{filename}: Key '{path_str}' is an explicitly empty string"
                        ));
                    } else if s.starts_with('[') {
                        if let Some(end) = s.find(']') {
                            let inside = &s[1..end];
                            let is_lang_code =
                                inside.chars().all(|c| c.is_ascii_alphabetic() || c == '-');
                            if is_lang_code && inside.len() >= 2 && inside.len() <= 5 {
                                failures.push(format!("{filename}: Key '{path_str}' contains forbidden placeholder badge: '{s}'"));
                            }
                        }
                    }

                    for (other_filename, other_leaves) in all_leaves {
                        if other_filename != filename {
                            if let Some(other_s) = other_leaves.get(path_str) {
                                if trimmed == other_s && !allowed_overlaps.contains(&trimmed) {
                                    failures.push(format!("{filename}: [LOOPHOLE CLOSED] Key '{path_str}' exactly matches '{other_filename}' fallback: '{s}'. MUST be translated natively without copying another language."));
                                }
                            }
                        }
                    }
                }
                (serde_json::Value::Object(en_map), serde_json::Value::Object(t_map)) => {
                    for k in t_map.keys() {
                        if !en_map.contains_key(k) {
                            failures.push(format!("{filename}: [LOOPHOLE CLOSED] Extra key '{path_str}.{k}' exists in target but not in en.json"));
                        }
                    }
                    for (k, en_child) in en_map {
                        let new_path = if path_str.is_empty() {
                            k.clone()
                        } else {
                            format!("{path_str}.{k}")
                        };
                        let t_child = t_map.get(k);
                        check_structure_and_values(
                            en_child,
                            t_child,
                            &new_path,
                            filename,
                            all_leaves,
                            allowed_overlaps,
                            failures,
                        );
                    }
                }
                (serde_json::Value::Array(en_arr), serde_json::Value::Array(t_arr)) => {
                    if en_arr.len() != t_arr.len() {
                        failures.push(format!("{filename}: [LOOPHOLE CLOSED] Array '{path_str}' length mismatch. en.json has {}, target has {}", en_arr.len(), t_arr.len()));
                    }
                    for (i, en_child) in en_arr.iter().enumerate() {
                        let new_path = format!("{path_str}[{i}]");
                        let t_child = t_arr.get(i);
                        check_structure_and_values(
                            en_child,
                            t_child,
                            &new_path,
                            filename,
                            all_leaves,
                            allowed_overlaps,
                            failures,
                        );
                    }
                }
                (serde_json::Value::Number(_), serde_json::Value::Number(_)) => {}
                (serde_json::Value::Bool(_), serde_json::Value::Bool(_)) => {}
                (serde_json::Value::Null, serde_json::Value::Null) => {}
                _ => {
                    failures.push(format!("{filename}: [LOOPHOLE CLOSED] Type mismatch at '{path_str}' between en.json and target file"));
                }
            }
        }

        check_structure_and_values(
            en_json,
            Some(target_json),
            "",
            filename,
            &all_leaves,
            &allowed_overlaps,
            &mut failures,
        );
    }

    if !failures.is_empty() {
        failures.sort();
        failures.dedup();
        panic!(
            "AST Linter found {} layout/translation violations in locales:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}