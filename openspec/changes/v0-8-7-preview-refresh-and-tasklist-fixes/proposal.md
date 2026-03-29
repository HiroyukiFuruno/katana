## Why

v0.8.x の OpenSpec 編集フローには、日常利用を阻害する 2 つの欠陥が残っています。ネストされた task list が preview 上で `bullet + checkbox` の二重マーカーになり可読性を崩していること、そして現在の更新導線が preview 専用の `RefreshDiagrams` に留まり、外部エディタで変更した Markdown を hash 管理で検知して再読込できないことです。

## What Changes

- preview ペイン専用の更新導線を、Code / Preview / Split の全モードから使える共有の文書更新ボタンへ置き換える
- active document ごとに「最後に取り込んだディスク内容の hash」を保持し、手動更新では hash に差分があるときだけ再読込する
- 定期的にディスク内容の hash を確認し、clean な文書では自動更新、dirty な文書では上書きせず warning だけを出す
- 自動更新の有効/無効と確認間隔を設定化し、提案 default 値を設計内で明示したうえでユーザー合意を前提にする
- ネストされた task list 項目では checkbox 自体を親行の唯一の先頭マーカーとし、子リストは従来の bullet / ordered list 表現と indentation を維持する
- nested task list と外部編集リロードの regression test / UI verification を追加する

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities

- `markdown-authoring`: nested task list の preview 表現と、hash 管理された manual / automatic refresh 契約を更新する
- `settings-persistence`: 自動更新の有効/無効と確認間隔の既定値・保存・復元契約を追加する
- `workspace-shell`: active document refresh を共通 chrome から実行できるようにする

## Impact

- `crates/katana-ui/src/views/top_bar.rs`
- `crates/katana-ui/src/views/panels/preview.rs`
- `crates/katana-ui/src/app/action.rs`
- `crates/katana-ui/src/app/document.rs`
- `crates/katana-ui/src/app/workspace.rs`
- `crates/katana-ui/src/shell_ui.rs`
- `crates/katana-ui/src/app_state.rs`
- `crates/katana-core/src/document.rs`
- `crates/katana-ui/src/shell_logic.rs` または共有 hash utility
- `crates/katana-platform/src/settings/types/behavior.rs`
- `crates/katana-platform/src/settings/defaults.rs`
- `crates/katana-ui/src/settings/tabs/behavior.rs`
- `vendor/egui_commonmark/src/parsers/pulldown.rs`
- `vendor/egui_commonmark_backend/src/pulldown.rs`
- `crates/katana-ui/tests/sample_fixture_tests.rs`
- `crates/katana-ui/src/shell_tests.rs`
- `crates/katana-platform/src/settings/tests.rs`
- refresh / preview / task list / behavior settings 関連の i18n と UI regression test
