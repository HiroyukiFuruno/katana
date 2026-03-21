## Context

`katana-ui` の統合テストは `egui_kittest::Harness` を利用しているが、現在は `trigger_action` や `app_state_mut()` を経由して state を直接動かす比率が高い。
この形は `process_action` や保存ロジックの検証には有効だが、実際の UI で発生する以下の層を通っていない。

- pointer/key input が frame に入る
- widget が `Response` を返す
- `Response.clicked()` や selection change を契機に `AppAction` が発火する
- state と描画結果が更新される

今回の v0.5.1 は、既知不具合の個別修正を主目的にするのではなく、そうした不具合を事前に検知できるテスト基盤とシナリオを v0.5.0 の patch として整備する変更である。

## Goals / Non-Goals

**Goals:**
- 実際の `egui` 入力イベントを使い、UI の `Response` を経由する interaction test パターンを定義する
- 重要 widget をテストから安定して特定できる仕組みを整える
- テスト責務を「ロジック層」と「UI interaction 層」に分離し、どちらで何を守るかを明確にする
- v0.5.0 で重要な操作シナリオを response-based test で守る

**Non-Goals:**
- UI 自動テストを snapshot 比較だけに置き換えること
- 全既存テストを response-based test に全面移行すること
- 新しい GUI テストフレームワークを導入すること
- 個別不具合の修正計画をこの change の中心にすること

## Decisions

### Decision 1: response-based verification を interaction test の基準にする

UI interaction 層のテストでは、state の直接変更を禁止し、実入力イベントから widget の `Response` が成立する経路を通す。

評価対象は以下のいずれか、または両方とする。

- `Response.rect` を起点にしたクリック位置の評価
- widget ID / test selector を用いた対象 widget の特定と、その操作結果の評価

要点は「最終的に state が変わったか」だけでなく、「その変化が UI の応答を通じて起きたか」を検証することである。

### Decision 2: 入力は press/release の 2 段で扱う

`egui` のクリック成立条件に合わせ、interaction helper は press と release を別イベントとして流す。

```rust
input.events.push(Event::PointerButton {
    pos,
    button: egui::PointerButton::Primary,
    pressed: true,
    modifiers: egui::Modifiers::NONE,
});

input.events.push(Event::PointerButton {
    pos,
    button: egui::PointerButton::Primary,
    pressed: false,
    modifiers: egui::Modifiers::NONE,
});
```

実装上は `egui_kittest` の公開 API で同等のことができるならそれを優先してよいが、テストの意味としては「実イベントで click を成立させる」ことを必須要件とする。

### Decision 3: widget targeting は visible label 依存を避ける

Unicode ラベルや表示文言だけに依存すると、i18n・記号・レイアウト変更でテストが不安定になる。
そのため、release-critical な操作対象には次のいずれかを採用する。

- 安定した `egui::Id`
- test helper から再利用できる widget locator
- `Response.rect` を取得するための収集ポイント

visible label で十分安定している箇所はそのまま使ってよいが、記号ボタンや i18n 文字列依存の強い箇所は識別子ベースへ寄せる。

### Decision 4: テストを 2 層に分けて維持する

| 層 | 目的 | 許可する手法 |
|---|---|---|
| ロジック層 | `AppAction` 処理、永続化、副作用の検証 | `trigger_action`, state 直接操作 |
| UI interaction 層 | 実操作から `Response` を経由して正しく動くかの検証 | pointer/key event, widget targeting, frame 更新 |

これにより、既存テストを無駄に壊さずに coverage を厚くできる。

### Decision 5: v0.5.0 patch として守るシナリオを先に定義する

interaction test は無制限に増やすのではなく、v0.5.0 の品質に直結する導線から優先する。

最低限の対象:
- workspace tree からの file selection
- split/layout toggle の往復操作
- settings の主要操作
- v0.5.0 で追加された UI 導線
  - export 導線
  - terms agreement 導線

各導線について「ユーザーが 1 回以上クリック/選択し、その結果として UI/state が期待どおりになる」シナリオを最低 1 本用意する。

## Risks / Trade-offs

**[Risk] `egui_kittest` から `Response.rect` を直接拾いにくい**
- **Mitigation**: widget ID 付与や helper 経由の locator を先に整備する

**[Risk] interaction test が文言変更に弱くなる**
- **Mitigation**: 記号ボタンや i18n 依存が強い箇所は label ではなく ID で特定する

**[Trade-off] テストコードは冗長になる**
- press/release、frame 更新、rect 取得の手順が増える
- ただし、UI 起因のデグレード検知力向上を優先する

**[Trade-off] 既存テストとの二重管理が発生する**
- 一部シナリオは logic と interaction の両方に存在する
- ただし、責務が異なるため重複ではなく防御層として扱う

## Open Questions

- `egui_kittest` の公開 API だけで十分に locator/interaction を表現できるか
- v0.5.0 の追加 UI のうち、どこまでを integration test、どこからを lower-level test とするか
- 安定 ID を本番コードに常設するか、test 向け helper で吸収するか
