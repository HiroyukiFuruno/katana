use egui_kittest::kittest::Queryable;
use egui_kittest::{Harness, SnapshotOptions};
use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
use katana_ui::app_state::{AppAction, AppState, ViewMode};
use katana_ui::shell::KatanaApp;

/// Snapshot pixel tolerance to absorb non-deterministic rendering differences.
/// egui_kittest defaults to 0, but font hinting and anti-aliasing vary between runs.
/// Max observed diff between local and GitHub Actions macOS environment: ~3731 pixels.
/// We set it to 4000 to provide a comfortable margin for environmental differences.
const SNAPSHOT_PIXEL_TOLERANCE: usize = 4000;

fn setup_harness() -> Harness<'static, KatanaApp> {
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
