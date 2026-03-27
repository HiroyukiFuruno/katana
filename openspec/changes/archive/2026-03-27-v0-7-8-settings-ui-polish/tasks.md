## 1. UI Integration (機能UIの実装)

- [x] 1.1 `crates/katana-ui/src/settings_window.rs` の外観 (Appearance) セクションに `ui_contrast_offset` を調整するスライダー（-100% 〜 +100% または -1.0 〜 1.0）を追加する。
- [x] 1.2 `crates/katana-ui/src/settings_window.rs` のシステム (System) セクションに `katana_ui::http_cache_loader::forget_all()` をトリガーする「HTTPキャッシュクリア」ボタンを追加する。
- [x] 1.3 必要に応じて、キャッシュクリア成功やファイルサイズなどのフィードバック表示を簡易的に実装する（トーストまたはログ出力）。

## 2. Layout Adjustments (レイアウト調整)

- [x] 2.1 `crates/katana-ui/src/settings_window.rs` の `render_custom_color_editor` 内の `egui::Grid` 描画を見直す。
- [x] 2.2 左列（テキストラベル）と右列（カラーピッカー）が垂直方向に中央揃えになるように、`ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), ...)` または `ui.allocate_ui_with_layout` 等を適用する。
- [x] 2.3 （必要であれば）他のGrid利用箇所のレイアウト・アライメントも同様に調整する。

## 3. Linter & Test バリデーション

- [x] 3.1 `settings_window.rs` の新たなUI追加後も Linter（`cargo clippy` / `katana-linter`）のルール違反がないことを確認する。
- [x] 3.2 統合テスト（`tests/integration.rs` 等）を実行し、設定ウインドウ表示でパニックや表示崩れ、フォーカスエラーが起きないことを確認する（`make check` 100% パス）。
