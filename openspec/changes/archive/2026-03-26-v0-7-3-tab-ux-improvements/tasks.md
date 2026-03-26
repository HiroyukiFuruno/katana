## Definition of Ready (DoR)

- [ ] proposal.md, design.md, specs が揃っていること
- [ ] 対象バージョン 0.7.3 のブランチ戦略が確認されていること

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

---

## 1. ハイライト背景バグ調査・修正

- [x] 1.1 再現条件の調査（特定GPU/描画環境、macOS バージョン、egui のバージョン）。※技術負債メモに記録されている「コードブロック背景色が `syntect` に強制上書きされる問題」と同一事象の可能性があるため、調査時に関連性を確認すること
- [x] 1.2 egui の `Painter::rect_filled` / `Frame::fill` の描画パスを追跡し、背景が描画されないコードパスを特定
- [x] 1.3 修正実装（描画パス変更またはフォールバック追加）
- [x] 1.4 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [x] 1.5 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [x] 再現していた環境で背景が表示されることを確認
- [x] `make check` が exit code 0 で通過
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Source Tree ディレクトリアイテムのコンテキストメニュー追加

> **実装対象の明確化**: タブバー上のタブではなく、左ペインのファイルツリー（Source Tree）上のファイル/ディレクトリアイテムが対象。

- [x] 2.1 Source Tree のファイル/ディレクトリアイテムに右クリックイベントを追加
- [x] 2.2 Workspace ペインと共通のコンテキストメニューコンポーネントを統合（重複実装を避ける）
- [x] 2.3 「開く」「コピー」「削除」「名前変更」などの操作をコンテキストメニュー経由で実行できるよう実装
- [x] 2.4 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [x] 2.5 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [x] Source Tree のタブ右クリックでメニューが表示される
- [x] Workspace コンテキストメニューと同等の操作が実行できる
- [x] `make check` が exit code 0 で通過
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. タブ双方向移動の修正とUI/UX改善

- [x] 3.1 左→右ドラッグ移動が機能しない原因を特定（配列境界チェック）
- [x] 3.2 双方向ドラッグ移動のロジックを修正
- [x] 3.3 移動中のスナップアニメーション（egui `lerp`）を統一実装
- [x] 3.4 ユーザーへのUIスナップショット（動画等）の提示および動作報告
- [x] 3.5 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [x] 左→右・右→左双方向のドラッグ移動が正常動作する
- [x] `make check` が exit code 0 で通過
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

---

## 4. Final Verification & Release Work

- [ ] 4.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 4.2 Ensure `make check` passes with exit code 0
- [ ] 4.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 4.4 Create a PR targeting `master`
- [ ] 4.5 Merge into master (※ `--admin` is permitted)
- [ ] 4.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.7.3`
- [ ] 4.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
