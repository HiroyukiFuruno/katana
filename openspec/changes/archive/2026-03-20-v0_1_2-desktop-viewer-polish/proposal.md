## Why

v0.5.0 時点の UI テストは `trigger_action` や `AppState` の直接変更に寄りすぎており、実際のユーザー操作から `egui::Response` が生成される経路を十分に検証できていない。
そのため、シナリオ上は pass しても「クリックしても反応しない」「一度切り替えると戻せない」「別の widget に入力が吸われる」といった UI 起因のデグレードを見逃す余地がある。

v1.0.0 前の最終 patch として、v0.5.0 の品質保証ラインを引き上げるため、UI 検証を「状態変化確認」中心から「実際の response を伴う操作評価」中心へ拡張する。

## What Changes

### UI interaction verification
- `egui` の実イベント入力を通し、press/release を伴うクリックや選択操作を評価するテスト方針を導入する
- 描画された widget の `Response.rect` または安定した widget 識別子を起点に、実際の UI 応答を評価する共通パターンを整備する
- `trigger_action` ベースのテストは「ロジック層」、ポインターイベント駆動のテストは「UI interaction 層」として責務を分離する

### Release-critical scenario expansion
- v0.5.0 で重要度の高い操作シナリオを、実 UI 応答を伴う検証へ拡充する
- 少なくとも workspace/file selection、layout toggle、settings 操作、v0.5.0 の追加 UI を代表する導線を対象にする

## Capabilities

### New Capabilities
- `ui-response-verification`: `egui::Response` を伴う実 UI 操作の検証パターンを提供する

### Modified Capabilities
- `workspace-shell`: 統合テストが state 直接変更だけでなく、実際の UI interaction を評価する構成に拡張される

## Impact

- `crates/katana-ui/tests`: UI interaction helper とシナリオテストの追加
- `crates/katana-ui/src`: テスト対象 widget に安定した識別子や探索しやすい構造を追加する可能性がある
- 既存の `trigger_action` ベーステストは削除せず、責務を明確化して維持する
- 依存追加は原則行わず、既存の `egui` / `egui_kittest` を活用する
