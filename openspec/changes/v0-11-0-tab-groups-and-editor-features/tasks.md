# Implementation Tasks: Tab Groups and Session UX Improvements (v0.11.0)

## Definition of Ready (DoR)

- [ ] proposal.md, design.md, specs が揃っていること
- [ ] 現行の tab pinning / close actions / workspace-scoped session restore (`views/top_bar.rs`, `app/action.rs`, `app/workspace.rs`) を確認していること

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

---

## 1. Session Model and Persistence Extension

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 1.1 既存 `workspace_tabs:{workspace_path}` payload を versioned session envelope に置き換える設計を実装する
- [ ] 1.2 session envelope に `tabs`, `active_path`, `expanded_directories`, `groups`, `version` を保持できるようにする
- [ ] 1.3 tab entry に pinned state を保存し、restore 時に `Document.is_pinned` へ反映する
- [ ] 1.4 legacy payload (`tabs`, `active_idx`, `expanded_directories`) を read-time upgrade で受けられるようにする
- [ ] 1.5 restore ON/OFF setting を workspace/session settings に追加し、既存 settings と serde 互換を保つ

### Definition of Done (DoD)

- [ ] workspace-scoped session save/load が grouped/pinned tab を扱えること
- [ ] 旧 payload からの read が defaults 補完で成立すること
- [ ] `make check` が exit code 0 で通過
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Tab Grouping UI and Runtime State

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 runtime tab group state (`id`, `name`, `color_hex`, `collapsed`, `members`) を定義する
- [ ] 2.2 tab context menu に group create / add / remove を追加し、1 tab が高々 1 group に所属する制約を守る
- [ ] 2.2.1 pinned tab には group add UI を出さない、または無効化し、grouped tab を pin した場合は membership を外す
- [ ] 2.3 group header の rename / color change / collapse toggle UI を実装する
- [ ] 2.4 `views/top_bar.rs` で group block を描画し、open tab order の最初の member 位置に anchored する projection を実装する
- [ ] 2.5 collapsed group が member tab を非表示にするだけで close しないことを保証する
- [ ] 2.5.1 active tab が collapsed group に属する場合は、その active member だけ visible に保つ
- [ ] 2.6 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 2.7 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] group create / rename / recolor / add / remove / collapse が一通り動作すること
- [ ] grouped tabs が workspace 再オープン後に復元されること
- [ ] group/pin の相互作用が design どおりに固定されていること
- [ ] `make check` が exit code 0 で通過
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Pinned Tab Safeguards

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 pinned tab の close button を非表示にし、tooltip 表示は維持する
- [ ] 3.2 `CloseDocument` が pinned tab を通常 close しないようにする
- [ ] 3.3 `CloseAllDocuments` / `CloseOtherDocuments` / `CloseDocumentsToRight` / `CloseDocumentsToLeft` が pinned tab をスキップするようにする
- [ ] 3.4 close shortcut から dispatch される close action でも pinned safeguard が有効であることを確認する
- [ ] 3.5 unpin 後は通常 close path に戻ることを確認する
- [ ] 3.6 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 3.7 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] pinned tab が通常 UI と batch close から削除されないこと
- [ ] unpin 後は通常 close できること
- [ ] `make check` が exit code 0 で通過
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. Verification and Recovery Paths

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 4.1 legacy session payload の read-time upgrade を test で確認する
- [ ] 4.2 grouped / pinned / restore setting OFF の各 session restore path を test で確認する
- [ ] 4.3 close policy の regression test を追加し、pinned tab が batch close から保護されることを確認する
- [ ] 4.4 実装途中に canonical order や session model の前提が崩れた場合、artifact が先に更新されていることを確認する

### Definition of Done (DoD)

- [ ] session persistence / group rendering / pin safeguards の主要 regression が test 化されていること
- [ ] upgrade path と restore setting OFF path が確認されていること
- [ ] `make check` が exit code 0 で通過
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
- [ ] 5.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.11.0`
- [ ] 5.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
