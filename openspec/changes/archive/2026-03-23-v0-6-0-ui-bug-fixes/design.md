## Architecture Changes

KatanaUIの `shell.rs` 内 `handle_select_document`、`KatanaApp::update` のタイミング処理を改善する小規模な論理変更です。

### Components Affected

- `shell.rs`: `handle_select_document`, `full_refresh_preview`, `KatanaApp`初期化処理
- `shell_ui.rs`: `KatanaApp::update`
- `svg_loader.rs`: エラーハンドリング
- `egui_commonmark/pulldown.rs`: Blockquote 描画ロジック

## Data Flow

特筆すべきデータの流れの変更はありません。既存の `Source` → `Hash` 評価パイプラインを正しく通過するようになります。

## Error Handling

SVG画像の取得エラー（無効なバッジ等）は `LoadError::NotSupported` または `Loading` に滞留しない適切なエラーとして扱い、再レンダリング無限ループによるCPUスパイクを抑止します。
