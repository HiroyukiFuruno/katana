## ADDED Requirements

### Requirement: UI インタラクションテストはポインターイベントを使ってボタンクリックを simulate する
統合テストは `egui::Event::PointerButton` のプレス・リリースペアを inject することで実際のユーザークリックを simulate しなければならない（SHALL）。`trigger_action` の直接呼び出しのみでは UI 層の検証として不十分である。

#### Scenario: Split 方向トグルボタンのクリック simulate
- **WHEN** Split モードが有効なタブが存在し、`⇕` ボタンが描画されている
- **THEN** ボタン rect の center 座標に `PointerButton { pressed: true }` → `PointerButton { pressed: false }` の順でイベントを inject し、`harness.step()` を2回実行した後、`state.active_split_direction()` が `Vertical` に変化していること

#### Scenario: Pane Order トグルボタンのクリック simulate
- **WHEN** Split モードが有効なタブが存在し、`📄|👁` ボタンが描画されている
- **THEN** 同様のクリック simulate 後、`state.active_pane_order()` が `PreviewFirst` に変化していること

### Requirement: UI ボタンは一意に特定できる ID を持たなければならない
テストから安定してウィジェットを特定するために、Split 切り替えボタンは `egui::Button::new(...).sense(...)` の代わりに `ui.add(egui::Button::new(...))` で描画し、ボタンラベルを ASCII または既知の Unicode 文字列で統一しなければならない（SHALL）。

#### Scenario: ボタンラベルで検索可能
- **WHEN** Split モード UI が描画されている
- **THEN** `harness.query_by_label("split-dir-toggle")` または `harness.query_all_by_value("⇕")` でボタンが1件以上ヒットすること

### Requirement: テストは2層に分離する
テスト関数は「ロジック層（AppAction → state）」と「UI 層（UI 操作 → AppAction → state）」の2つに明確に分けなければならない（SHALL）。

#### Scenario: ロジック層テストは trigger_action を使う
- **WHEN** `SetSplitDirection(Vertical)` を `trigger_action` 経由で呼ぶ
- **THEN** `active_split_direction()` が `Vertical` を返す
- **AND** `settings.settings().split_direction` は変化しない（永続化されない）

#### Scenario: UI 層テストはポインターイベントを使う
- **WHEN** ユーザーが Split 方向トグルボタンをクリックする（イベント inject）
- **THEN** `active_split_direction()` が変化する
- **AND** 再クリックで元の方向に戻ること（ラウンドトリップ検証）
