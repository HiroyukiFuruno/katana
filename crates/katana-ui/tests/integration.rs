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
