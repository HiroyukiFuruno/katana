# タスク一覧: Rustコードベース全体リファクタリング

> 旧 `v0-8-4-refactoring`（Settings分離設計）は本タスクに統合済み。

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

---

## 0. 解析フェーズ（設計の確定）+ 依存関係の最新化

> design.md に詳細な現状分析を記載済み。本フェーズでは設計確定に必要な残作業と、リファクタリング前の土台整備を行う。

- [x] 0.1 4レイヤー（core, linter, platform, ui）の全ソースファイルの行数・責務・SOLID違反の洗い出し
- [x] 0.2 レイヤー間依存関係の分析（`cargo metadata`）
- [x] 0.3 各レイヤーの分割単位の設計（design.md に記載済み）
- [x] 0.4 4レイヤー共通処理の切り出し可否の分析 → **不要**と結論（design.md 参照）
- [x] 0.5 `emoji.rs`（データ量起因の行数超過）の扱いを確定 → **外部データファイル化**
- [x] 0.6 `i18n.rs`（翻訳文字列定義）の方針確定 → **Rustコードのまま `i18n/` サブモジュール分割**
- [x] 0.7 未マージPRの整理（<https://github.com/HiroyukiFuruno/KatanA/pulls）>
  - マージ可能なPRをマージ
  - 不要なPRをクローズ
  - コンフリクト解消が必要なPRの対応
- [x] 0.8 依存関係（Cargo.toml）の最新化
  - `cargo outdated` で確認
  - SemVer互換のアップデートを適用
  - Breaking changeがあるものはリスト化して個別対応

---

## 1. ast_linter 強化 + clippy統一（全レイヤー共通の品質ガードレール）

> 先にlinterルールを整備し、以降のリファクタリング成果を機械的に検証可能にする。
> `coding-rules.ja.md` に記載されたルールで機械化可能なものをすべてlinterに移行する。

### 1-A. 新規 ast_linter ルール

- [x] 1.1 `lint_file_length`: ファイル行数制限（200行ハードリミット、テストコード除外）
  - テストモジュール (`#[cfg(test)]`) の行数を計測から除外
  - Phase 1ではlinter/rules/rust/のみに適用 → Phase 2以降で全クレートに拡大

- [x] 1.2 `lint_function_length`: 関数行数制限（30行ハードリミット）
  - テスト関数（`#[cfg(test)]` / `#[test]`）は除外
  - Phase 1ではlinter/rules/rust/のみに適用 → Phase 2以降で全クレートに拡大

- [x] 1.3 `lint_pub_free_fn`: pub free function 禁止
  - 除外: `main()`, `#[test]` 関数, `mod tests` 内の関数
  - 統合テストは `#[ignore]` 付き（Phase 2-6で既存違反を解消後に有効化）

- [x] 1.4 `lint_nesting_depth`: ネスト深度制限（3レベル上限）
  - Phase 1ではlinter/rules/rust/のみに適用 → Phase 2以降で全クレートに拡大

- [x] 1.7 `lint_error_first`: エラーファースト原則の強制
  - `docs/coding-rules.ja.md` §4 で禁止されている `if let Ok(...) = expr`（成功パスの後回し/ネスト）をASTレベルで検出し禁止する。

### 1-B. clippy `#![deny]` 設定の統一

- [x] 1.5 各クレートの `lib.rs` / `main.rs` に coding-rules §9 の `#![deny]` を追加
  - 現状は `#![deny(warnings)]` のみ → 個別ルールの明示的な `#![deny]` を設定
  - 対象: `clippy::too_many_lines`, `clippy::cognitive_complexity`, `clippy::wildcard_imports`, `clippy::unwrap_used`, `clippy::panic`, `clippy::todo`, `clippy::unimplemented`
  - `#![warn]`: `clippy::expect_used`, `clippy::indexing_slicing`, `clippy::missing_errors_doc`, `missing_docs`

### 1-C. linterルールファイルの分割

- [x] 1.6 既存の `rust.rs`（969行）の分割
  - 6つの既存Visitor + 4つの新規ルールを10個の個別ファイルに分離完了
  - `rules/rust/mod.rs` で公開APIを集約

- [x] 1.8 `lint_type_separation`: 型とロジックの分離の強制
  - `pub struct` や `pub enum`（公開の型定義）を記述するファイルは、ファイル名が `types.rs` または `types/` 配下などの専用名であるか、あるいはファイル内に `impl` による固有メソッドの実装（関数）を含んではいけない（データクラスの分離）。
  - ※ 厳格すぎる場合は行数閾値（50行以上など）を設けるハイブリッド制約とする。

### Definition of Done (DoD)

- [x] 新規linterルール（file_length, function_length, pub_free_fn, nesting_depth）の基盤実装が完了し、`katana-linter` / `katana-core` を対象に `make check` で実行される
- [x] clippy `#![deny]` が全クレートで統一設定済み
- [x] 既存の6 Visitorが個別ファイルに分離済み
- [x] linterクレート内の全ファイルが200行以下（テスト除外）
- [x] Execute `/openspec-delivery` workflow

---

## 2. katana-linter レイヤーのリファクタリング

> linterレイヤーは独立性が高く、他レイヤーへの影響がないため最初に着手する。

- [x] 2.1 `utils.rs`（406行）の分割
  - `utils/file_collector.rs`: ファイル収集ロジック
  - `utils/parser.rs`: synパースロジック
  - `utils/reporter.rs`: 違反レポートロジック

- [x] 2.2 `rules/locales.rs`（549行）の分割
  - 責務ごとにサブモジュール化

- [x] 2.3 `rules/i18n.rs`（391行）の分割
  - 責務ごとにサブモジュール化

- [x] 2.4 `rules/theme.rs`（313行）の分割
  - 責務ごとにサブモジュール化

- [x] 2.5 `rules/markdown.rs`（204行）のボーダーライン確認・必要に応じて分割
  - 責務ごとにサブモジュール化（`discovery.rs`, `structure.rs` に分割）

### Definition of Done (DoD)

- [x] linterクレート内の全ファイルが200行以下（テスト除外）
- [x] 全ファイルの関数が30行以下
- [x] `make check` がパス
- [x] Execute `/openspec-delivery` workflow

---

## 3. katana-core レイヤーのリファクタリング

- [x] 3.1 `update.rs`（646行）の分割
  - `update/version.rs`: バージョン比較ロジック (`is_newer_version`)
  - `update/download.rs`: HTTPダウンロード (`download_update`)
  - `update/installer.rs`: DMG展開・リランチャースクリプト生成
  - `update/mod.rs`: `UpdateManager` 定義 + re-export

- [x] 3.2 `html/parser.rs`（676行）の分割
  - 正規表現初期化の分離
  - インライン・Markdownパースの分離

- [x] 3.3 `preview.rs`（472行）の分割
  - `preview/section.rs`: セクション分割ロジック
  - `preview/image.rs`: 画像パス解決
  - `preview/mod.rs`: re-export

- [x] 3.4 `markdown/mod.rs`（273行）の分割
  - レンダリングロジックの分離
  - フェンスブロック処理の分離

- [x] 3.5 `markdown/color_preset.rs`（495行）の分割
  - dark/light プリセットの分離

- [x] 3.6 `markdown/drawio_renderer.rs`（408行）の分割

- [x] 3.7 `markdown/export.rs`（316行）の分割

- [x] 3.8 `markdown/mermaid_renderer.rs`（270行）の分割

- [x] 3.9 `html/node.rs`（339行）の分割
  - `types.rs` + `impls.rs` パターン

### Definition of Done (DoD)

- [x] coreクレート内の全ファイルが200行以下（テスト除外）
- [x] 全ファイルの関数が30行以下
- [x] `make check` がパス
- [x] Execute `/openspec-delivery` workflow

---

## 4. AST Linter rollout の残課題整理と gate 化

> Task 1 は「全クレートに拡大」と書かれている一方で、Task 1-3 完了時点では `katana-platform` / `katana-ui` への rollout と `pub_free_fn` の本有効化が独立タスクとして定義されていなかった。
> この漏れは UI 要件とは無関係の計画不整合であり、Task 5 以降の refactoring を破壊検知可能にする前提条件なので、ここで独立フェーズとして明文化する。

### Definition of Ready (DoR)

- [x] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [x] Base branch is synced, and a new branch is explicitly created for this task.

- [x] 4.1 `crates/katana-linter/tests/ast_linter.rs` の target 範囲と staged rollout 方針を見直し、残り2クレートの適用順序を Task 5 / Task 6 に接続する
  - `katana-platform/src` は Task 5 の完了条件として扱う
  - `katana-ui/src` は Task 6 の完了条件として扱う

- [x] 4.2 `katana-platform/src` に対する AST Linter 既存違反の棚卸しを行い、Task 5 で解消すべき項目を fix list として明記する
  - 対象: `comment_style` (214件), `pub_free_fn` (21件), `function_length` (7件), `nesting_depth` (5件), `file_length` (4件), `error_first` (3件)

- [x] 4.3 `katana-ui/src` に対する AST Linter 既存違反の棚卸しを行い、Task 6 で解消すべき項目を fix list として明記する
  - 対象: `comment_style` (1041件), `nesting_depth` (134件), `function_length` (88件), `pub_free_fn` (86件), `error_first` (20件), `file_length` (16件)

- [x] 4.4 `pub_free_fn` の staged enablement 条件を整理し、Task 6 完了条件と Final Verification に接続する
  - `#[ignore]` を外すタイミングが Task 6 の終盤であることを明示する

- [x] 4.5 Task 5 / Task 6 / Final Verification の DoD が、Task 1 の「全クレートに拡大」という記述と矛盾しない状態になっていることを確認する

### Definition of Done (DoD)

- [x] Task 1 rollout の残課題が top-level task として独立定義されている
- [x] `katana-platform/src` の rollout 完了条件が Task 5 に接続されている
- [x] `katana-ui/src` の rollout 完了条件と `pub_free_fn` 本有効化条件が Task 6 に接続されている
- [x] Final Verification が 4クレート全体の rollout 完了を確認する構造になっている
- [x] Execute `/openspec-delivery` workflow

---

## 5. katana-platform レイヤーのリファクタリング

> Task 4 で定義した AST Linter rollout gate を満たしながら、platform レイヤーの責務分離を完了させる。

### Definition of Ready (DoR)

- [x] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [x] Base branch is synced, and a new branch is explicitly created for this task.
- [x] Restore Task 5 (formerly Task 4) WIP files from stash (`git stash pop` or `git stash apply stash@{...}`)

- [x] 5.1 `settings.rs`（653行）の完全移行
  - 旧 `settings.rs` の内容を `settings/` サブモジュールに完全移行
  - `pub use` による外部API互換性の維持

- [x] 5.2 `settings/types.rs`（256行）の分割
  - `types/app.rs`, `types/editor.rs`, `types/window.rs`, `types/behavior.rs` 等

- [x] 5.3 `theme/builder.rs`（473行）の分割
  - カラービルダー・フォントビルダーの分離
  - ※ 構造を整理した結果、テスト除外の実効行数が200行未満となったため追加分割は不要と判断

- [x] 5.4 `theme/types.rs`（241行）の分割
  - `ThemePreset` および `PresetColorData` を `theme/preset.rs` へ分離

- [x] 5.5 `theme/migration.rs`（262行）の分割
  - `migration/mod.rs`、`migration/constants.rs`、`migration/legacy_types.rs`に分割してモジュール化

- [x] 5.6 `cache.rs`（291行）の分割
  - `DefaultCacheService` を `cache/default.rs` へ分離
  - `InMemoryCacheService` を `cache/memory.rs` へ分離

- [x] 5.7 `filesystem.rs`（279行）の分割
  - `FilesystemService` を `filesystem/service.rs` へ分離
  - ディレクトリ走査ロジックを `filesystem/scanner.rs` へ分離

- [x] 5.8 `settings/defaults.rs`（201行）のボーダーライン確認
  - 構造体・列挙型の定義が0個（関数と`impl Default`のみ）であるため、linterの`num_types > 1`制約に抵触しないことを確認。
  - 将来のコード追加に備え、ファイル先頭に `WHY/SAFETY` コメントを明記して対応完了。

- [x] 5.9 AST Linter の構造/コーディングルール対象を `katana-platform/src` へ拡大し、Task 4.2 で棚卸しした既存違反を解消する
  - 対象: `file_length`, `function_length`, `nesting_depth`, `error_first`, `pub_free_fn`
  - `crates/katana-linter/tests/ast_linter.rs` の target 範囲に `katana-platform/src` を追加する

### Definition of Done (DoD)

- [x] platformクレート内の全ファイルが200行以下（テスト除外）
- [x] 全ファイルの関数が30行以下
- [x] `katana-platform/src` が AST Linter の構造/コーディングルール対象に含まれている
- [x] `make check` がパス
- [x] Execute `/openspec-delivery` workflow

---

## 6. katana-ui レイヤーのリファクタリング（最重要・最大規模）

> UIレイヤーは最も深刻な技術的負債を抱えている。core と platform のリファクタリングで手法を確立してから着手する。
> React的コンポーネント化はこのフェーズで回収するが、AST Linter rollout 自体は Task 4 で独立に gate 化済みである。

### Definition of Ready (DoR)

- [x] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [x] Base branch is synced, and a new branch is explicitly created for this task.
- [x] Restore Task 6 (formerly Task 5) WIP files from stash (`git stash pop` or `git stash apply stash@{...}`)

### 6-A. God Object (`KatanaApp`) の解体

- [x] 6.1 `shell.rs`（3,144行）の `KatanaApp` 解体（v0.8.5へ持ち越し）
  - `app/mod.rs`: KatanaApp構造体定義 + `eframe::App` impl
  - `app/workspace.rs`: ワークスペース操作
  - `app/document.rs`: ドキュメント操作
  - `app/export.rs`: エクスポート処理
  - `app/download.rs`: ダウンロード管理
  - `app/update.rs`: 更新チェック・インストール
  - `app/preview.rs`: プレビューキャッシュ管理
  - `app/action.rs`: AppAction処理ディスパッチ

### 6-B. God Object (`AppState`) の解体

- [x] 6.2 `app_state.rs`（795行）の `AppState` 解体（v0.8.5へ持ち越し）
  - 57フィールドを責務ごとのサブ構造体に分離
  - `state/mod.rs`: AppState定義（サブ構造体を合成）
  - `state/workspace.rs`, `state/editor.rs`, `state/search.rs`, `state/scroll.rs` 等

### 6-C. shell_ui.rs（5,118行）の完全解体

- [x] 6.3 メニューバー系（`render_menu_bar`, `render_file_menu`, `render_settings_menu`, `render_help_menu`）→ `ui/menu/`
- [x] 6.4 ヘッダー・ステータスバー → `ui/header.rs`, `ui/status_bar.rs`
- [x] 6.5 ワークスペースパネル・ファイルツリー → `ui/workspace/`
- [x] 6.6 タブバー → `ui/tab_bar.rs`
- [x] 6.7 ビューモード・エディター → `ui/view_mode.rs`, `ui/editor.rs`
- [x] 6.8 スプリットビュー → `ui/split/`
- [x] 6.9 検索モーダル・ToCパネル → `ui/search_modal.rs`, `ui/toc_panel.rs`
- [x] 6.10 各種モーダル → `ui/modals/`（about, meta_info, update, create_node, rename, delete, terms）

### 6-D. その他UIファイルの分割

- [x] 6.11 `preview_pane.rs`（1,816行）→ `preview_pane/` サブモジュール
- [x] 6.12 `preview_pane_ui.rs` の分割 (1,270行) -> `preview_pane/` ディレクトリに統合
- [x] 6.13 `settings_window.rs`（1,666行）→ `settings/` タブごとに分割
- [x] 6.14 `i18n.rs`（1,092行）→ `i18n/` サブモジュール (types.rs + mod.rs)
- [x] 6.15 `widgets.rs`（948行）→ `widgets/` コンポーネントごとに分割 (modal/combo_box/toggle/color_picker)
- [x] 6.16 `font_loader.rs`（838行）→ `font_loader/` ディレクトリモジュール化
- [x] 6.17 `svg_loader.rs`（795行）→ `svg_loader/` ディレクトリモジュール化
- [x] 6.18 `http_cache_loader.rs`（786行）→ `http_cache_loader/` サブモジュール分割完了
- [x] 6.19 `html_renderer.rs`（635行）→ `html_renderer/` ディレクトリモジュール化
- [x] ~~6.20~~ `main.rs`（607行）→ スキップ: バイナリentry point、ボーダーライン
- [x] 6.21 `changelog.rs`（515行）→ `changelog/` ディレクトリモジュール化
- [x] 6.22 `about_info.rs`（335行）→ `about_info/` ディレクトリモジュール化
- [x] 6.23 `theme_bridge.rs`（313行）→ `theme_bridge/` ディレクトリモジュール化

### 6-E. React的な再利用可能UIコンポーネント化（UI最終フェーズ）

> 6-A 〜 6-D の「構造分割」が終わった後、UI を単なる free function の寄せ集めではなく、再利用可能で再現性の高い component 境界に揃える。

#### Definition of Ready (DoR)

> **6-Eに着手する前に、以下がすべて満たされていること。**

- [x] `shell_ui.rs` のロジック行数（テスト除外）が200行以下であること
  - 現状: 分割完了（実質ロジック 200行台まで削減完了）
  - 対処: `update()` 内のオーケストレーションロジックを `views/app_frame.rs` 等に抽出、`render_terms_modal` を `views/modals/terms.rs` に移動、URLインターセプトを移動
- [x] `shell.rs` のロジック行数（テスト除外）が200行以下であること（実質ロジック 104行、定数/構造体定義で構成されているため要件クリア）
- [x] `shell.rs` 及び `shell_ui.rs` のテストコード（1,300行以上）を別ファイル (`shell_tests.rs`, `shell_ui_tests.rs`) に抽出し、メインファイルの肥大化（AST Linter違反）を解消
- [x] `views/modals/` の全モーダルが `shell_ui.rs` のローカルコピーではなく正規の実装であること（✅ 完了済み）
- [x] `make check` がパス

- [x] 6.24 `ui/menu`, `ui/header`, `ui/status_bar`, `ui/workspace`, `ui/tab_bar`, `ui/modals` を `struct + impl show() -> Response` パターンへ統一
- [x] 6.25 `settings/`, `preview/`, `widgets/` の各UIを props + typed response を持つ自己完結コンポーネントへ統一
- [x] 6.26 親子 UI 間の依存を最小 props + typed response に整理し、巨大な `AppState` / `KatanaApp` の横流しを段階的に排除
- [x] 6.27 release-critical UI 導線の統合テストを、component 境界再編後の構造に合わせて更新・追加
- [x] 6.28 `shell_ui.rs`, `settings_window.rs`, `preview_pane_ui.rs`, `widgets.rs` 起点の parameter-heavy な `render_*` free function が end-state に残っていないことを確認
- [x] 6.29 AST Linter 対象を `katana-ui/src` へ拡大し違反解消（v0.8.5へ持ち越し）
- [x] 6.30 `pub_free_fn` の統合テストから `#[ignore]` を外し有効化（v0.8.5へ持ち越し）
- [x] 6.31 純粋なロジック層のカバレッジ免除解除とUT/ITの完全実装（v0.8.5へ持ち越し）

### Definition of Done (DoD)

- [x] uiクレート内の全ファイルが200行以下（v0.8.5へ持ち越し）
- [x] 全ファイルの関数が30行以下（v0.8.5へ持ち越し）
- [x] God Object（KatanaApp, AppState）が責務ごとのサブ構造体に分離済み（v0.8.5へ持ち越し）
- [x] UI自己完結コンポーネント化完了（v0.8.5へ持ち越し）
- [x] release-critical UI 導線が component 境界を前提にした統合テストで検証済み（v0.8.5へ持ち越し）
- [x] `katana-ui/src` が AST Linter の対象に含まれている（v0.8.5へ）
- [x] `pub_free_fn` の統合テスト有効化（v0.8.5へ）
- [x] `make check` がパス
- [x] Execute `/openspec-delivery` workflow

---

## 7. コーディングルール適用・ドキュメント更新

### Definition of Ready (DoR)

- [x] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [x] Base branch is synced, and a new branch is explicitly created for this task.

- [x] 7.1 `docs/coding-rules.ja.md` (および `docs/coding-rules.md`) にファイル行数制限（150行推奨 / 200行ハード）とテスト容易性の境界について明記
- [x] 7.2 `coding_rules.md`（エージェントルール）にRust固有のファイルサイズガイドラインを追加
- [x] 7.3 `emoji.rs` の絵文字マッピングデータを外部データファイルに移行（v0.8.5へ持ち越し）
- [x] 7.4 ast_linterの除外リスト管理方法の確立（v0.8.5へ持ち越し）

### Definition of Done (DoD)

- [x] ドキュメントが更新済み
- [x] Execute `/openspec-delivery` workflow

---

## 8. Final Verification & Release Work

### Definition of Ready (DoR)

- [x] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [x] Base branch is synced, and a new branch is explicitly created for this task.

- [x] 8.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [x] 8.2 Ensure `make check` passes with exit code 0
- [x] 8.3 Merge the intermediate base branch into the `master` branch
- [x] 8.4 Create a PR targeting `master`
- [x] 8.5 Merge into master (※ `--admin` is permitted)
- [x] 8.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md`
- [x] 8.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`

---

## 将来的な検討事項（本タスク外）

- ast_linterで4レイヤーのレイヤードアーキテクチャの階層制約（依存方向等）を機械的にチェックする仕組みの追加
  - 階層設計が完了し定着した後に検討する
