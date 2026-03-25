## Definition of Ready (DoR)

- [ ] proposal.md, design.md, specs が揃っていること
- [ ] 対象バージョン 0.7.3 のブランチ戦略が確認されていること

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

---

## 1. ハイライト背景バグ調査・修正

- [ ] 1.1 再現条件の調査（特定GPU/描画環境、macOS バージョン、egui のバージョン）。※技術負債メモに記録されている「コードブロック背景色が `syntect` に強制上書きされる問題」と同一事象の可能性があるため、調査時に関連性を確認すること
- [ ] 1.2 egui の `Painter::rect_filled` / `Frame::fill` の描画パスを追跡し、背景が描画されないコードパスを特定
- [ ] 1.3 修正実装（描画パス変更またはフォールバック追加）
- [ ] 1.4 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 1.5 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)
- [ ] 再現していた環境で背景が表示されることを確認
- [ ] `make check` が exit code 0 で通過
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Source Tree ディレクトリアイテムのコンテキストメニュー追加

> **実装対象の明確化**: タブバー上のタブではなく、左ペインのファイルツリー（Source Tree）上のファイル/ディレクトリアイテムが対象。

- [ ] 2.1 Source Tree のファイル/ディレクトリアイテムに右クリックイベントを追加
- [ ] 2.2 Workspace ペインと共通のコンテキストメニューコンポーネントを統合（重複実装を避ける）
- [ ] 2.3 「開く」「コピー」「削除」「名前変更」などの操作をコンテキストメニュー経由で実行できるよう実装
- [ ] 2.4 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 2.5 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)
- [ ] Source Tree のタブ右クリックでメニューが表示される
- [ ] Workspace コンテキストメニューと同等の操作が実行できる
- [ ] `make check` が exit code 0 で通過
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. タブ双方向移動の修正とUI/UX改善

- [ ] 3.1 左→右ドラッグ移動が機能しない原因を特定（配列境界チェック）
- [ ] 3.2 双方向ドラッグ移動のロジックを修正
- [ ] 3.3 移動中のスナップアニメーション（egui `lerp`）を統一実装
- [ ] 3.4 ユーザーへのUIスナップショット（動画等）の提示および動作報告
- [ ] 3.5 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)
- [ ] 左→右・右→左双方向のドラッグ移動が正常動作する
- [ ] `make check` が exit code 0 で通過
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. タブグルーピング機能の実装

- [ ] 4.1 `TabGroup { id, name, color, tab_ids }` データモデルを定義
- [ ] 4.2 プリセット8色（赤・橙・黄・緑・青・紫・ピンク・グレー）の定数定義
- [ ] 4.3 コンテキストメニューに「グループに追加」→「新しいグループ」「既存グループ名」を追加
- [ ] 4.4 グループ名インライン編集UIの実装
- [ ] 4.5 グループの色変更UI（プリセット選択 + ColorPicker）の実装
- [ ] 4.6 「グループを解除」の実装（タブは閉じない）
- [ ] 4.7 グループ情報のワークスペース単位永続化（設定ファイルへの読み書き）
- [ ] 4.8 ワークスペース切り替え時のグループ状態切り替え実装
- [ ] 4.9 グループのバージョンフィールドを設定スキーマに追加（後方互換性）
- [ ] 4.10 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 4.11 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)
- [ ] グループの作成・削除・色変更・タブ追加が動作する
- [ ] アプリ再起動後にグループが復元される
- [ ] `make check` が exit code 0 で通過
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 5. セッション復元（タブ履歴永続化）の実装

- [ ] 5.1 `SessionState { tabs: Vec<TabEntry>, active_tab: Option<TabId> }` を定義
- [ ] 5.2 アプリ終了時に `last_session.json` をアプリデータディレクトリに書き込む処理を追加
- [ ] 5.3 アプリ起動時に `last_session.json` を読み込み、タブを復元する処理を追加
- [ ] 5.4 ファイル破損時のフォールバック（エラーを無視してデフォルト起動）を実装
- [ ] 5.5 設定画面に「前回のタブを復元する」ON/OFFトグルを追加
- [ ] 5.6 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 5.7 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)
- [ ] 再起動後に前回開いていたタブが復元される
- [ ] 設定でOFF時は復元されない
- [ ] `make check` が exit code 0 で通過
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 6. ピン留め機能の改善

- [ ] 6.1 ピン留め中のタブの閉じるボタン（×）を非表示にする
- [ ] 6.2 ピン留め中でもホバー時にタイトルをツールチップで表示する
- [ ] 6.3 ショートカットキーでのタブ閉じ操作がピン留めタブに無効になるよう実装
- [ ] 6.4 コンテキストメニューの「ピン留め解除」実装を確認・修正（解除後に×ボタン再表示）
- [ ] 6.5 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 6.6 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)
- [ ] ピン留め中のタブがコンテキストメニューなしで削除できないことを確認
- [ ] ピン留め中にタイトルがツールチップで表示される
- [ ] `make check` が exit code 0 で通過
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 7. Final Verification & Release Work

- [ ] 7.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 7.2 Ensure `make check` passes with exit code 0
- [ ] 7.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 7.4 Create a PR targeting `master`
- [ ] 7.5 Merge into master (※ `--admin` is permitted)
- [ ] 7.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.7.3`
- [ ] 7.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
