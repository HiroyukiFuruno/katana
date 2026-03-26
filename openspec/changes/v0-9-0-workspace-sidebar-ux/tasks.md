## Definition of Ready (DoR)

- [ ] proposal.md, design.md, specs が揃っていること
- [ ] 対象バージョン 0.9.0 の変更 ID とスコープが確認されていること
- [ ] 現行のワークスペースヘッダー配置と左レール化対象（表示切替・検索・履歴）を `shell_ui.rs` で再確認していること

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

---

## 1. 左アクティビティレールの実装

- [ ] 1.1 `shell_ui.rs` に固定幅の左アクティビティレールを追加し、ワークスペース表示切り替え・検索・履歴ボタンを縦配置する
- [ ] 1.2 ワークスペース表示切り替えを既存の `show_workspace` 状態と連動させ、ペインを閉じてもレールが残るようにする
- [ ] 1.3 履歴ボタンを既存の `settings.workspace.paths` と `OpenWorkspace` / `RemoveWorkspace` に接続し、最近のワークスペース履歴メニューをレール側へ移す
- [ ] 1.4 左レールのアイコンを既存資産のまま一段大きく描画し、ホバー説明とアクティブ状態を整理する
- [ ] 1.5 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 1.6 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] 左レールからワークスペース表示切り替え・検索・履歴が利用できること
- [ ] ワークスペースペインを閉じても左レールが残り、同じ導線で再表示できること
- [ ] `make check` が exit code 0 で通過
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. ワークスペースヘッダーの再配置

- [ ] 2.1 ワークスペースペインから `Workspace` / `ワークスペース` の見出し文言を削除する
- [ ] 2.2 更新ボタンをヘッダー先頭側へ移し、全展開・全閉ボタンを末尾側へ再配置する
- [ ] 2.3 フィルタートグルと正規表現入力 UI をヘッダー内に維持し、既存挙動が変わらないことを確認する
- [ ] 2.4 ワークスペースの表示切り替え後も展開状態・フィルター状態が破綻しないように整合を取る
- [ ] 2.5 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 2.6 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] 見出し文言が消え、更新と全展開・全閉の配置が要求どおりになっていること
- [ ] フィルターの表示・入力・絞り込み結果が既存どおり動作すること
- [ ] `make check` が exit code 0 で通過
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. 検索導線と回帰確認

- [ ] 3.1 左レールの検索ボタンから既存の検索モーダルを開けるようにし、ショートカット導線を維持する
- [ ] 3.2 レール化に伴うツールチップ・i18n 文言・アクセシビリティラベルを整理する
- [ ] 3.3 UI テストまたはハーネスで、左レール起点の表示切り替え・検索・履歴操作の回帰を追加確認する
- [ ] 3.4 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 3.5 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] 検索はショートカットと左レールの両方から開けること
- [ ] 履歴メニューの開く・削除・ワークスペース再オープンが回帰していないこと
- [ ] `make check` が exit code 0 で通過
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. Final Verification & Release Work

- [ ] 4.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 4.2 Ensure `make check` passes with exit code 0
- [ ] 4.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 4.4 Create a PR targeting `master`
- [ ] 4.5 Merge into master (※ `--admin` is permitted)
- [ ] 4.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.9.0`
- [ ] 4.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
