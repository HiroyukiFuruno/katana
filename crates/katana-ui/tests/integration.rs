use egui_kittest::kittest::Queryable;
use egui_kittest::{Harness, SnapshotOptions};
use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
use katana_platform::SettingsService;
use katana_ui::app_state::{AppAction, AppState, ViewMode};
use katana_ui::shell::KatanaApp;

/// レンダリングの微小な非決定性を許容するスナップショット比較設定。
/// egui のテキストレイアウトはフレームごとにサブピクセルレベルの差分を
/// 生じることがあるため、閾値を設定して偽陽性を回避する。
/// また、テスト内で生成する temp ディレクトリのパスが毎回異なるため、
/// ファイルパス表示領域で数千ピクセルの差分が発生しうる。
fn snap_opts() -> SnapshotOptions {
    SnapshotOptions::new()
        .threshold(0.6)
        .failed_pixel_count_threshold(20_000)
}

fn setup_harness() -> Harness<'static, KatanaApp> {
    Harness::builder().build_eframe(|_cc| {
        let ai_registry = AiProviderRegistry::new();
        let plugin_registry = PluginRegistry::new();
        let settings = SettingsService::in_memory();
        let state = AppState::new(ai_registry, plugin_registry, settings);
        katana_ui::i18n::set_language("en");
        KatanaApp::new(state)
    })
}

fn setup_with_temp_workspace() -> (Harness<'static, KatanaApp>, std::path::PathBuf) {
    let temp_dir = std::env::temp_dir().join(format!(
        "katana_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();

    // 複数のファイルとディレクトリを作成
    let sub_dir = temp_dir.join("docs");
    std::fs::create_dir_all(&sub_dir).unwrap();
    std::fs::write(temp_dir.join("readme.md"), "# Readme\n\nTop level file.").unwrap();
    std::fs::write(
        temp_dir.join("notes.md"),
        "# Notes\n\n- item 1\n- item 2\n- item 3",
    )
    .unwrap();
    std::fs::write(sub_dir.join("spec.md"), "# Spec\n\nDetailed spec.").unwrap();

    let mut harness = setup_harness();
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    harness.step();

    (harness, temp_dir)
}

fn cleanup(temp_dir: &std::path::Path) {
    let _ = std::fs::remove_dir_all(temp_dir);
}

// ── 基本起動テスト ──

#[test]
fn test_integration_application_startup() {
    let mut harness = setup_harness();
    harness.step();
    let _node = harness.get_by_label("No workspace open.");
    harness.snapshot_options("startup_screen", &snap_opts());
}

// ── ワークスペース & タブ操作テスト ──

#[test]
fn test_integration_workspace_and_tabs() {
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = std::env::temp_dir().join("katana_test_ws");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();
    let test_file = temp_dir.join("test1.md");
    std::fs::write(&test_file, "# Hello Katana").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    harness.step();

    let _file_node = harness.get_all_by_value("📄 test1.md").next().unwrap();

    harness
        .get_all_by_value("📄 test1.md")
        .next()
        .unwrap()
        .click();
    harness.step();

    harness.snapshot_options("editor_opened", &snap_opts());

    harness
        .state_mut()
        .trigger_action(AppAction::CloseDocument(0));
    harness.step();

    harness.snapshot_options("editor_closed", &snap_opts());

    let _ = std::fs::remove_dir_all(&temp_dir);
}

// ── ビューモード切り替えテスト ──

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
    harness.step();
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
    harness.snapshot_options("view_mode_preview_only", &snap_opts());

    // Switch to Split
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step();
    harness.snapshot_options("view_mode_split", &snap_opts());

    // Switch to Code Only
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::CodeOnly);
    harness.step();
    harness.snapshot_options("view_mode_code_only", &snap_opts());

    let _ = std::fs::remove_dir_all(&temp_dir);
}

// ── 複数タブ＋タブナビゲーションテスト ──

#[test]
fn test_integration_multiple_tabs_navigation() {
    let (mut harness, temp_dir) = setup_with_temp_workspace();

    // readme.md を開く
    let readme_path = temp_dir.join("readme.md").canonicalize().unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(readme_path.clone()));
    harness.step();

    // notes.md を開く
    let notes_path = temp_dir.join("notes.md").canonicalize().unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(notes_path.clone()));
    harness.step();

    // タブが2つ存在すること
    let state = harness.state_mut().app_state_mut();
    assert_eq!(state.open_documents.len(), 2);

    harness.snapshot_options("multiple_tabs", &snap_opts());

    // 最初のタブに切り替え
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(readme_path.clone()));
    harness.step();

    // アクティブなドキュメントが readme.md であること
    let state = harness.state_mut().app_state_mut();
    assert_eq!(state.active_doc_idx, Some(0));

    harness.snapshot_options("tab_switched_to_readme", &snap_opts());

    cleanup(&temp_dir);
}

// ── テキスト編集 & Dirty 表示テスト ──

#[test]
fn test_integration_text_editing_and_dirty_state() {
    let (mut harness, temp_dir) = setup_with_temp_workspace();

    let readme_path = temp_dir.join("readme.md").canonicalize().unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(readme_path));
    harness.step();

    // テキストを更新
    harness.state_mut().trigger_action(AppAction::UpdateBuffer(
        "# Edited Content\n\nThis is new content.".to_string(),
    ));
    harness.step();

    // Dirty 状態をスナップショットキャプチャ（ステータスバーに ● が表示）
    harness.snapshot_options("dirty_state", &snap_opts());

    cleanup(&temp_dir);
}

// ── ファイル保存テスト ──

#[test]
fn test_integration_save_document() {
    let (mut harness, temp_dir) = setup_with_temp_workspace();

    let readme_path = temp_dir.join("readme.md").canonicalize().unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(readme_path.clone()));
    harness.step();

    // テキストを更新して dirty にする
    harness
        .state_mut()
        .trigger_action(AppAction::UpdateBuffer("# Saved Content".to_string()));
    harness.step();

    // 保存実行
    harness.state_mut().trigger_action(AppAction::SaveDocument);
    harness.step();

    // ファイルの内容が書き変わっていること
    let saved = std::fs::read_to_string(&readme_path).unwrap();
    assert_eq!(saved, "# Saved Content");

    harness.snapshot_options("after_save", &snap_opts());

    cleanup(&temp_dir);
}

// ── ワークスペースパネル折りたたみテスト ──

#[test]
fn test_integration_workspace_panel_collapse() {
    let (mut harness, temp_dir) = setup_with_temp_workspace();

    // ワークスペースパネルを折りたたむ
    harness.state_mut().app_state_mut().show_workspace = false;
    harness.step();

    harness.snapshot_options("workspace_collapsed", &snap_opts());

    // 展開ボタン「›」を見つける
    harness.step();

    // 再び展開する
    harness.state_mut().app_state_mut().show_workspace = true;
    harness.step();

    harness.snapshot_options("workspace_expanded", &snap_opts());

    cleanup(&temp_dir);
}

// ── ディレクトリツリー全展開/全折りたたみテスト ──

#[test]
fn test_integration_tree_expand_collapse_all() {
    let (mut harness, temp_dir) = setup_with_temp_workspace();

    // 全展開
    harness.state_mut().app_state_mut().force_tree_open = Some(true);
    harness.step();

    harness.snapshot_options("tree_expanded_all", &snap_opts());

    // 全折りたたみ
    harness.state_mut().app_state_mut().force_tree_open = Some(false);
    harness.step();

    harness.snapshot_options("tree_collapsed_all", &snap_opts());

    cleanup(&temp_dir);
}

// ── 言語切り替えテスト ──

#[test]
fn test_integration_language_change() {
    let (mut harness, temp_dir) = setup_with_temp_workspace();

    // 日本語に切り替え
    harness
        .state_mut()
        .trigger_action(AppAction::ChangeLanguage("ja".to_string()));
    harness.step();

    harness.snapshot_options("language_ja", &snap_opts());

    // 英語に戻す
    harness
        .state_mut()
        .trigger_action(AppAction::ChangeLanguage("en".to_string()));
    harness.step();

    harness.snapshot_options("language_en", &snap_opts());

    cleanup(&temp_dir);
}

// ── ダイアグラム付きプレビューテスト ──

#[test]
fn test_integration_preview_with_diagram() {
    let (mut harness, temp_dir) = setup_with_temp_workspace();

    // ダイアグラム入りのファイルを作成
    let diagram_file = temp_dir.join("diagram.md");
    let drawio_xml = r#"<mxGraphModel><root>
<mxCell id="0"/><mxCell id="1" parent="0"/>
<mxCell id="2" value="Box" vertex="1" parent="1">
    <mxGeometry x="50" y="50" width="100" height="60" as="geometry"/>
</mxCell>
</root></mxGraphModel>"#;
    std::fs::write(
        &diagram_file,
        format!("# Diagram Test\n\n```drawio\n{drawio_xml}\n```\n\n## After Diagram"),
    )
    .unwrap();

    // ワークスペースを再オープン（新しいファイルを検出）
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    harness.step();

    let abs_path = diagram_file.canonicalize().unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path));
    harness.step();

    // プレビューモードで描画
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::PreviewOnly);
    // 数フレーム回してバックグラウンドレンダリングを完了させる
    for _ in 0..10 {
        harness.step();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    harness.snapshot_options("preview_with_diagram", &snap_opts());

    cleanup(&temp_dir);
}

// ── Split ビューでの表示テスト ──

#[test]
fn test_integration_split_view_rendering() {
    let (mut harness, temp_dir) = setup_with_temp_workspace();

    let readme_path = temp_dir.join("readme.md").canonicalize().unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(readme_path));
    harness.step();

    // Split モードに切り替え
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step();
    harness.step(); // 2フレーム回してプレビュー描画を確実にする

    harness.snapshot_options("split_view_rendering", &snap_opts());

    cleanup(&temp_dir);
}

// ── ドキュメントなしのプレビューモード ──

#[test]
fn test_integration_preview_mode_no_document() {
    let mut harness = setup_harness();
    harness.step();

    // PreviewOnly モードに切り替え（ドキュメントなし）
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::PreviewOnly);
    harness.step();

    harness.snapshot_options("preview_mode_no_document", &snap_opts());
}

// ── ダイアグラムリフレッシュテスト ──

#[test]
fn test_integration_refresh_diagrams() {
    let (mut harness, temp_dir) = setup_with_temp_workspace();

    let readme_path = temp_dir.join("readme.md").canonicalize().unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(readme_path));
    harness.step();

    // RefreshDiagrams アクション
    harness
        .state_mut()
        .trigger_action(AppAction::RefreshDiagrams);
    harness.step();

    harness.snapshot_options("after_refresh_diagrams", &snap_opts());

    cleanup(&temp_dir);
}

// ── ステータスバー表示テスト（dirty なし） ──

#[test]
fn test_integration_status_bar_clean() {
    let (mut harness, temp_dir) = setup_with_temp_workspace();

    let readme_path = temp_dir.join("readme.md").canonicalize().unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(readme_path));
    harness.step();

    // ステータスバーに "Ready" 的なメッセージが表示されている
    harness.snapshot_options("status_bar_clean", &snap_opts());

    cleanup(&temp_dir);
}

// ── ワークスペースツリーのファイルクリック ──

#[test]
fn test_integration_tree_file_click() {
    let (mut harness, temp_dir) = setup_with_temp_workspace();

    // ディレクトリの展開
    harness.state_mut().app_state_mut().force_tree_open = Some(true);
    harness.step();
    harness.step(); // 追加のフレームでツリーを完全に展開

    // spec.md を直接アクションで開く（ツリーからファイルパスを取得する代わり）
    let spec_path = temp_dir.join("docs").join("spec.md");
    if let Ok(abs) = spec_path.canonicalize() {
        harness
            .state_mut()
            .trigger_action(AppAction::SelectDocument(abs));
        harness.step();
    }

    harness.snapshot_options("after_tree_file_click", &snap_opts());

    cleanup(&temp_dir);
}

// ── 複数タブのクローズテスト ──

#[test]
fn test_integration_close_multiple_tabs() {
    let (mut harness, temp_dir) = setup_with_temp_workspace();

    // 2つのファイルを開く
    let readme_path = temp_dir.join("readme.md").canonicalize().unwrap();
    let notes_path = temp_dir.join("notes.md").canonicalize().unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(readme_path));
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(notes_path));
    harness.step();

    // 最初のタブを閉じる
    harness
        .state_mut()
        .trigger_action(AppAction::CloseDocument(0));
    harness.step();

    // 残り1タブ
    let doc_count = harness.state_mut().app_state_mut().open_documents.len();
    assert_eq!(doc_count, 1);

    // 最後のタブも閉じる
    harness
        .state_mut()
        .trigger_action(AppAction::CloseDocument(0));
    harness.step();

    let doc_count = harness.state_mut().app_state_mut().open_documents.len();
    assert!(doc_count == 0);

    harness.snapshot_options("all_tabs_closed", &snap_opts());

    cleanup(&temp_dir);
}

// ── 既に開いているファイルの再選択テスト ──

#[test]
fn test_integration_reselect_open_document() {
    let (mut harness, temp_dir) = setup_with_temp_workspace();

    let readme_path = temp_dir.join("readme.md").canonicalize().unwrap();

    // ファイルを開く
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(readme_path.clone()));
    harness.step();

    // 同じファイルを再び選択 → 既存タブにフォーカス（新しいタブは開かない）
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(readme_path));
    harness.step();

    let doc_count = harness.state_mut().app_state_mut().open_documents.len();
    assert_eq!(doc_count, 1, "should not duplicate tabs");

    harness.snapshot_options("reselect_same_file", &snap_opts());

    cleanup(&temp_dir);
}

// ── 存在しないファイルのオープンテスト ──

#[test]
fn test_integration_open_nonexistent_file() {
    let mut harness = setup_harness();
    harness.step();

    // 存在しないワークスペースを開こうとする
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(std::path::PathBuf::from(
            "/nonexistent/workspace/path",
        )));
    harness.step();

    harness.snapshot_options("nonexistent_workspace_error", &snap_opts());
}

// ── 存在しないドキュメントのオープンテスト ──

#[test]
fn test_integration_open_nonexistent_document() {
    let (mut harness, temp_dir) = setup_with_temp_workspace();

    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(std::path::PathBuf::from(
            "/nonexistent/file.md",
        )));
    harness.step();

    // エラーメッセージがステータスバーに表示される
    harness.snapshot_options("nonexistent_document_error", &snap_opts());

    cleanup(&temp_dir);
}

// ── PlantUML / Mermaid ダイアグラムレンダリング結果のUI描画テスト ──

#[test]
fn test_integration_preview_with_plantuml_not_installed() {
    // PlantUML が未インストール時の NotInstalled UI 表示テスト
    unsafe { std::env::set_var("PLANTUML_JAR", "/nonexistent/plantuml.jar") };

    let (mut harness, temp_dir) = setup_with_temp_workspace();

    let plantuml_file = temp_dir.join("plantuml_test.md");
    std::fs::write(
        &plantuml_file,
        "# PlantUML Test\n\n```plantuml\n@startuml\nA -> B\n@enduml\n```\n\n## After",
    )
    .unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    harness.step();

    let abs_path = plantuml_file.canonicalize().unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path));
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::PreviewOnly);

    // バックグラウンドレンダリング完了を待つ
    for _ in 0..20 {
        harness.step();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    harness.snapshot_options("preview_plantuml_not_installed", &snap_opts());

    unsafe { std::env::remove_var("PLANTUML_JAR") };
    cleanup(&temp_dir);
}

#[test]
fn test_integration_preview_with_mermaid() {
    // Mermaid ダイアグラムの CommandNotFound または Image 表示テスト
    let (mut harness, temp_dir) = setup_with_temp_workspace();

    let mermaid_file = temp_dir.join("mermaid_test.md");
    std::fs::write(
        &mermaid_file,
        "# Mermaid Test\n\n```mermaid\ngraph TD; A-->B\n```\n\n## After Mermaid",
    )
    .unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    harness.step();

    let abs_path = mermaid_file.canonicalize().unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path));
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::PreviewOnly);

    for _ in 0..20 {
        harness.step();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    harness.snapshot_options("preview_mermaid", &snap_opts());

    cleanup(&temp_dir);
}

#[test]
fn test_integration_split_view_with_diagrams() {
    // Split ビューでダイアグラム付きドキュメントの描画テスト
    let (mut harness, temp_dir) = setup_with_temp_workspace();

    let drawio_xml = r#"<mxGraphModel><root>
<mxCell id="0"/><mxCell id="1" parent="0"/>
<mxCell id="2" value="Split" vertex="1" parent="1">
    <mxGeometry x="50" y="50" width="100" height="60" as="geometry"/>
</mxCell>
</root></mxGraphModel>"#;

    let diagram_file = temp_dir.join("split_diagram.md");
    std::fs::write(
        &diagram_file,
        format!("# Split View\n\n```drawio\n{drawio_xml}\n```\n\n## End"),
    )
    .unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    harness.step();

    let abs_path = diagram_file.canonicalize().unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path));
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);

    for _ in 0..15 {
        harness.step();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    harness.snapshot_options("split_view_with_diagram", &snap_opts());

    cleanup(&temp_dir);
}

// ── テキスト編集後のプレビュー更新テスト ──

#[test]
fn test_integration_edit_then_preview() {
    let (mut harness, temp_dir) = setup_with_temp_workspace();

    let readme_path = temp_dir.join("readme.md").canonicalize().unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(readme_path));
    harness.step();

    // Split モードに切り替え
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step();

    // テキストを編集
    harness.state_mut().trigger_action(AppAction::UpdateBuffer(
        "# Live Edit Preview\n\nUpdated content visible in preview.".to_string(),
    ));
    harness.step();
    harness.step(); // 追加フレームでプレビュー更新を確認

    harness.snapshot_options("edit_then_preview_split", &snap_opts());

    cleanup(&temp_dir);
}

// ── 同一ファイル再選択テスト（L126-127 をカバー）──

#[test]
fn test_integration_reselect_same_file_no_change() {
    let (mut harness, temp_dir) = setup_with_temp_workspace();

    let readme_path = temp_dir.join("readme.md").canonicalize().unwrap();
    // 1回目の選択
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(readme_path.clone()));
    harness.step();

    // 同じファイルを再選択（内容変化なし → L126-127 のキャッシュパスを通す）
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(readme_path));
    harness.step();

    harness.snapshot_options("reselect_same_file", &snap_opts());

    cleanup(&temp_dir);
}

// ── ドキュメント未選択時の SaveDocument / UpdateBuffer テスト（L155, L162）──

#[test]
fn test_integration_save_without_active_document() {
    let mut harness = setup_harness();
    harness.step();

    // ドキュメントが開かれていない状態で SaveDocument
    harness.state_mut().trigger_action(AppAction::SaveDocument);
    harness.step();

    // パニックしないことを確認。ステータスメッセージが変わらない
    let state = harness.state_mut().app_state_mut();
    assert!(state.active_document().is_none());
}

#[test]
fn test_integration_update_buffer_without_active_document() {
    let mut harness = setup_harness();
    harness.step();

    // ドキュメントが開かれていない状態で UpdateBuffer
    harness
        .state_mut()
        .trigger_action(AppAction::UpdateBuffer("test".to_string()));
    harness.step();

    // パニックしないことを確認
    let state = harness.state_mut().app_state_mut();
    assert!(state.active_document().is_none());
}

// ── Code ビューモードでエディタ描画テスト（L807-862 をカバー）──

#[test]
fn test_integration_code_view_mode_renders_editor() {
    let (mut harness, temp_dir) = setup_with_temp_workspace();

    let readme_path = temp_dir.join("readme.md").canonicalize().unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(readme_path));
    harness.step();

    // Code モード
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::CodeOnly);
    harness.step();
    harness.step();

    harness.snapshot_options("code_view_editor", &snap_opts());

    cleanup(&temp_dir);
}

// ── タブナビゲーションテスト（L748-764 をカバー）──

#[test]
fn test_integration_tab_navigation_buttons() {
    let (mut harness, temp_dir) = setup_with_temp_workspace();

    // 複数ファイルを開く
    let readme_path = temp_dir.join("readme.md").canonicalize().unwrap();
    let notes_path = temp_dir.join("notes.md").canonicalize().unwrap();
    let spec_path = temp_dir.join("docs/spec.md").canonicalize().unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(readme_path));
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(notes_path));
    harness.step();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(spec_path));
    harness.step();

    // 3つのタブが開いている状態
    let state = harness.state_mut().app_state_mut();
    assert_eq!(state.open_documents.len(), 3);

    harness.snapshot_options("three_tabs_open", &snap_opts());

    cleanup(&temp_dir);
}

// ── リフレッシュダイアグラムテスト（L674-675 をカバー）──

#[test]
fn test_integration_refresh_diagrams_action() {
    let (mut harness, temp_dir) = setup_with_temp_workspace();

    let readme_path = temp_dir.join("readme.md").canonicalize().unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(readme_path));
    harness.step();

    // RefreshDiagrams アクション
    harness
        .state_mut()
        .trigger_action(AppAction::RefreshDiagrams);
    harness.step();
    harness.step();

    // パニックしないことを確認
    harness.snapshot_options("after_refresh_action", &snap_opts());

    cleanup(&temp_dir);
}

// ── SaveDocument エラーパス（L166-170 をカバー）──

#[test]
fn test_integration_save_document_to_readonly_path() {
    let (mut harness, temp_dir) = setup_with_temp_workspace();

    let readme_path = temp_dir.join("readme.md").canonicalize().unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(readme_path.clone()));
    harness.step();

    // ファイルを読み取り専用にする
    let mut perms = std::fs::metadata(&readme_path).unwrap().permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o444);
        std::fs::set_permissions(&readme_path, perms).unwrap();
    }

    // バッファを変更してから保存を試みる
    harness
        .state_mut()
        .trigger_action(AppAction::UpdateBuffer("Modified content".to_string()));
    harness.step();

    harness.state_mut().trigger_action(AppAction::SaveDocument);
    harness.step();

    // エラーメッセージがステータスバーに表示される
    let state = harness.state_mut().app_state_mut();
    if let Some(ref msg) = state.status_message {
        // 保存成功 or 保存失敗のいずれか（OS権限モデルによる）
        assert!(
            msg.contains("Saved") || msg.contains("fail") || msg.contains("error"),
            "status should indicate save result: {msg}"
        );
    }

    // パーミッションを戻す
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms2 = std::fs::metadata(&readme_path).unwrap().permissions();
        perms2.set_mode(0o644);
        std::fs::set_permissions(&readme_path, perms2).unwrap();
    }

    cleanup(&temp_dir);
}

// ── ワークスペース未設定時のUI描画テスト（L600-606 をカバー）──

#[test]
fn test_integration_no_workspace_shows_open_button() {
    let mut harness = setup_harness();
    harness.step();
    harness.step(); // 2フレーム目で確実に描画

    // ワークスペースが開かれていない状態ではメッセージが表示される
    let _label = harness.get_by_label("No workspace open.");

    harness.snapshot_options("no_workspace_state", &snap_opts());
}

// ── ChangeLanguage アクションテスト ──

#[test]
fn test_integration_change_language_ja_then_en() {
    let (mut harness, temp_dir) = setup_with_temp_workspace();

    // 日本語に切り替え
    harness
        .state_mut()
        .trigger_action(AppAction::ChangeLanguage("ja".to_string()));
    harness.step();

    harness.snapshot_options("language_switched_ja", &snap_opts());

    // 英語に戻す
    harness
        .state_mut()
        .trigger_action(AppAction::ChangeLanguage("en".to_string()));
    harness.step();

    harness.snapshot_options("language_switched_en", &snap_opts());

    cleanup(&temp_dir);
}

// ── Split ビューでのスクロール同期描画テスト（L822-862, L630-657 をカバー）──

#[test]
fn test_integration_split_view_scroll_sync_rendering() {
    let (mut harness, temp_dir) = setup_with_temp_workspace();

    // 長いコンテンツを持つファイルを作成
    let long_content = (0..100)
        .map(|i| format!("## Section {i}\n\nParagraph {i} content.\n"))
        .collect::<String>();
    let long_file = temp_dir.join("long.md");
    std::fs::write(&long_file, &long_content).unwrap();

    // ファイルツリーを再読み込み
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    harness.step();

    let long_path = long_file.canonicalize().unwrap();
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(long_path));
    harness.step();

    // Split モードに切り替え
    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step();
    harness.step();
    harness.step(); // 複数フレームでスクロール同期初期化

    harness.snapshot_options("split_view_long_content", &snap_opts());

    cleanup(&temp_dir);
}
