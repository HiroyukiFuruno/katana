use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
use katana_ui::app_state::{AppAction, AppState, ViewMode};
use katana_ui::shell::KatanaApp;

fn setup_harness() -> Harness<'static, KatanaApp> {
    Harness::builder().build_eframe(|_cc| {
        let ai_registry = AiProviderRegistry::new();
        let plugin_registry = PluginRegistry::new();
        let state = AppState::new(ai_registry, plugin_registry);
        katana_ui::i18n::set_language("en");
        KatanaApp::new(state)
    })
}

#[test]
fn test_integration_application_startup() {
    let mut harness = setup_harness();
    harness.step();
    let _node = harness.get_by_label("No workspace open.");
    harness.snapshot("startup_screen");
}

#[test]
fn test_integration_workspace_and_tabs() {
    let mut harness = setup_harness();
    harness.step();

    // Create a temporary directory and file
    let temp_dir = std::env::temp_dir().join("katana_test_ws");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();
    let test_file = temp_dir.join("test1.md");
    std::fs::write(&test_file, "# Hello Katana").unwrap();

    // Inject AppAction to simulate open workspace
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    harness.step();

    // Check if the tree shows the file test1.md
    let _file_node = harness.get_all_by_value("📄 test1.md").next().unwrap();

    // Click it to open it
    harness
        .get_all_by_value("📄 test1.md")
        .next()
        .unwrap()
        .click();
    harness.step();

    // Verify it opened and editor handles it (Title will be "katana — test1.md")
    // In kittest, `kittest::Queryable` can query values. Let's just do a snapshot.
    harness.snapshot("editor_opened");

    // Close the document (tab 'x' button or close action)
    harness
        .state_mut()
        .trigger_action(AppAction::CloseDocument(0));
    harness.step();

    // Tab is closed, fallback to workspace view
    harness.snapshot("editor_closed");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_view_modes() {
    let mut harness = setup_harness();
    harness.step();

    // Open workspace with a file
    let temp_dir = std::env::temp_dir().join("katana_test_ws_modes");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();
    let test_file = temp_dir.join("test_modes.md");
    std::fs::write(&test_file, "# Hello View Modes\n**Bold text here.**").unwrap();

    // Inject Open & Select Document
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    harness.step();
    // Use the specific file path
    let abs_path = test_file.canonicalize().unwrap_or(test_file);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path));
    harness.step();

    // Switch to Preview Only
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::PreviewOnly);
    harness.step();
    harness.snapshot("view_mode_preview_only");

    // Switch to Split
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step();
    harness.snapshot("view_mode_split");

    // Switch to Code Only
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::CodeOnly);
    harness.step();
    harness.snapshot("view_mode_code_only");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

// Additional integration tests to cover more shell.rs branches

// Test UpdateBuffer action (shell.rs L886)
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
    harness.step();
    let abs_path = test_file.canonicalize().unwrap_or(test_file);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path));
    harness.step();

    // Update buffer
    harness.state_mut().trigger_action(AppAction::UpdateBuffer(
        "# Updated\n\nNew content".to_string(),
    ));
    harness.step();
    harness.snapshot("buffer_updated");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

// Test SaveDocument action (shell.rs save flow)
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
    harness.step();
    let abs_path = test_file.canonicalize().unwrap_or(test_file.clone());
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path.clone()));
    harness.step();

    // Update then save
    harness
        .state_mut()
        .trigger_action(AppAction::UpdateBuffer("# Saved Content".to_string()));
    harness.step();
    harness.state_mut().trigger_action(AppAction::SaveDocument);
    harness.step();
    harness.snapshot("document_saved");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

// Test SelectDocument when multiple docs (tab bar navigation coverage)
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
    harness.step();

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

    harness.snapshot("multiple_docs_open");

    // Close document 1
    harness
        .state_mut()
        .trigger_action(AppAction::CloseDocument(0));
    harness.step();
    harness.snapshot("after_close_first");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

// Test preview pane with mermaid/drawio content (cover preview_pane.rs branches)
#[test]
fn test_integration_preview_with_diagram_content() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = std::env::temp_dir().join("katana_test_ws_diag");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();
    let test_file = temp_dir.join("diagram_test.md");
    let content = "# Diagram Test\n\n```mermaid\ngraph TD; A-->B\n```\n\n```drawio\n<mxGraphModel><root><mxCell id=\"0\"/></root></mxGraphModel>\n```\n";
    std::fs::write(&test_file, content).unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    harness.step();
    let abs_path = test_file.canonicalize().unwrap_or(test_file);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path));
    harness.step();

    // In Preview Only mode to exercise preview_pane heavily
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::PreviewOnly);
    harness.step();
    harness.snapshot("preview_with_diagrams");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

// Test with directory containing subdirectory (shell.rs: tree with Directory entries)
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
    harness.step();
    harness.snapshot("workspace_with_subdirs");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

// ワークスペースパネル折りたたみUIをカバー (shell.rs: L394-407)
#[test]
fn test_integration_workspace_panel_collapsed() {
    let mut harness = setup_harness();
    harness.step();

    // show_workspace を false に設定してから描画
    harness.state_mut().app_state_mut().show_workspace = false;
    harness.step();
    harness.snapshot("workspace_collapsed");

    // 「›」展開ボタンをkittestでクリックを試みる (shell.rs L403-404をカバー)
    // ボタンが見つからない場合はスキップする（kittestではボタンの文字列が
    // Unicode形式で比較されるため見つからないこともある）
    {
        use egui_kittest::kittest::Queryable;
        for label in ["›", ">", "❯"] {
            // query_all_by_value はパニックしないため、見つからない場合は空イテレータを返す
            let nodes: Vec<_> = harness.query_all_by_value(label).collect();
            if let Some(node) = nodes.into_iter().next() {
                node.click();
                harness.step();
                break;
            }
        }
    }

    harness.state_mut().app_state_mut().show_workspace = true;
    harness.step();
}

// Split モードでエディタとプレビューを同時表示 (shell.rs: L604-)
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
    harness.step();

    let abs_path = test_file.canonicalize().unwrap_or(test_file);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path));
    harness.step();

    // Split モードに切り替え
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step();
    harness.snapshot("split_mode");

    // Code Only モードに切り替え
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::CodeOnly);
    harness.step();
    harness.snapshot("code_only_mode");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

// タブを複数開いて閉じる（CloseDocumentの多タブ処理）
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
    harness.step();

    // File 1 を開く
    let p1 = temp_dir
        .join("file1.md")
        .canonicalize()
        .unwrap_or_else(|_| temp_dir.join("file1.md"));
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(p1));
    harness.step();

    // File 2 を開く
    let p2 = temp_dir
        .join("file2.md")
        .canonicalize()
        .unwrap_or_else(|_| temp_dir.join("file2.md"));
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(p2));
    harness.step();

    harness.snapshot("two_tabs_open");

    // タブ 0 を閉じる（残ったタブが active_doc_idx を適切に更新する）
    harness
        .state_mut()
        .trigger_action(AppAction::CloseDocument(0));
    harness.step();

    // タブ 0 を閉じる（最後のタブ）
    harness
        .state_mut()
        .trigger_action(AppAction::CloseDocument(0));
    harness.step();
    harness.snapshot("all_tabs_closed");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

// ワークスペースの force_tree_open トグル（tree 全展開/全折畳）
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
    harness.step();

    // 全展開
    harness.state_mut().app_state_mut().force_tree_open = Some(true);
    harness.step();

    // 全折畳
    harness.state_mut().app_state_mut().force_tree_open = Some(false);
    harness.step();
    harness.snapshot("tree_collapse");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

// PreviewOnly モードでドキュメントが選択されていない場合の UI (shell.rs L490-492)
#[test]
fn test_integration_preview_only_no_document() {
    let mut harness = setup_harness();
    harness.step();

    // PreviewOnly では active_document が None の場合 "no_document_selected" を表示
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(katana_ui::app_state::ViewMode::PreviewOnly);
    harness.step();
    harness.snapshot("preview_only_no_doc");
}

// RefreshDiagrams アクションが処理される (shell.rs L542相当)
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
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(md_path));
    harness.step();

    // RefreshDiagrams アクションを発行
    harness
        .state_mut()
        .trigger_action(AppAction::RefreshDiagrams);
    harness.step();
}
