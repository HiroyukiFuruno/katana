## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

---

## 1. Shared Refresh Entry Point

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 1.1 `RefreshDiagrams` の既存 call site（theme change / asset reload / preview refresh UI）を棚卸しし、internal rerender と user-triggered refresh / auto-refresh の責務境界を確定する
- [ ] 1.2 shared refresh action を shell 共通 chrome に追加し、CodeOnly / PreviewOnly / Split の全 view mode で同一導線から実行できるようにする
- [ ] 1.3 preview pane 専用 refresh ボタンを撤去し、preview 側には export / ToC など preview 固有操作だけを残す
- [ ] 1.4 refresh success / dirty skip / unchanged hash / reload failure の status / i18n 契約を追加する
- [ ] 1.5 自動更新 default 値の提案理由をユーザーへ提示し、`auto_refresh_interval_secs` の合意を取得する
- [ ] 1.6 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 1.7 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] shared refresh の canonical 導線が 1 つだけになり、Code / Preview / Split で同じ挙動になる
- [ ] internal rerender 経路は disk reload を伴わないまま維持される
- [ ] auto-refresh default 値はユーザー合意済みである
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 2. Hash-Managed Refresh And Settings

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 active document ごとに「last imported disk hash」を保持する状態を追加し、preview 再描画 hash と責務を分離する
- [ ] 2.2 初回 load / successful save / successful reload の各経路で last imported disk hash が正しく更新されるようにする
- [ ] 2.3 manual refresh は on-disk hash に差分があるときだけ reload 判定へ進み、差分がなければ no-op とする
- [ ] 2.4 auto-refresh polling を active document 対象で実装し、enable / interval を `behavior` settings に追加する
- [ ] 2.5 clean 文書では hash 差分検知後に `FilesystemService::load_document()` から buffer を再読込し、dirty 文書では render-only refresh + warning に留める
- [ ] 2.6 dirty 文書で検出した同一 external hash に対して warning を重複表示しない pending 状態管理を追加する
- [ ] 2.7 read failure 時は current buffer を維持し、recoverable error を status bar へ出す
- [ ] 2.8 workspace refresh は tree rescan 専用のままにし、document refresh と混線しないように整理する
- [ ] 2.9 hash lifecycle / manual no-op / clean reload / dirty skip / warning dedupe / auto-refresh interval / read failure / settings persistence の regression test を追加する

### Definition of Done (DoD)

- [ ] 外部エディタで更新された clean 文書は shared refresh または auto-refresh で取り込める
- [ ] hash 差分がなければ manual / automatic refresh のどちらでも不要 reload は起きない
- [ ] dirty 文書は manual / automatic refresh でも silent overwrite されない
- [ ] 同一 external hash に対する dirty warning は 1 回だけ表示される
- [ ] auto-refresh の設定値は保存・復元される
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 3. Nested Task List Rendering

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 `vendor/egui_commonmark` の delayed / wrapped event 処理で元の event index を保持し、nested parsing でも task list 判定が失われないようにする
- [ ] 3.2 task list 親行では checkbox だけを先頭マーカーとして表示し、余計な bullet を出さないようにする
- [ ] 3.3 nested child list の bullet / ordered marker / indentation は従来表現を維持する
- [ ] 3.4 native task list（`[x]`, `[ ]`）と custom state（`[/]`, `[-]`, `[~]`）の両方に対する parser / preview regression test を追加する
- [ ] 3.5 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 3.6 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] nested task list の親行から二重マーカーが消え、子リストの表現は回帰していない
- [ ] parser 層と KatanA preview 層の両方で回帰が検出できる
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 4. End-to-End Verification

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 4.1 外部エディタで Markdown を変更し、CodeOnly / PreviewOnly / Split の各モードから shared refresh で反映できることを検証する
- [ ] 4.2 auto-refresh interval 経過後に clean 文書は自動反映され、dirty 文書は warning のみで保護されることを確認する
- [ ] 4.3 unchanged hash 時には manual / automatic refresh のどちらでも no-op になることを確認する
- [ ] 4.4 shared refresh 実行時に図・画像キャッシュが適切に再描画され、theme change 等の internal rerender 経路は従来どおり render-only で動くことを確認する
- [ ] 4.5 dirty 文書で同一 external hash を維持したまま複数 polling interval が経過しても warning が重複しないことを確認する
- [ ] 4.6 `katana-ui` と vendored parser と settings の対象テストを実行し、nested task list と refresh contract の回帰がないことを確認する

### Definition of Done (DoD)

- [ ] ユーザー操作・自動更新・内部自動再描画・nested task list 表示の各経路が spec どおりに整合している
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 5. Final Verification & Release Work

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 5.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 5.2 Ensure `make check` passes with exit code 0
- [ ] 5.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 5.4 Create a PR targeting `master`
- [ ] 5.5 Merge into master (※ `--admin` is permitted)
- [ ] 5.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.8.6`
- [ ] 5.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
