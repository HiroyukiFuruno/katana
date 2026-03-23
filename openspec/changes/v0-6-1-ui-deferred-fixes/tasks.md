## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

## 1. P3 インラインコード配置最適化

- [x] 1.1 `pulldown.rs` 内の `self.line.append_to` に使用される `egui::Align` を `BOTTOM` から `Center` へ変更。
- [x] 1.2 `katana-ui/tests/preview_pane.rs` において、`html_...` となっている旧テストケースを `Center` 対応へと修正する。
- [ ] 1.3 解消されていませんでした...image.pngを参照してください。インラインコードの文字が5px亭午上に寄せる必要があります。取り消し線も同様ですね。

### Definition of Done (DoD)

- [x] インラインコードのアラインメントが中央寄りになり、テストが問題なくパスする
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 2. 実験機能 テーブル描画変更

- [x] 2.1 `Tag::Table` 処理時にテーブル描画を改修: `egui::Grid` + `min_col_width` でカラム幅均等分配、`parse_row` バグ修正。
- [x] 2.2 カラム間の罫線（`vlines`）を再実装し、文字揃えをサポートする。

### Definition of Done (DoD)

- [x] テーブルがMarkdown仕様通りに罫線を備えて描画される
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 3. ワークスペースおよびタブUIの改善

- [x] 3.1 ワークスペースのディレクトリのクリック判定とコンテキストメニュー表示領域を「アイコン＋ディレクトリ名」に広げる（確認済：egui::Sense::click()で行全体がクリック可能）
- [x] 3.2 ワークスペースのディレクトリのコンテキストメニューからファイル群を開く場合、先頭のファイルがアクティブタブとして表示されるようにする
- [x] 3.3 タブバーで左右の移動ボタン押下時、アクティブタブが可視領域に入るよう横スクロールを追従させる
- [x] 3.4 ライトモード時に、エクスポートボタンやワークスペースの履歴ボタンの配色を調整し、視認性を向上させる
- [x] 3.5 ワークスペースのディレクトリから複数ファイルを開く処理: 先頭ファイルのみアクティブ化、他はlazy loadで対応

### Definition of Done (DoD)

- [x] ディレクトリアイコンと名前の両方でクリックおよびコンテキストメニューが動作する
- [x] ディレクトリから複数ファイルを開いた際に先頭ファイルがアクティブになる
- [x] タブの左右移動時にアクティブタブが画面内にスクロールして追従する
- [x] ライトモードにおいても、エクスポートや履歴ボタンなどの視認性が十分に確保されている
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
