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

- [x] 全対象箇所のアイコン・文字が上下中央に揃っていることをスナップショットで確認
- [x] `egui::ComboBox` の直接使用が設定画面から撤廃されている
- [x] `make check` が exit code 0 で通過
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. システムデフォルト挙動の設定化（Behavior タブ）

設定画面に「Behavior」タブを追加し、以下3つの動作設定を実装する。

- [x] 2.1 `BehaviorSettings` struct を `katana-platform/src/settings.rs` に追加（全フィールドに `#[serde(default)]`）
  - `confirm_close_dirty_tab: bool` (default: true) — 未保存タブの閉じ確認
  - `scroll_sync_enabled: bool` (default: true) — スクロール同期
  - `auto_save: bool` (default: false) — オートセーブ
  - `auto_save_interval_secs: u64` (default: 5) — 自動保存間隔（auto_save=false時は非表示）
- [x] 2.2 `SettingsTab::Behavior` と `render_behavior_tab()` を追加
- [x] 2.3 i18n: `SettingsBehaviorMessages` + 全10言語の locales/*.json に `settings.behavior` 追加
- [x] 2.4 ロジック接続:
  - A1: `CloseDocument` で dirty + 設定ON → 確認ダイアログ → 強制クローズ
  - B1: 設定で永続化 + Split View UIに一時無効トグルボタン
  - E1: メインループにオートセーブタイマー追加（dirty + interval経過 → SaveDocument）
- [x] 2.5 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [x] 2.6 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [x] 設定画面から各挙動を変更でき、再起動後も反映されていること
- [x] 旧設定ファイルでも `#[serde(default)]` でエラーなく起動すること
- [x] `make check` が exit code 0 で通過
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. ワークスペース表示対象拡張子の設定

Workspace設定タブ内に、表示対象ファイルの拡張子設定を追加する。

- [x] 3.1 `WorkspaceSettings` に `visible_extensions: Vec<String>` を追加（default: `["md"]`）
- [x] 3.2 設定画面のWorkspaceタブに拡張子トグル（`.md`, `.markdown`, `.mdx`）を追加
- [x] 3.3 ファイルツリーのフィルタロジックを `visible_extensions` に基づくよう変更
- [x] 3.4 新規ファイル作成ダイアログのデフォルト拡張子をプルダウン（StyledComboBox）で選択可能に
- [x] 3.5 i18n: 関連メッセージを全言語に追加
- [x] 3.6 ユーザーへのUIスナップショット（画像等）の提示および動作報告（※ユーザー指示により未確認進行・事後レビュー）
- [x] 3.7 ユーザーからのフィードバックに基づくUIの微調整および改善実装（※同上）

### Definition of Done (DoD)

- [x] トグルで有効にした拡張子のファイルがワークスペースに表示されること
- [x] 新規作成時にプルダウンから拡張子を選択できること
- [x] `make check` が exit code 0 で通過
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. カスタムテーマの名前付き保存

- [x] 4.1 `CustomTheme { name: String, colors: ThemeColors }` struct を定義
- [x] 4.2 設定スキーマに `custom_themes: Vec<CustomTheme>`（最大10件）を追加
- [x] 4.3 「名前をつけて保存」UIの実装（現在のカスタム色をテーマとして保存）
- [x] 4.4 カスタムテーマプリセット一覧に保存済みテーマを表示する実装
- [x] 4.5 カスタムテーマの削除UI（右クリックメニューまたは削除ボタン）
- [x] 4.6 適用中のカスタムテーマが削除された場合のデフォルトへのフォールバック処理
- [x] 4.7 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [x] 4.8 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [x] カスタムテーマを保存・選択・削除できること
- [x] アプリ再起動後にカスタムテーマが維持されること
- [x] `make check` が exit code 0 で通過
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## [DEFERRED to v0.7.8] 5. タブグルーピング機能の実装

> Note: Per user feedback, Task 5 and all subsequent tasks are deferred to v0.7.8 to prioritize v0.7.7.

- [ ] 5.1 `TabGroup { id, name, color, tab_ids }` データモデルを定義
- [ ] 5.2 プリセット8色（赤・橙・黄・緑・青・紫・ピンク・グレー）の定数定義
- [ ] 5.3 コンテキストメニューに「グループに追加」→「新しいグループ」「既存グループ名」を追加
- [ ] 5.4 グループ名インライン編集UIの実装
- [ ] 5.5 グループの色変更UI（プリセット選択 + ColorPicker）の実装
- [ ] 5.6 「グループを解除」の実装（タブは閉じない）
- [ ] 5.7 グループ情報のワークスペース単位永続化（設定ファイルへの読み書き）
- [ ] 5.8 ワークスペース切り替え時のグループ状態切り替え実装
- [ ] 5.9 グループのバージョンフィールドを設定スキーマに追加（後方互換性）
- [ ] 5.10 グループは伸縮してすることで複数のタブを表示・非表示できる。
- [ ] 5.11 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 5.12 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] グループの作成・削除・色変更・タブ追加が動作する
- [ ] アプリ再起動後にグループが復元される
- [ ] `make check` が exit code 0 で通過
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 6. セッション復元（タブ履歴永続化）の実装

- [ ] 6.1 `SessionState { tabs: Vec<TabEntry>, active_tab: Option<TabId> }` を定義
- [ ] 6.2 アプリ終了時に `last_session.json` をアプリデータディレクトリに書き込む処理を追加
- [ ] 6.3 アプリ起動時に `last_session.json` を読み込み、タブを復元する処理を追加
- [ ] 6.4 ファイル破損時のフォールバック（エラーを無視してデフォルト起動）を実装
- [ ] 6.5 設定画面に「前回のタブを復元する」ON/OFFトグルを追加
- [ ] 6.6 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 6.7 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] 再起動後に前回開いていたタブが復元される
- [ ] 設定でOFF時は復元されない
- [ ] `make check` が exit code 0 で通過
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 7. ピン留め機能の改善

- [ ] 7.1 ピン留め中のタブの閉じるボタン（×）を非表示にする
- [ ] 7.2 ピン留め中でもホバー時にタイトルをツールチップで表示する
- [ ] 7.3 ショートカットキーでのタブ閉じ操作がピン留めタブに無効になるよう実装
- [ ] 7.4 コンテキストメニューの「ピン留め解除」実装を確認・修正（解除後に×ボタン再表示）
- [ ] 7.5 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 7.6 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] ピン留め中のタブがコンテキストメニューなしで削除できないことを確認
- [ ] ピン留め中にタイトルがツールチップで表示される
- [ ] `make check` が exit code 0 で通過
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 8. Final Verification & Release Work

- [ ] 8.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 8.2 Ensure `make check` passes with exit code 0
- [ ] 8.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 8.4 Create a PR targeting `master`
- [ ] 8.5 Merge into master (※ `--admin` is permitted)
- [ ] 8.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.7.6`
- [ ] 8.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
