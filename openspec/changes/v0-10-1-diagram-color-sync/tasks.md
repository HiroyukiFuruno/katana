## Definition of Ready (DoR)

- [ ] proposal.md, design.md, specs が揃っていること
- [ ] 対象バージョン 0.10.1 の bugfix scope が確認されていること
- [ ] `shell_ui.rs` / `preview_pane/renderer.rs` / `mermaid_renderer` / `plantuml_renderer` の現行 theme lookup 経路を確認していること

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

---

## 1. Diagram Theme Context の導入

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 1.1 `ThemeColors` から diagram 用 `DiagramRenderTheme` を生成する helper を定義する
- [ ] 1.2 `preview_pane/core_render.rs` の render job に request-scoped theme snapshot を載せる
- [ ] 1.3 `preview_pane/renderer.rs` の dispatch 経路を更新し、diagram backend が explicit theme parameter を受けるようにする
- [ ] 1.4 `mermaid_renderer` と `plantuml_renderer` から render path 上の `DiagramColorPreset::current()` 直接参照を外す
- [ ] 1.5 同じ helper に依存する diagram backend がある場合は theme source を統一する
- [ ] 1.6 theme mapping に追加ルールが必要と分かった場合は、コード継続前に `design.md` / `specs` / `tasks.md` を更新する

### Definition of Done (DoD)

- [ ] diagram renderer は request-scoped theme snapshot を使って描画されること
- [ ] render path の global preset lookup が主要経路から外れていること
- [ ] `make check` が exit code 0 で通過
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Theme-Aware Cache Key と Refresh Path の修正

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 `preview_pane/renderer.rs` の diagram cache key を dark/light bool 依存から `theme_fingerprint` 依存へ置き換える
- [ ] 2.2 同じ dark mode 内で `preview.text` 等が変わった場合も cache miss になることを確認する
- [ ] 2.3 `shell_ui.rs` の theme change → `RefreshDiagrams` 経路で、current theme snapshot に基づく再描画になるよう整合を取る
- [ ] 2.4 fingerprint が過剰 invalidation または不足 invalidation を起こすと分かった場合は artifact を先に更新する

### Definition of Done (DoD)

- [ ] theme change 後の diagram cache が stale 色を再利用しないこと
- [ ] `RefreshDiagrams` 後の active / inactive preview が current theme と一致すること
- [ ] `make check` が exit code 0 で通過
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Regression Test と Verification

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 unit test を追加し、theme fingerprint が変わると diagram cache key も変わることを確認する
- [ ] 3.2 Mermaid と PlantUML の少なくとも 2 経路で、theme / preview text color 変更後に renderer 入力が current theme と一致することを確認する
- [ ] 3.3 custom preview.text を同一 dark mode 内で変更した場合も diagram が再描画されることを確認する
- [ ] 3.4 static preset は fallback 用に残してよいが、preview render path の correctness 判定に使っていないことを確認する
- [ ] 3.5 実装途中に design 前提と乖離した点があれば、関連 artifact が先に更新されていることを確認する

### Definition of Done (DoD)

- [ ] Mermaid / PlantUML の regression が test で再発防止されていること
- [ ] custom preview color 変更で diagram text color が追従すること
- [ ] `make check` が exit code 0 で通過
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. Final Verification & Release Work

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 4.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 4.2 Ensure `make check` passes with exit code 0
- [ ] 4.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 4.4 Create a PR targeting `master`
- [ ] 4.5 Merge into master (※ `--admin` is permitted)
- [ ] 4.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.10.1`
- [ ] 4.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
