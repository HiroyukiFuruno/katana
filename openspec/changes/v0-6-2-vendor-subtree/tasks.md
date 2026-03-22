## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

## 1. vendor/egui_commonmark サブツリー化の実行

- [ ] 1.1 `vendor/egui_commonmark` 全体を履歴から退避、あるいはコミット上で一旦削除する。
- [ ] 1.2 `git subtree add` を使用して、指定されたリモートリポジトリ・特定のリビジョンから `vendor/egui_commonmark` を導入する。
- [ ] 1.3 `pulldown.rs` 等に存在した Katana 固有の改修内容を再抽出・パッチとして分離してコミットする。

### Definition of Done (DoD)

- [ ] サブツリー化が完了し、`make test` など全ての検証が元通りパスすること。
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Final Verification & Release Work

- [ ] 2.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 2.2 Ensure `make check` passes with exit code 0
- [ ] 2.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 2.4 Create a PR targeting `master`
- [ ] 2.5 Merge into master (※ `--admin` is permitted)
- [ ] 2.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.6.2`
- [ ] 2.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
