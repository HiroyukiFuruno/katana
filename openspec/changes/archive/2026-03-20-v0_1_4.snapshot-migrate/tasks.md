# Snapshot Migration タスク

## 1. integration.rs: snapshot_options() 除去とセマンティック移行

- [x] 1.1 `SNAPSHOT_PIXEL_TOLERANCE` 定数および `SnapshotOptions` import を削除
- [x] 1.2 `test_integration_application_startup`: snapshot → `get_by_label("No workspace open.")` アサーション（既存チェック活用）
- [x] 1.3 `test_integration_workspace_and_tabs`: 3箇所の snapshot → ファイルツリー表示・タブ開閉の状態アサーション
- [x] 1.4 `test_integration_view_modes`: 3箇所の snapshot → ViewMode 状態 + ラベル存在アサーション
- [x] 1.5 `test_integration_update_buffer`: snapshot → UpdateBuffer 後のバッファ内容アサーション
- [x] 1.6 `test_integration_save_document`: snapshot → SaveDocument 後のファイル内容アサーション
- [x] 1.7 `test_integration_multiple_documents_and_navigation`: 2箇所の snapshot → タブ数・アクティブタブ状態アサーション
- [x] 1.8 `test_integration_preview_with_diagram_content`: snapshot → diagram セクション存在アサーション
- [x] 1.9 `test_integration_workspace_with_subdirectory`: snapshot → ディレクトリエントリ表示アサーション
- [x] 1.10 `test_integration_workspace_panel_collapsed`: snapshot → show_workspace 状態アサーション
- [x] 1.11 `test_integration_split_mode_with_document`: 2箇所の snapshot → ViewMode 確認アサーション
- [x] 1.12 `test_integration_multiple_tabs_close`: 2箇所の snapshot → タブ全閉じ後の状態アサーション
- [x] 1.13 `test_integration_workspace_tree_expand_collapse`: snapshot → force_tree_open 状態アサーション
- [x] 1.14 `test_integration_preview_only_no_document`: snapshot → "No workspace open." ラベル存在アサーション

## 2. sample_fixture_tests.rs: snapshot 残存コード整理

- [x] 2.1 `snapshot_fixture()` 関数を削除（snapshot_name 引数を持つが、実際にはスナップショット不使用）
- [x] 2.2 `snapshot_diagrams_en`, `snapshot_diagrams_ja` テスト関数を削除（`#[ignore]` 付きの snapshot テスト）
- [x] 2.3 snapshot 関連コメント（「Snapshot tests」への言及, "§1.4 → Visual verification is delegated to snapshot"）を更新

## 3. tests/snapshots/ ディレクトリ削除

- [x] 3.1 `crates/katana-ui/tests/snapshots/` ディレクトリごと全 56 PNG ファイルを削除

## 4. Makefile 整理

- [x] 4.1 `test-update-snapshots` ターゲットを削除
- [x] 4.2 `test-integration` ターゲットのコメントを更新（snapshot 不使用を反映）

## 5. 依存整理

- [x] 5.1 `egui_kittest` の `SnapshotOptions` が不要になったか確認（`Harness` 自体は継続使用）
- [x] 5.2 不要な import / use 文を整理

## 6. 検証

- [x] 6.1 `cargo test --workspace` 全テストパス
- [x] 6.2 `make check` 全ゲートパス（fmt + clippy + IT + coverage）
- [x] 6.3 `tests/snapshots/` ディレクトリが存在しないことを確認

## 7. クリーンアップ

- [x] 7.1 ドキュメント更新
- [x] 7.2 Makefileの不要なターゲットの削除
- [x] 7.3 Makefileのコメント更新
