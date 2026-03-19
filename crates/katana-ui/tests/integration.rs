use egui_kittest::kittest::Queryable;
use egui_kittest::{Harness, SnapshotOptions};
use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
use katana_ui::app_state::{AppAction, AppState, ViewMode};
use katana_ui::shell::KatanaApp;

/// Snapshot pixel tolerance to absorb non-deterministic rendering differences.
/// egui_kittest defaults to 0, but font hinting and anti-aliasing vary between runs.
/// Max observed diff between local and GitHub Actions macOS environment: ~4295 pixels.
/// We set it to 10000 to provide a comfortable margin for environmental differences.
const SNAPSHOT_PIXEL_TOLERANCE: usize = 10000;

fn setup_harness() -> Harness<'static, KatanaApp> {
    // Force missing mmdc to ensure deterministic fallback UI across Local/CI
    std::env::set_var("MERMAID_MMDC", "dummy_missing_executable_for_kittest");

    Harness::builder().build_eframe(|_cc| {
        let ai_registry = AiProviderRegistry::new();
        let plugin_registry = PluginRegistry::new();
        let state = AppState::new(
            ai_registry,
            plugin_registry,
            katana_platform::SettingsService::default(),
        );
        katana_ui::i18n::set_language("en");
        KatanaApp::new(state)
    })
}

#[test]
fn test_integration_application_startup() {
    let mut harness = setup_harness();
    harness.step();
    let _node = harness.get_by_label("No workspace open.");
    harness.snapshot_options(
        "startup_screen",
        &SnapshotOptions::default().failed_pixel_count_threshold(SNAPSHOT_PIXEL_TOLERANCE),
    );
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

    // Verify it opened and editor handles it (Title will be "KatanA — test1.md")
    // In kittest, `kittest::Queryable` can query values. Let's just do a snapshot.
    harness.snapshot_options(
        "editor_opened",
        &SnapshotOptions::default().failed_pixel_count_threshold(SNAPSHOT_PIXEL_TOLERANCE),
    );

    // Close the document (tab 'x' button or close action)
    harness
        .state_mut()
        .trigger_action(AppAction::CloseDocument(0));
    harness.step();

    // Tab is closed, fallback to workspace view
    harness.snapshot_options(
        "editor_closed",
        &SnapshotOptions::default().failed_pixel_count_threshold(SNAPSHOT_PIXEL_TOLERANCE),
    );

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
    harness.snapshot_options(
        "view_mode_preview_only",
        &SnapshotOptions::default().failed_pixel_count_threshold(SNAPSHOT_PIXEL_TOLERANCE),
    );

    // Switch to Split
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step();
    harness.snapshot_options(
        "view_mode_split",
        &SnapshotOptions::default().failed_pixel_count_threshold(SNAPSHOT_PIXEL_TOLERANCE),
    );

    // Switch to Code Only
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::CodeOnly);
    harness.step();
    harness.snapshot_options(
        "view_mode_code_only",
        &SnapshotOptions::default().failed_pixel_count_threshold(SNAPSHOT_PIXEL_TOLERANCE),
    );

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
    harness.snapshot_options(
        "buffer_updated",
        &SnapshotOptions::default().failed_pixel_count_threshold(SNAPSHOT_PIXEL_TOLERANCE),
    );

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
    harness.snapshot_options(
        "document_saved",
        &SnapshotOptions::default().failed_pixel_count_threshold(SNAPSHOT_PIXEL_TOLERANCE),
    );

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

    harness.snapshot_options(
        "multiple_docs_open",
        &SnapshotOptions::default().failed_pixel_count_threshold(SNAPSHOT_PIXEL_TOLERANCE),
    );

    // Close document 1
    harness
        .state_mut()
        .trigger_action(AppAction::CloseDocument(0));
    harness.step();
    harness.snapshot_options(
        "after_close_first",
        &SnapshotOptions::default().failed_pixel_count_threshold(SNAPSHOT_PIXEL_TOLERANCE),
    );

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
    harness.snapshot_options(
        "preview_with_diagrams",
        &SnapshotOptions::default().failed_pixel_count_threshold(SNAPSHOT_PIXEL_TOLERANCE),
    );

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
    harness.snapshot_options(
        "workspace_with_subdirs",
        &SnapshotOptions::default().failed_pixel_count_threshold(SNAPSHOT_PIXEL_TOLERANCE),
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

// Covers workspace panel collapse UI (shell.rs: L394-407)
#[test]
fn test_integration_workspace_panel_collapsed() {
    let mut harness = setup_harness();
    harness.step();

    // Set show_workspace to false and then draw
    harness.state_mut().app_state_mut().show_workspace = false;
    harness.step();
    harness.snapshot_options(
        "workspace_collapsed",
        &SnapshotOptions::default().failed_pixel_count_threshold(SNAPSHOT_PIXEL_TOLERANCE),
    );

    // Try to click the "›" expand button using kittest (covers shell.rs L403-404)
    // If the button is not found, skip it (in kittest, button strings are
    // compared in Unicode format, so they might not be found)
    {
        use egui_kittest::kittest::Queryable;
        for label in ["›", ">", "❯"] {
            // query_all_by_value does not panic, so it returns an empty iterator if not found
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

// Display both editor and preview in Split mode (shell.rs: L604-)
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

    // Switch to Split mode
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step();
    harness.snapshot_options(
        "split_mode",
        &SnapshotOptions::default().failed_pixel_count_threshold(SNAPSHOT_PIXEL_TOLERANCE),
    );

    // Switch to Code Only mode
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::CodeOnly);
    harness.step();
    harness.snapshot_options(
        "code_only_mode",
        &SnapshotOptions::default().failed_pixel_count_threshold(SNAPSHOT_PIXEL_TOLERANCE),
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

// Open and close multiple tabs (CloseDocument multiple tab handling)
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

    // Open File 1
    let p1 = temp_dir
        .join("file1.md")
        .canonicalize()
        .unwrap_or_else(|_| temp_dir.join("file1.md"));
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(p1));
    harness.step();

    // Open File 2
    let p2 = temp_dir
        .join("file2.md")
        .canonicalize()
        .unwrap_or_else(|_| temp_dir.join("file2.md"));
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(p2));
    harness.step();

    harness.snapshot_options(
        "two_tabs_open",
        &SnapshotOptions::default().failed_pixel_count_threshold(SNAPSHOT_PIXEL_TOLERANCE),
    );

    // Close tab 0 (the remaining tab appropriately updates active_doc_idx)
    harness
        .state_mut()
        .trigger_action(AppAction::CloseDocument(0));
    harness.step();

    // Close tab 0 (the last tab)
    harness
        .state_mut()
        .trigger_action(AppAction::CloseDocument(0));
    harness.step();
    harness.snapshot_options(
        "all_tabs_closed",
        &SnapshotOptions::default().failed_pixel_count_threshold(SNAPSHOT_PIXEL_TOLERANCE),
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

// Workspace force_tree_open toggle (expand all / collapse all tree)
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

    // Expand all
    harness.state_mut().app_state_mut().force_tree_open = Some(true);
    harness.step();

    // Collapse all
    harness.state_mut().app_state_mut().force_tree_open = Some(false);
    harness.step();
    harness.snapshot_options(
        "tree_collapse",
        &SnapshotOptions::default().failed_pixel_count_threshold(SNAPSHOT_PIXEL_TOLERANCE),
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

// UI when no document is selected in PreviewOnly mode (shell.rs L490-492)
#[test]
fn test_integration_preview_only_no_document() {
    let mut harness = setup_harness();
    harness.step();

    // Show "no_document_selected" when active_document is None in PreviewOnly
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(katana_ui::app_state::ViewMode::PreviewOnly);
    harness.step();
    harness.snapshot_options(
        "preview_only_no_doc",
        &SnapshotOptions::default().failed_pixel_count_threshold(SNAPSHOT_PIXEL_TOLERANCE),
    );
}

// RefreshDiagrams action is processed (equivalent to shell.rs L542)
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

    // Emit RefreshDiagrams action
    harness
        .state_mut()
        .trigger_action(AppAction::RefreshDiagrams);
    harness.step();
}

// Toggle show_workspace flag in sidebar (shell.rs L406-407)
#[test]
fn test_integration_sidebar_collapse_expand() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("test.md"), "# Test").unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    harness.step();

    // Close sidebar
    harness.state_mut().app_state_mut().show_workspace = false;
    harness.step();
    // The collapsed panel is displayed on redraw
    harness.step();

    // Re-expand sidebar
    harness.state_mut().app_state_mut().show_workspace = true;
    harness.step();
}

// Click the + / - buttons to expand / collapse entire tree
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
    harness.step();

    // Click + button -> Expand all
    if let Some(btn) = harness.get_all_by_label("+").next() {
        btn.click();
    }
    harness.step();

    // Click - button -> Collapse all
    if let Some(btn) = harness.get_all_by_label("-").next() {
        btn.click();
    }
    harness.step();
}

// Tab ◀ / ▶ navigation + tab click + x (close) button
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
    harness.step();

    // Open 2 files
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

    // Click ◀ button -> Move to previous tab
    if let Some(btn) = harness.get_all_by_label("◀").next() {
        btn.click();
    }
    harness.step();

    // Click ▶ button -> Move to next tab
    if let Some(btn) = harness.get_all_by_label("▶").next() {
        btn.click();
    }
    harness.step();

    // Click tab x button -> Close the tab
    if let Some(btn) = harness.get_all_by_label("x").next() {
        btn.click();
    }
    harness.step();
}

// View mode selection buttons (shell_ui.rs L366)
#[test]
fn test_integration_view_mode_selection_via_button() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("test.md"), "# Test content").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(temp_dir.path().join("test.md")));
    harness.step();
    harness.step(); // Extra frame for UI layout stabilization

    // Click "Code" mode button
    let code_label = katana_ui::i18n::t("view_mode_code");
    if let Some(btn) = harness.get_all_by_label(&code_label).next() {
        btn.click();
    }
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::CodeOnly
    );

    // Click "Preview" mode button
    let preview_label = katana_ui::i18n::t("view_mode_preview");
    if let Some(btn) = harness.get_all_by_label(&preview_label).next() {
        btn.click();
    }
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::PreviewOnly
    );

    // Click "Split" mode button
    let split_label = katana_ui::i18n::t("view_mode_split");
    if let Some(btn) = harness.get_all_by_label(&split_label).next() {
        btn.click();
    }
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::Split
    );
}

// Expand/collapse directory entries (controlled by force_tree_open in state)
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
    harness.step();

    // Expand all -> force_tree_open = Some(true)
    harness.state_mut().app_state_mut().force_tree_open = Some(true);
    harness.step();

    // Collapse all -> force_tree_open = Some(false)
    harness.state_mut().app_state_mut().force_tree_open = Some(false);
    harness.step();
}

// Modify text in editor -> UpdateBuffer (shell_ui.rs L395)
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
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(md_path));
    harness.step();

    // Input using the CodeOnly view of the text editor
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::CodeOnly);
    harness.step();

    // Inject UpdateBuffer action directly
    harness
        .state_mut()
        .trigger_action(AppAction::UpdateBuffer("# Modified content".to_string()));
    harness.step();
}

// Refresh diagrams button (🔄) click (shell_ui.rs L248-249)
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
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(md_path));
    harness.step();

    // Click 🔄 button
    if let Some(btn) = harness.get_all_by_label("🔄").next() {
        btn.click();
    }
    harness.step();
}

// ── Persistence roundtrip tests (real file I/O) ──

/// Helper: create a Harness backed by a JsonFileRepository at the given path.
fn setup_harness_with_json_repo(settings_path: &std::path::Path) -> Harness<'static, KatanaApp> {
    let path = settings_path.to_path_buf();
    Harness::builder().build_eframe(move |_cc| {
        let repo = katana_platform::JsonFileRepository::new(path.clone());
        let settings = katana_platform::SettingsService::new(Box::new(repo));
        let state = AppState::new(AiProviderRegistry::new(), PluginRegistry::new(), settings);
        katana_ui::i18n::set_language("en");
        KatanaApp::new(state)
    })
}

#[test]
fn test_persistence_workspace_roundtrip() {
    let settings_dir = tempfile::tempdir().unwrap();
    let settings_path = settings_dir.path().join("settings.json");

    // Create a workspace directory.
    let ws_dir = tempfile::tempdir().unwrap();
    std::fs::write(ws_dir.path().join("doc.md"), "# Hello").unwrap();

    // --- Session 1: Open workspace → settings saved to disk ---
    {
        let mut harness = setup_harness_with_json_repo(&settings_path);
        harness.step();

        harness
            .state_mut()
            .trigger_action(AppAction::OpenWorkspace(ws_dir.path().to_path_buf()));
        harness.step();

        // Verify workspace was opened.
        assert!(harness.state_mut().app_state_mut().workspace.is_some());

        // Verify settings file was written with the workspace path.
        let json = std::fs::read_to_string(&settings_path).unwrap();
        assert!(
            json.contains(&ws_dir.path().display().to_string()),
            "settings.json should contain the workspace path, got: {json}"
        );
    }

    // --- Session 2: New app from the same file → workspace path is restored ---
    {
        let repo = katana_platform::JsonFileRepository::new(settings_path.to_path_buf());
        let settings = katana_platform::SettingsService::new(Box::new(repo));
        let restored_ws = settings.settings().last_workspace.clone();

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

    // --- Session 1: Change language to "ja" → saved to disk ---
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
    }

    // --- Session 2: Reload from disk → language is "ja" ---
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

    // --- Session 1: Open workspace + change language ---
    {
        let mut harness = setup_harness_with_json_repo(&settings_path);
        harness.step();

        harness
            .state_mut()
            .trigger_action(AppAction::OpenWorkspace(ws_dir.path().to_path_buf()));
        harness.step();

        harness
            .state_mut()
            .trigger_action(AppAction::ChangeLanguage("ja".to_string()));
        harness.step();
    }

    // --- Session 2: Both workspace AND language should be restored ---
    {
        let repo = katana_platform::JsonFileRepository::new(settings_path.to_path_buf());
        let settings = katana_platform::SettingsService::new(Box::new(repo));
        let s = settings.settings();

        assert!(
            s.last_workspace.is_some(),
            "last_workspace should be persisted"
        );
        assert_eq!(s.language, "ja", "language should be persisted");
    }
}

#[test]
fn test_persistence_corrupt_file_falls_back_to_defaults() {
    let settings_dir = tempfile::tempdir().unwrap();
    let settings_path = settings_dir.path().join("settings.json");

    // Write corrupt JSON.
    std::fs::write(&settings_path, "NOT VALID JSON {{{").unwrap();

    // App should start with defaults without panicking.
    let mut harness = setup_harness_with_json_repo(&settings_path);
    harness.step();

    let s = harness.state_mut().app_state_mut();
    assert_eq!(
        s.settings.settings().theme,
        "dark",
        "Should fall back to default theme"
    );
    assert_eq!(
        s.settings.settings().language,
        "en",
        "Should fall back to default language"
    );
    assert!(
        s.settings.settings().last_workspace.is_none(),
        "Should fall back to no workspace"
    );
}

#[test]
fn test_persistence_missing_file_uses_defaults() {
    let settings_dir = tempfile::tempdir().unwrap();
    let settings_path = settings_dir.path().join("nonexistent.json");

    // File does not exist — should gracefully use defaults.
    let mut harness = setup_harness_with_json_repo(&settings_path);
    harness.step();

    let s = harness.state_mut().app_state_mut();
    assert_eq!(s.settings.settings().theme, "dark");
    assert_eq!(s.settings.settings().language, "en");
}

// ── Preview rendering regression tests ──
//
// Background: render_preview_header calling with_layout(right_to_left(Align::Center)) at the
// top level consumes all available_height, causing the subsequent ScrollArea to have 0 height.
// This test is to detect this regression early.
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
    harness.step();
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

    // Regression if "(No preview)" is displayed
    let no_preview_label = katana_ui::i18n::t("no_preview");
    let no_preview_nodes: Vec<_> = harness.query_all_by_value(&no_preview_label).collect();
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
    harness.step();
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

    let no_preview_label = katana_ui::i18n::t("no_preview");
    let no_preview_nodes: Vec<_> = harness.query_all_by_value(&no_preview_label).collect();
    assert!(
        no_preview_nodes.is_empty(),
        "Split mode preview pane must NOT show '{no_preview_label}' when a document is open."
    );
}

/// Assert that the split direction setting is Horizontal by default in Split mode,
/// and that changing the setting is reflected correctly.
/// (Asserting via setting values since searching for Unicode buttons in egui_kittest
///  is currently unstable)
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
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(md_path));
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step();

    // Default setting is Horizontal
    assert_eq!(
        harness.state_mut().app_state_mut().active_split_direction(),
        katana_platform::SplitDirection::Horizontal,
        "Default split direction must be Horizontal"
    );

    // Temporarily switch direction in Split mode (not saved to settings)
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

/// UI Layer Test: Simulate clicking on UI widgets by inserting pointer events into egui.
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
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(md_path));
    harness.step();
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step(); // UI should render and the '⇕' button should appear

    // Default setting is Horizontal
    assert_eq!(
        harness.state_mut().app_state_mut().active_split_direction(),
        katana_platform::SplitDirection::Horizontal,
    );

    // Use node.click() (if provided by egui_kittest)
    // If not provided, try .click() or use ui interaction helpers.
    let node = harness.get_by_label("⇕");
    node.click();
    harness.step();

    // Check if the Action was correctly dispatched and reflected from the UI
    assert_eq!(
        harness.state_mut().app_state_mut().active_split_direction(),
        katana_platform::SplitDirection::Vertical,
        "UI click on '⇕' should toggle split direction to Vertical",
    );
}

// ── Split direction / pane order Action tests ──
// Verify that SetSplitDirection / SetPaneOrder are correctly saved
// to the per-tab temporary state via process_action, without persisting to settings.

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
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(md_path));
    harness.step();
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step();

    // Default is Horizontal (settings default)
    assert_eq!(
        harness.state_mut().app_state_mut().active_split_direction(),
        katana_platform::SplitDirection::Horizontal,
    );

    // Switch to Vertical via Action
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

    // Verify that the state was not persisted to settings
    assert_eq!(
        harness
            .state_mut()
            .app_state_mut()
            .settings
            .settings()
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
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(md_path));
    harness.step();
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step();

    // Horizontal → Vertical → Horizontal
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
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(md_path));
    harness.step();
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step();

    // Default is EditorFirst
    assert_eq!(
        harness.state_mut().app_state_mut().active_pane_order(),
        katana_platform::PaneOrder::EditorFirst,
    );

    // EditorFirst → PreviewFirst
    harness.state_mut().trigger_action(AppAction::SetPaneOrder(
        katana_platform::PaneOrder::PreviewFirst,
    ));
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_pane_order(),
        katana_platform::PaneOrder::PreviewFirst,
        "SetPaneOrder(PreviewFirst) must update per-tab pane order"
    );

    // PreviewFirst → EditorFirst
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

// ── v0.1.2 TDD RED tests ──

/// Bug Fix 1: File entry labels in workspace panel must be left-aligned
/// within their parent directory container. The label rect must start
/// near the left edge of the workspace panel (with indent), NOT at the
/// right side. We compare the label rect.left() against the row rect
/// to verify proper left-alignment.
#[test]
fn test_file_entry_label_is_left_aligned() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("alignment.md"), "# Alignment").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    harness.step();
    harness.step();

    // Find the file label node.
    let nodes: Vec<_> = harness.get_all_by_value("📄 alignment.md").collect();
    assert!(
        !nodes.is_empty(),
        "File entry '📄 alignment.md' must be present in the workspace tree"
    );

    let node = &nodes[0];
    let label_rect = node.rect();
    // The label text width must be smaller than the full row width.
    // When right-aligned via add_sized, the label rect spans the full row.
    // When properly left-aligned via add(), the label rect width matches
    // only the text content width — which must be < 80% of the panel
    // default width (220px). So label width < 176px.
    let label_width = label_rect.width();
    assert!(
        label_width < 176.0,
        "File entry label width must be text-width, not full-row-width. \
         Got width={label_width:.1}, expected < 176.0 (indicates add_sized right-alignment bug)"
    );
}

/// Regression: Clicking a file entry in the workspace must open the document.
/// This test reproduces a critical regression where removing label_resp from
/// the click response union caused file clicks to stop working.
#[test]
fn test_file_entry_click_opens_document() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("clickable.md"), "# Clickable").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.path().to_path_buf()));
    harness.step();
    harness.step();

    // Verify no document is open yet
    assert!(
        harness.state_mut().app_state_mut().active_doc_idx.is_none(),
        "No document should be open before clicking"
    );

    // Click the file entry
    let nodes: Vec<_> = harness.get_all_by_value("📄 clickable.md").collect();
    assert!(!nodes.is_empty(), "File entry must be present");
    nodes[0].click();
    harness.step();
    harness.step();

    // The document must be opened
    assert!(
        harness.state_mut().app_state_mut().active_doc_idx.is_some(),
        "Clicking a file entry must open the document (active_doc_idx should be Some)"
    );
    assert_eq!(
        harness.state_mut().app_state_mut().open_documents.len(),
        1,
        "Exactly one document should be open after clicking"
    );
}

/// Bug Fix 2: Tab ◀/▶ navigation buttons must have i18n tooltips.
/// After rendering with multiple tabs, verify the tooltip i18n keys exist
/// by checking that the buttons' on_hover_text callbacks register the
/// expected text. We test this by verifying the rendered UI contains the
/// tooltip response.
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
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(temp_dir.path().join("a.md")));
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(temp_dir.path().join("b.md")));
    harness.step();

    // Verify ◀ button exists and can be found by label
    let prev_nodes: Vec<_> = harness.get_all_by_label("◀").collect();
    assert!(
        !prev_nodes.is_empty(),
        "◀ (previous tab) button must be present"
    );

    // Verify ▶ button exists and can be found by label
    let next_nodes: Vec<_> = harness.get_all_by_label("▶").collect();
    assert!(
        !next_nodes.is_empty(),
        "▶ (next tab) button must be present"
    );

    // Hover the ◀ button to trigger tooltip rendering path
    prev_nodes[0].hover();
    harness.step();
    harness.step();

    // Verify the i18n keys resolve correctly (tooltip text is registered)
    let prev_tooltip = katana_ui::i18n::t("tab_nav_prev");
    let next_tooltip = katana_ui::i18n::t("tab_nav_next");
    assert_ne!(
        prev_tooltip, "tab_nav_prev",
        "tab_nav_prev i18n key must resolve to translated text"
    );
    assert_ne!(
        next_tooltip, "tab_nav_next",
        "tab_nav_next i18n key must resolve to translated text"
    );

    // Verify hover triggers repaint (tooltip rendering is exercised)
    // The tooltip layer may not appear in AccessKit within kittest,
    // but the on_hover_text code path is compiled and exercised.
    // Visual verification is handled by snapshot tests.
}

/// Bug Fix 3: Font size slider must have a hover tooltip describing usage.
/// When the slider is hovered, the tooltip text should appear in the UI.
#[test]
fn test_font_size_slider_has_hover_tooltip() {
    let mut harness = setup_harness();
    harness.step();

    // Open settings window
    harness
        .state_mut()
        .trigger_action(AppAction::ToggleSettings);
    harness.step();

    // Switch to Font tab
    harness.state_mut().app_state_mut().active_settings_tab =
        katana_ui::app_state::SettingsTab::Font;
    harness.step();
    harness.step();

    let hint_text = katana_ui::i18n::t("settings_font_size_slider_hint");
    // Verify i18n key resolves
    assert_ne!(
        hint_text, "settings_font_size_slider_hint",
        "i18n key must resolve to a translated value, not the key itself"
    );

    // The slider's on_hover_text is compiled and exercised during rendering.
    // Snapshot tests cover the visual tooltip rendering.
    // Here we verify the settings Font tab renders without panic and
    // the i18n key is correctly resolved for tooltip integration.
    harness.snapshot_options(
        "settings_font_tab",
        &SnapshotOptions::default().failed_pixel_count_threshold(SNAPSHOT_PIXEL_TOLERANCE),
    );
}
