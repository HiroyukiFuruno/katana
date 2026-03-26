# v0.7.2: 更新確認ダイアログ縦伸びバグ修正

## 1. 実装

### 1.1 TDD(RED): 縦伸び再現テストを追加

- `crates/katana-ui/src/shell_ui.rs` の `render_update_window` における `auto_shrink([false; 2])` が縦伸びを引き起こすことを integration test で証明する（この段階ではテスト FAIL）

### 1.2 バグ修正(GREEN): auto_shrink を修正

- `ScrollArea::auto_shrink([false; 2])` → `auto_shrink([true, true])` に変更
- テストが GREEN になることを確認

### 1.3 CHANGELOG 更新

- `CHANGELOG.md` に v0.7.2 エントリを追加（EN）
- [x] v0.7.3 アップデートプロセスの改修 (Task 4)
- `CHANGELOG.ja.md` に v0.7.2 エントリを追加（JA）

## 2. リリース

### 2.1 commit & make release

- 変更を commit して `make release VERSION=0.7.2 FORCE=1` を実行
