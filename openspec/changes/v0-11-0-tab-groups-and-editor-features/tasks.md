# Implementation Tasks: タブグルーピング・セッション復元・ピン留め機能改善 (v0.10.0)

## 1. タブグルーピング機能の実装

- [ ] 1.1 `TabGroup { id, name, color, tab_ids }` データモデルを定義
- [ ] 1.2 プリセット8色（赤・橙・黄・緑・青・紫・ピンク・グレー）の定数定義
- [ ] 1.3 コンテキストメニューに「グループに追加」→「新しいグループ」「既存グループ名」を追加
- [ ] 1.4 グループ名インライン編集UIの実装
- [ ] 1.5 グループの色変更UI（プリセット選択 + ColorPicker）の実装
- [ ] 1.6 「グループを解除」の実装（タブは閉じない）
- [ ] 1.7 グループ情報のワークスペース単位永続化（設定ファイルへの読み書き）
- [ ] 1.8 ワークスペース切り替え時のグループ状態切り替え実装
- [ ] 1.9 グループのバージョンフィールドを設定スキーマに追加（後方互換性）
- [ ] 1.10 グループは伸縮してすることで複数のタブを表示・非表示できる。
- [ ] 1.11 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 1.12 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] グループの作成・削除・色変更・タブ追加が動作する
- [ ] アプリ再起動後にグループが復元される
- [ ] `make check` が exit code 0 で通過
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. セッション復元（タブ履歴永続化）の実装

- [ ] 2.1 `SessionState { tabs: Vec<TabEntry>, active_tab: Option<TabId> }` を定義
- [ ] 2.2 アプリ終了時に `last_session.json` をアプリデータディレクトリに書き込む処理を追加
- [ ] 2.3 アプリ起動時に `last_session.json` を読み込み、タブを復元する処理を追加
- [ ] 2.4 ファイル破損時のフォールバック（エラーを無視してデフォルト起動）を実装
- [ ] 2.5 設定画面に「前回のタブを復元する」ON/OFFトグルを追加、デフォルトはON、OFFの場合はtabゼロ起動
- [ ] 2.6 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 2.7 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] 再起動後に前回開いていたタブが復元される
- [ ] 設定でOFF時は復元されない
- [ ] `make check` が exit code 0 で通過
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. ピン留め機能の改善

- [ ] 3.1 ピン留め中のタブの閉じるボタン（×）を非表示にする、ピンを解除しないと閉じれないようにする。ピン留めの状態は、task2のセッション復元機能で復元されるようにする。
- [ ] 3.2 ピン留め中のタブサイズを既存のもの + アイコンの幅とする
- [ ] 3.3 ピン留め中でもホバー時にタイトルをツールチップで表示する
- [ ] 3.4 ショートカットキーでのタブ閉じ操作がピン留めタブに無効になるよう実装
- [ ] 3.5 コンテキストメニューの「ピン留め解除」実装を確認・修正（解除後に×ボタン再表示）
- [ ] 3.6 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 3.7 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] ピン留め中のタブがコンテキストメニューなしで削除できないことを確認
- [ ] ピン留め中にタイトルがツールチップで表示される
- [ ] `make check` が exit code 0 で通過
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. Final Verification & Release Work

- [ ] 4.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 4.2 Ensure `make check` passes with exit code 0
- [ ] 4.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 4.4 Create a PR targeting `master`
- [ ] 4.5 Merge into master (※ `--admin` is permitted)
- [ ] 4.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.10.0`
- [ ] 4.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
