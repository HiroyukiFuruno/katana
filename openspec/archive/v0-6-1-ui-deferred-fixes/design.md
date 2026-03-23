## Architecture Changes

- `egui_commonmark` 内の `pulldown.rs` において、`Tag::Table` 処理全体を `egui::Grid` から `egui_extras::TableBuilder` ベースへ書き換えます。
- インラインコードのベースライン配置指示を `egui::Align::Center` に変更します。
- ワークスペースのディレクトリ要素における相互作用範囲（インタラクションエリア）をアイコンのみからテキスト領域全体へ拡張します。
- タブバー（Tab viewer）のスクロール状態をアクティブタブの位置に応じて自動調整する処理を追加します。
- ライトモード時のエクスポートボタン・履歴ボタンの配色定義（Theme設定やカスタムColor32定義）を調整します。

### Components Affected

- `vendor/egui_commonmark/src/parsers/pulldown.rs`
- `crates/katana-ui/tests/preview_pane.rs`

## Data Flow

特になし

## Error Handling

特になし
