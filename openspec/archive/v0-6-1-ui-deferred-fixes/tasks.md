## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

## 1. P3 インラインコード配置最適化

- [x] 1.1 `pulldown.rs` 内の `self.line.append_to` に使用される `egui::Align` を `BOTTOM` から `Center` へ変更。
- [x] 1.2 `katana-ui/tests/preview_pane.rs` において、`html_...` となっている旧テストケースを `Center` 対応へと修正する。
- [x] 1.3 インラインコードと取り消し線のvalignをALIGN::TOPに変更し、~5pxの上方向シフトを実現
- [/] 1.4 【再指摘 p2】改善していない、取り消し線の位置が下寄り、「インラインコード」のbgが下寄り

### Definition of Done (DoD)

- [x] インラインコードのアラインメントが中央寄りになり、テストが問題なくパスする
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 2. 実験機能 テーブル描画変更

- [x] 2.1 `Tag::Table` 処理時にテーブル描画を改修: `egui::Grid` + `min_col_width` でカラム幅均等分配、`parse_row` バグ修正。
- [x] 2.2 カラム間の罫線（`vlines`）を再実装し、文字揃えをサポートする。
- [x] 2.3 【再指摘 p0】テーブルレイアウト崩壊。原則横幅いっぱい（左右のmargin 5px）に表示して、カラム数で均等割、ヘッダー、カラムごとにpadding 2.5px（※縦罫線の描画、行ごとのbg変化は完了済）

### Definition of Done (DoD)

- [x] テーブルがMarkdown仕様通りに罫線を備えて描画され、横幅分配・パディングが要件を満たすこと
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 3. ワークスペースおよびタブUIの改善

- [x] 3.1 ワークスペースのディレクトリのクリック判定とコンテキストメニュー表示領域を「アイコン＋ディレクトリ名」に広げる（確認済：egui::Sense::click()で行全体がクリック可能）
- [x] 3.2 ワークスペースのディレクトリのコンテキストメニューからファイル群を開く場合、先頭のファイルがアクティブタブとして表示されるようにする
- [x] 3.3 タブバーで左右の移動ボタン押下時、アクティブタブが可視領域に入るよう横スクロールを追従させる
- [x] 3.4 ライトモード時に、エクスポートボタンやワークスペースの履歴ボタンの配色を調整し、視認性を向上させる
- [x] 3.5 ワークスペースのディレクトリから複数ファイルを開く処理: 先頭ファイルのみアクティブ化、他はlazy loadで対応
- [/] 3.6 【再指摘 p1】改善されていない、文字部分をクリックできない、ホバーしてもアクションが発生しない（ディレクトリ and ファイル名）アイコン+名称を囲ったエレメント（行全体）にホバーをかける。メタ情報のtooltipを廃止して、コンテキストメニュ -> メタ情報表示でファイルやディレクトリの詳細を表示する
- [x] 3.7 【再指摘 p1】ユーザー操作で横スクロールする場合も発生する必要はない、左右のボタン押下時のみにスクロール追従するように変更すること
- [x] 3.8 【再指摘 p1】workspaceのfilterや目次表示などlightモードの全てのアイコンはbgをグレーにする指定が漏れている
- [x] 3.9 【再指摘 p1】非同期かつ並列的なアプローチに本当になっているか？ユーザー想定（開く順にタブ増加）と異なり一斉に増えているように見えるため検証・改善
- [x] 3.10 プレビューウィンドウとメインウィンドウの外枠の謎のpadding or marginを削除
- [x] 3.11 プレビューウィンドウとメインウィンドウの内枠の右の余白を左と揃える。
- [x] 3.12 tableのヘッダーとrowとの間のboaderの表示がおかしいため正しく修正してください。途中で途切れるような形で引かれています。
- [ ] 3.13 tableのaline中央寄せ（commitしてから試行錯誤）

### Definition of Done (DoD)

- [x] ディレクトリアイコンと名前の両方でクリックおよびコンテキストメニューが完全に動作しホバーアクションも機能する
- [x] ディレクトリから複数ファイルを開いた際に先頭ファイルがアクティブになる
- [x] タブの左右移動ボタン押下時にのみアクティブタブが画面内にスクロールして追従する
- [x] ライトモードにおいて全ての指定されたアイコンのbgがグレーになり視認性が確保されている
- [x] マルチタブオープンが真の並列/非同期処理として UX 上も正しく反映される
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
