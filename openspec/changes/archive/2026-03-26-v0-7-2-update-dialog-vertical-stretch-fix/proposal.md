## Why

v0.7.1 で `egui::Window` に `.resizable(false)` を追加したが、内部の `ScrollArea::auto_shrink([false; 2])` が残存しており、「最新版です」などコンテンツが少ない分岐でも ScrollArea が縦方向に無制限に伸びてしまう。ユーザーが報告したダイアログ縦伸びバグ（スクリーンショット確認済み）として v0.7.2 で即時修正する。

## What Changes

- `render_update_window` 内の `ScrollArea::auto_shrink([false; 2])` を `[true, true]` に変更し、コンテンツに即した高さ自動縮小を有効化する。
- TDD(RED) として、更新確認ウィンドウが「最新版」状態で不当に大きな高さを持たないことを検証する integration test を追加する。
- CHANGELOG (EN/JA) に v0.7.2 パッチノートを追記する。

## Capabilities

### New Capabilities

（なし）

### Modified Capabilities

- `update-dialog`: ScrollArea の auto_shrink 設定変更により、ウィンドウ高さがコンテンツに追従するよう挙動が変わる。

## Impact

- `crates/katana-ui/src/shell_ui.rs`: `render_update_window` 関数内 1 行変更
- `tests/integration.rs` または `katana-ui` unit tests: 縦伸び再発防止テスト追加
- `CHANGELOG.md` / `CHANGELOG.ja.md`: v0.7.2 エントリ追記
