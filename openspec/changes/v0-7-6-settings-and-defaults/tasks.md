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

## 5. Final Verification & Release Work

- [x] 5.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [x] 5.2 Ensure `make check` passes with exit code 0
- [ ] 5.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 5.4 Create a PR targeting `master`
- [ ] 5.5 Merge into master (※ `--admin` is permitted)
- [ ] 5.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.7.6`
- [ ] 5.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
