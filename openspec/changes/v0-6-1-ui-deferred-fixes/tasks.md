## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

## 1. P3 インラインコード配置最適化

- [x] 1.1 `pulldown.rs` 内の `self.line.append_to` に使用される `egui::Align` を `BOTTOM` から `Center` へ変更。
- [x] 1.2 `katana-ui/tests/preview_pane.rs` において、`html_...` となっている旧テストケースを `Center` 対応へと修正する。

### Definition of Done (DoD)

- [x] インラインコードのアラインメントが中央寄りになり、テストが問題なくパスする
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 2. 実験機能 テーブル描画変更

- [ ] 2.1 `Tag::Table` 処理時に、現在の `egui::Grid::new` 実装を破棄し、`TableBuilder` への置き換えを実行する。
- [ ] 2.2 カラム間の罫線（`vlines`）を再実装し、文字揃えをサポートする。

### Definition of Done (DoD)

- [ ] テーブルがMarkdown仕様通りに罫線を備えて描画される
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 3. ワークスペースおよびタブUIの改善

- [ ] 3.1 ワークスペースのディレクトリのクリック判定とコンテキストメニュー表示領域を「アイコン＋ディレクトリ名」に広げる（ファイルと同様の挙動）
- [ ] 3.2 ワークスペースのディレクトリのコンテキストメニューからファイル群を開く場合、先頭のファイルがアクティブタブとして表示されるようにする
- [ ] 3.3 タブバーで左右の移動ボタン押下時、アクティブタブが可視領域に入るよう横スクロールを追従させる
- [ ] 3.4 ライトモード時に、エクスポートボタンやワークスペースの履歴ボタンの配色を調整し、視認性を向上させる

### Definition of Done (DoD)

- [ ] ディレクトリアイコンと名前の両方でクリックおよびコンテキストメニューが動作する
- [ ] ディレクトリから複数ファイルを開いた際に先頭ファイルがアクティブになる
- [ ] タブの左右移動時にアクティブタブが画面内にスクロールして追従する
- [ ] ライトモードにおいても、エクスポートや履歴ボタンなどの視認性が十分に確保されている
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. Final Verification & Release Work

- [ ] 4.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 4.2 Ensure `make check` passes with exit code 0
- [ ] 4.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 4.4 Create a PR targeting `master`
- [ ] 4.5 Merge into master (※ `--admin` is permitted)
- [ ] 4.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.6.1`
- [ ] 4.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
