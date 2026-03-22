## Architecture Changes

- `egui_commonmark` 内の `pulldown.rs` において、`Tag::Table` 処理全体を `egui::Grid` から `egui_extras::TableBuilder` ベースへ書き換えます。
- インラインコードのベースライン配置指示を `egui::Align::Center` に変更します。

### Components Affected

- `vendor/egui_commonmark/src/parsers/pulldown.rs`
- `crates/katana-ui/tests/preview_pane.rs`

## Data Flow

特になし

## Error Handling

特になし
