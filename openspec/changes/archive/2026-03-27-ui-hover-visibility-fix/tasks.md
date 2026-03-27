## 1. Vendor (`egui_commonmark`) のAPI拡張

- [x] 1.1 `vendor/egui_commonmark/src/lib.rs` で、`CommonMarkViewer` 構造体に `active_bg_color: Option<egui::Color32>` と `hover_bg_color: Option<egui::Color32>` を追加し、ビルダーメソッドを定義する
- [x] 1.2 `vendor/egui_commonmark/src/parsers/pulldown.rs` の描画ロジックを修正し、`active_bg_color` / `hover_bg_color` がセットされていればその色を使用し、設定されていなければ従来のハードコード値（フォールバック）を使用するように変更する

## 2. Platform層 (テーマ機構) の修正

- [x] 2.1 `crates/katana-platform/src/theme/types.rs` の `PreviewColors` に、`current_line_background: Rgba` と `hover_line_background: Rgba` を追加する（`#[serde(default)]` 属性を用いて互換性を維持する）
- [x] 2.2 `crates/katana-platform/src/theme/presets/dark/*.rs` 内の `current_line_background` および `hover_line_background` （Code / Preview 両方）を、「白（RGB: 255, 255, 255）」の透過色（Alpha: 15〜25など）へ一括変更し、視認性を高める
- [x] 2.3 `crates/katana-platform/src/theme/presets/light/*.rs` についても「黒（RGB: 0, 0, 0）」の透過色（Alpha: 15〜25など）へ一括調整し、視認性を全体的に向上させる

## 3. UI層 (プレビューペイン) の機能統合

- [x] 3.1 `crates/katana-ui/src/preview_pane_ui.rs` にて、テーマ設定から取得した `preview.current_line_background` および `preview.hover_line_background` を `egui::Color32` に変換する
- [x] 3.2 変換した背景色を `CommonMarkViewer::new()` に渡し、エディタとの連携時（カーソル移動およびホバー時）に、プレビュー上でもテーマ準拠の色でハイライトされるよう描画パイプラインを結合する

## 4. Verification

- [x] 4.1 プロジェクト全体で `cargo test` (`test_in_memory_cache_service`の修正を含む既存テスト)、各種 linter (`make check` / `ast_linter.sh`) を実行し、静的解析エラー・カバレッジ違反が発生していないことを確認する
- [x] 4.2 コントラストオフセット機能（`TemaColors::with_contrast_offset`）の単体テストを適宜修正する
- [x] 4.3 UIを手動起動し、ライト・ダークの両テーマで、エディタとプレビューをホバー・クリックした際に背景が明瞭に浮かび上がるかを検証する
