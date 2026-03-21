## Definition of Ready (DoR)

- **前提条件**: `desktop-viewer-polish-v0.5.0` の内容が `main` に反映されていること
- **目的の明確化**: 本 change は個別不具合の修正ではなく、UI の実 interaction を評価できる検証計画であること
- **対象範囲**: `katana-ui` の release-critical な UI 導線を優先すること

## Branch Rule

タスクグループ（##単位）ごとに 1 セッションで以下を回す:

1. ブランチ作成: `desktop-viewer-polish-v0.5.1-task{N}`
2. 実装 / PR / セルフレビュー / マージ: `/opsx:apply`

---

## 1. テスト戦略の棚卸し

- [x] 1.1 現在の `katana-ui` テストを「ロジック層」と「UI interaction 層」に分類する
- [x] 1.2 state 直接変更だけでは守れない release-critical シナリオを列挙する
- [x] 1.3 v0.5.0 patch として最低限守る UI 導線を確定する

### Definition of Done (DoD)
- [x] 既存テストの責務が一覧化されていること
- [x] interaction test が必要な導線が合意可能な粒度で定義されていること

---

## 2. Response ベースの interaction helper 整備

- [x] 2.1 `egui` の実イベントを使って press/release を流す helper を整備する
- [x] 2.2 widget の `Response.rect` または安定 ID を取得する locator パターンを整備する
- [x] 2.3 interaction test では state 直接変更を使わない運用ルールをテストコード上で明示する

### Definition of Done (DoD)
- [x] クリック対象の widget を test から安定して特定できること
- [x] 1 回の click を press/release を含む形で再利用できること
- [x] helper の利用例が 1 本以上のテストで示されていること

---

## 3. Release-critical シナリオの拡充

- [x] 3.1 workspace tree からの file selection を interaction test で検証する
- [x] 3.2 split/layout toggle の往復操作を interaction test で検証する
- [x] 3.3 settings の主要操作を interaction test で検証する
- [x] 3.4 v0.5.0 で追加された UI 導線について、少なくとも 1 本ずつ interaction test を用意する
  - [x] export 導線
  - [x] terms agreement 導線

### Definition of Done (DoD)
- [x] 各シナリオが実際の UI 操作から期待結果に到達することを確認できること
- [x] 少なくとも 1 件は、従来の state 直接変更テストでは見逃し得た不具合を interaction test が検知できる構成になっていること

---

## 4. 既存テストの責務整理

- [x] 4.1 `trigger_action` ベースの既存テストにロジック層テストであることを明示する
- [x] 4.2 interaction test と重複するシナリオは、責務の違いが分かる命名へ整理する
- [x] 4.3 snapshot テストとの使い分けをコメントまたは補助ドキュメントで明文化する

### Definition of Done (DoD)
- [x] 新旧テストの役割分担が読み取れること
- [x] 保守時に「どの層で落ちたか」が判別しやすい状態になっていること

---

## 5. 検証

- [x] 5.1 `cargo test -p katana-ui --test integration` が pass すること
- [x] 5.2 必要な関連テストが pass すること
- [x] 5.3 `make check-light` または同等の軽量検証が pass すること

### Definition of Done (DoD)
- [x] interaction test を追加した状態で CI 相当の検証が安定して通ること
