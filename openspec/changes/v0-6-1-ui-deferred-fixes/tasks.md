## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

## 1. P3 インラインコード配置最適化

- [ ] 1.1 `pulldown.rs` 内の `self.line.append_to` に使用される `egui::Align` を `BOTTOM` から `Center` へ変更。
- [ ] 1.2 `katana-ui/tests/preview_pane.rs` において、`html_...` となっている旧テストケースを `Center` 対応へと修正する。

### Definition of Done (DoD)

- [ ] インラインコードのアラインメントが中央寄りになり、テストが問題なくパスする
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 2. 実験機能 テーブル描画変更

- [ ] 2.1 `Tag::Table` 処理時に、現在の `egui::Grid::new` 実装を破棄し、`TableBuilder` への置き換えを実行する。
- [ ] 2.2 カラム間の罫線（`vlines`）を再実装し、文字揃えをサポートする。

### Definition of Done (DoD)

- [ ] テーブルがMarkdown仕様通りに罫線を備えて描画される
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Final Verification & Release Work

- [ ] 3.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 3.2 Ensure `make check` passes with exit code 0
- [ ] 3.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 3.4 Create a PR targeting `master`
- [ ] 3.5 Merge into master (※ `--admin` is permitted)
- [ ] 3.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.6.1`
- [ ] 3.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
