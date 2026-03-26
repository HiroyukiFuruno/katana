## Definition of Ready (DoR)

- [ ] proposal.md, design.md, specs が揃っていること
- [ ] 対象バージョン 0.7.4 のブランチ戦略が確認されていること
- [ ] ComboBox共通化対象の2箇所を実装前に再確認すること

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

---

## 1. 設定画面プルダウンの共通UI統一

### 対象箇所（実装前に再確認・追加洗い出しを行うこと）

| No | ファイル | 行 | ID | 説明 |
|---|---|---|---|---|
| 1 | `settings_window.rs` | L883 | `update_interval` | 更新間隔選択ComboBox |
| 2 | `shell_ui.rs` | L2185 | `terms_lang_select` | 利用規約言語選択ComboBox |

- [ ] 1.1 全ソースを検索し、上記以外の `egui::ComboBox` 使用箇所がないことを確認（洗い出しを完了させる）
- [ ] 1.2 previewセッションで開発した共通UIパターンを参考に `add_styled_combobox` 汎用関数を実装
- [ ] 1.3 `settings_window.rs` L883 の `update_interval` ComboBoxを共通コンポーネントに置き換え
- [ ] 1.4 `shell_ui.rs` L2185 の `terms_lang_select` ComboBoxを共通コンポーネントに置き換え
- [ ] 1.5 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 1.6 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] 全対象箇所のアイコン・文字が上下中央に揃っていることをスナップショットで確認
- [ ] `egui::ComboBox` の直接使用が設定画面から撤廃されている
- [ ] `make check` が exit code 0 で通過
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. システムデフォルト挙動の設定化

- [ ] 2.1 設定画面に追加する対象の洗い出しと確定（実装前に以下の候補から優先度付け）
  - ※セッション復元ON/OFFの**設定UI**はv0.7.3で実装済みであること。本タスクではv0.7.3に実装した設定ロジックを参照する形で統合する（重複実装不可）
  - タブを閉じる際の確認ダイアログ（変更済みファイルのみ）← 本changeで新規実装
  - 新規ファイルのデフォルト拡張子 ← 本changeで新規実装
- [ ] 2.2 `AppBehavior` struct を `katana-platform/src/settings.rs` に追加（全フィールドに `#[serde(default)]`）
- [ ] 2.3 設定画面に「動作」セクションを追加し、各設定のUI（トグル・テキスト入力等）を実装
- [ ] 2.4 各デフォルト挙動をアプリのロジックに接続
- [ ] 2.5 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 2.6 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] 設定画面からシステムデフォルト挙動を変更でき、再起動後も反映されていること
- [ ] 旧設定ファイルでも `#[serde(default)]` でエラーなく起動すること
- [ ] `make check` が exit code 0 で通過
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. カスタムテーマの名前付き保存

- [ ] 3.1 `CustomTheme { name: String, colors: ThemeColors }` struct を定義
- [ ] 3.2 設定スキーマに `custom_themes: Vec<CustomTheme>`（最大10件）を追加
- [ ] 3.3 「名前をつけて保存」UIの実装（現在のカスタム色をテーマとして保存）
- [ ] 3.4 カスタムテーマプリセット一覧に保存済みテーマを表示する実装
- [ ] 3.5 カスタムテーマの削除UI（右クリックメニューまたは削除ボタン）
- [ ] 3.6 適用中のカスタムテーマが削除された場合のデフォルトへのフォールバック処理
- [ ] 3.7 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 3.8 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] カスタムテーマを保存・選択・削除できること
- [ ] アプリ再起動後にカスタムテーマが維持されること
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
- [ ] 4.10 グループは伸縮してすることで複数のタブを表示・非表示できる。
- [ ] 4.11 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 4.12 ユーザーからのフィードバックに基づくUIの微調整および改善実装

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
- [ ] 7.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.7.4`
- [ ] 7.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
