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

## 2. vendor/egui_commonmark ユニットテスト整備

- [ ] 2.1 `vendor/egui_commonmark` に対するユニットテストの構成方針を決定（`tests/` ディレクトリ or インラインテスト）
- [ ] 2.2 `pulldown.rs` の主要ロジック（テーブル解析、インライン装飾、取り消し線描画、中央寄せ等）のユニットテストを追加
- [ ] 2.3 Katana固有パッチ部分のリグレッションテストを追加
- [ ] 2.4 `cargo test -p egui_commonmark` で全テストパスを確認

---

## 3. Final Verification & Release Work

- [ ] 3.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 3.2 Ensure `make check` passes with exit code 0
- [ ] 3.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 3.4 Create a PR targeting `master`
- [ ] 3.5 Merge into master (※ `--admin` is permitted)
- [ ] 3.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.6.2`
- [ ] 3.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
