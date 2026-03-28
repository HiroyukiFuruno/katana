# タスク一覧: Rustコードベース全体リファクタリング

> 旧 `v0-8-4-refactoring`（Settings分離設計）は本タスクに統合済み。

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

---

## 0. 解析フェーズ（設計の確定）

> design.md に詳細な現状分析を記載済み。本フェーズでは設計確定に必要な残作業を行う。

- [x] 0.1 4レイヤー（core, linter, platform, ui）の全ソースファイルの行数・責務・SOLID違反の洗い出し
- [x] 0.2 レイヤー間依存関係の分析（`cargo metadata`）
- [x] 0.3 各レイヤーの分割単位の設計（design.md に記載済み）
- [x] 0.4 4レイヤー共通処理の切り出し可否の分析 → **不要**と結論（design.md 参照）
- [ ] 0.5 `emoji.rs`（データ量起因の行数超過）の扱いを確定
- [ ] 0.6 `i18n.rs`（翻訳文字列定義）の外部ファイル化 vs Rustコード分割の方針確定

---

## 1. ast_linter 強化 + clippy統一（全レイヤー共通の品質ガードレール）

> 先にlinterルールを整備し、以降のリファクタリング成果を機械的に検証可能にする。
> `coding-rules.ja.md` に記載されたルールで機械化可能なものをすべてlinterに移行する。

### 1-A. 新規 ast_linter ルール

- [ ] 1.1 `lint_file_length`: ファイル行数制限（200行ハードリミット、テストコード除外）
  - テストモジュール (`#[cfg(test)]`) の行数を計測から除外する
  - エラーメッセージに分離設計のガイダンスを含める
  - 既存の違反ファイルは初回導入時に除外リスト化し、段階的に解消する

- [ ] 1.2 `lint_function_length`: 関数行数制限（30行ハードリミット）
  - `fn` / `impl fn` の本体行数を計測（コメント行・空行含む）
  - テスト関数（`#[cfg(test)]` / `#[test]`）は除外
  - coding-rules §2 の「関数サイズ: 30行を上限」を機械的に強制
  - 既存の違反関数は初回導入時に除外リスト化し、段階的に解消

- [ ] 1.3 `lint_pub_free_fn`: pub free function 禁止
  - coding-rules §1.1 の「ドメインロジックは必ず struct + impl ブロック」を強制
  - `pub fn` / `pub(crate) fn` がモジュールトップレベルに定義されている場合にエラー
  - 除外: `main()`, `#[test]` 関数, `mod tests` 内の関数
  - 既存の違反は除外リスト化し、段階的に解消

- [ ] 1.4 `lint_nesting_depth`: ネスト深度制限（3レベル上限）
  - coding-rules §3 の「ネスト最大3レベル」を強制
  - `if` / `match` / `for` / `while` / `loop` のネスト深度が3を超える場合にエラー
  - テスト関数は除外
  - 注: clippy `cognitive_complexity` は認知的複雑さであり、純粋なネスト深度とは異なる

### 1-B. clippy `#![deny]` 設定の統一

- [ ] 1.5 各クレートの `lib.rs` / `main.rs` に coding-rules §9 の `#![deny]` を追加
  - 現状は `#![deny(warnings)]` のみ → 個別ルールの明示的な `#![deny]` を設定
  - 対象: `clippy::too_many_lines`, `clippy::cognitive_complexity`, `clippy::wildcard_imports`, `clippy::unwrap_used`, `clippy::panic`, `clippy::todo`, `clippy::unimplemented`
  - `#![warn]`: `clippy::expect_used`, `clippy::indexing_slicing`, `clippy::missing_errors_doc`, `missing_docs`

### 1-C. linterルールファイルの分割

- [ ] 1.6 既存の `rust.rs`（969行）の分割
  - 6つの既存Visitor + 4つの新規ルールを個別ファイルに分離（design.md の構造に従う）
  - `rules/rust/mod.rs` で公開APIを集約

### Definition of Done (DoD)
- [ ] 新規linterルール（file_length, function_length, pub_free_fn, nesting_depth）が `make check` で実行される
- [ ] clippy `#![deny]` が全クレートで統一設定済み
- [ ] 既存の6 Visitorが個別ファイルに分離済み
- [ ] linterクレート内の全ファイルが200行以下（テスト除外）
- [ ] Execute `/openspec-delivery` workflow

---

## 2. katana-linter レイヤーのリファクタリング

> linterレイヤーは独立性が高く、他レイヤーへの影響がないため最初に着手する。

- [ ] 2.1 `utils.rs`（406行）の分割
  - `utils/file_collector.rs`: ファイル収集ロジック
  - `utils/parser.rs`: synパースロジック
  - `utils/reporter.rs`: 違反レポートロジック

- [ ] 2.2 `rules/locales.rs`（549行）の分割
  - 責務ごとにサブモジュール化

- [ ] 2.3 `rules/i18n.rs`（391行）の分割
  - 責務ごとにサブモジュール化

- [ ] 2.4 `rules/theme.rs`（313行）の分割

- [ ] 2.5 `rules/markdown.rs`（204行）のボーダーライン確認・必要に応じて分割

### Definition of Done (DoD)
- [ ] linterクレート内の全ファイルが200行以下（テスト除外）
- [ ] 全ファイルの関数が30行以下
- [ ] `make check` がパス
- [ ] Execute `/openspec-delivery` workflow

---

## 3. katana-core レイヤーのリファクタリング

- [ ] 3.1 `update.rs`（646行）の分割
  - `update/version.rs`: バージョン比較ロジック (`is_newer_version`)
  - `update/download.rs`: HTTPダウンロード (`download_update`)
  - `update/installer.rs`: DMG展開・リランチャースクリプト生成
  - `update/mod.rs`: `UpdateManager` 定義 + re-export

- [ ] 3.2 `html/parser.rs`（676行）の分割
  - 正規表現初期化の分離
  - インライン・Markdownパースの分離

- [ ] 3.3 `preview.rs`（472行）の分割
  - `preview/section.rs`: セクション分割ロジック
  - `preview/image.rs`: 画像パス解決
  - `preview/mod.rs`: re-export

- [ ] 3.4 `markdown/mod.rs`（273行）の分割
  - レンダリングロジックの分離
  - フェンスブロック処理の分離

- [ ] 3.5 `markdown/color_preset.rs`（495行）の分割
  - dark/light プリセットの分離

- [ ] 3.6 `markdown/drawio_renderer.rs`（408行）の分割

- [ ] 3.7 `markdown/export.rs`（316行）の分割

- [ ] 3.8 `markdown/mermaid_renderer.rs`（270行）の分割

- [ ] 3.9 `html/node.rs`（339行）の分割
  - `types.rs` + `impls.rs` パターン

### Definition of Done (DoD)
- [ ] coreクレート内の全ファイルが200行以下（テスト除外）
- [ ] 全ファイルの関数が30行以下
- [ ] `make check` がパス
- [ ] Execute `/openspec-delivery` workflow

---

## 4. katana-platform レイヤーのリファクタリング

- [ ] 4.1 `settings.rs`（653行）の完全移行
  - 旧 `settings.rs` の内容を `settings/` サブモジュールに完全移行
  - `pub use` による外部API互換性の維持

- [ ] 4.2 `settings/types.rs`（256行）の分割
  - `types/app.rs`, `types/editor.rs`, `types/window.rs`, `types/behavior.rs` 等

- [ ] 4.3 `theme/builder.rs`（473行）の分割
  - カラービルダー・フォントビルダーの分離

- [ ] 4.4 `theme/types.rs`（241行）の分割

- [ ] 4.5 `theme/migration.rs`（262行）の分割

- [ ] 4.6 `cache.rs`（291行）の分割

- [ ] 4.7 `filesystem.rs`（279行）の分割

- [ ] 4.8 `settings/defaults.rs`（201行）のボーダーライン確認

### Definition of Done (DoD)
- [ ] platformクレート内の全ファイルが200行以下（テスト除外）
- [ ] 全ファイルの関数が30行以下
- [ ] `make check` がパス
- [ ] Execute `/openspec-delivery` workflow

---

## 5. katana-ui レイヤーのリファクタリング（最重要・最大規模）

> UIレイヤーは最も深刻な技術的負債を抱えている。coreとplatformのリファクタリングで手法を確立してから着手する。

### 5-A. God Object (`KatanaApp`) の解体

- [ ] 5.1 `shell.rs`（3,144行）の `KatanaApp` 解体
  - `app/mod.rs`: KatanaApp構造体定義 + `eframe::App` impl
  - `app/workspace.rs`: ワークスペース操作
  - `app/document.rs`: ドキュメント操作
  - `app/export.rs`: エクスポート処理
  - `app/download.rs`: ダウンロード管理
  - `app/update.rs`: 更新チェック・インストール
  - `app/preview.rs`: プレビューキャッシュ管理
  - `app/action.rs`: AppAction処理ディスパッチ

### 5-B. God Object (`AppState`) の解体

- [ ] 5.2 `app_state.rs`（795行）の `AppState` 解体
  - 57フィールドを責務ごとのサブ構造体に分離
  - `state/mod.rs`: AppState定義（サブ構造体を合成）
  - `state/workspace.rs`, `state/editor.rs`, `state/search.rs`, `state/scroll.rs` 等

### 5-C. shell_ui.rs（5,118行）の完全解体

- [ ] 5.3 メニューバー系（`render_menu_bar`, `render_file_menu`, `render_settings_menu`, `render_help_menu`）→ `ui/menu/`
- [ ] 5.4 ヘッダー・ステータスバー → `ui/header.rs`, `ui/status_bar.rs`
- [ ] 5.5 ワークスペースパネル・ファイルツリー → `ui/workspace/`
- [ ] 5.6 タブバー → `ui/tab_bar.rs`
- [ ] 5.7 ビューモード・エディター → `ui/view_mode.rs`, `ui/editor.rs`
- [ ] 5.8 スプリットビュー → `ui/split/`
- [ ] 5.9 検索モーダル・ToCパネル → `ui/search_modal.rs`, `ui/toc_panel.rs`
- [ ] 5.10 各種モーダル → `ui/modals/`（about, meta_info, update, create_node, rename, delete, terms）

### 5-D. その他UIファイルの分割

- [ ] 5.11 `preview_pane.rs`（1,816行）→ `preview/` サブモジュール
- [ ] 5.12 `preview_pane_ui.rs`（1,270行）→ `preview/` に統合
- [ ] 5.13 `settings_window.rs`（1,666行）→ `settings/` タブごとに分割
- [ ] 5.14 `i18n.rs`（1,092行）→ `i18n/` サブモジュール
- [ ] 5.15 `widgets.rs`（948行）→ `widgets/` コンポーネントごとに分割
- [ ] 5.16 `font_loader.rs`（838行）→ 必要に応じて分割
- [ ] 5.17 `svg_loader.rs`（795行）→ `loaders/svg.rs` + 分割
- [ ] 5.18 `http_cache_loader.rs`（786行）→ `loaders/http_cache.rs` + 分割
- [ ] 5.19 `html_renderer.rs`（635行）→ 分割
- [ ] 5.20 `main.rs`（595行）→ `setup/` サブモジュール
- [ ] 5.21 `changelog.rs`（515行）→ 分割
- [ ] 5.22 `about_info.rs`（335行）→ 分割
- [ ] 5.23 `theme_bridge.rs`（313行）→ 分割

### Definition of Done (DoD)
- [ ] uiクレート内の全ファイルが200行以下（テスト除外）
- [ ] 全ファイルの関数が30行以下
- [ ] God Object（KatanaApp, AppState）が責務ごとのサブ構造体・モジュールに分離済み
- [ ] `make check` がパス
- [ ] Execute `/openspec-delivery` workflow

---

## 6. コーディングルール適用・ドキュメント更新

- [ ] 6.1 `docs/coding-rules.ja.md` にファイル行数制限（150行推奨 / 200行ハード）と関数行数制限（30行）を明記
- [ ] 6.2 `coding_rules.md`（エージェントルール）にRust固有のファイルサイズガイドラインを追加
- [ ] 6.3 ast_linterの除外リスト（`emoji.rs` 等のデータファイル）を定義・管理方法を確立

### Definition of Done (DoD)
- [ ] ドキュメントが更新済み
- [ ] Execute `/openspec-delivery` workflow

---

## 7. Final Verification & Release Work

- [ ] 7.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [ ] 7.2 Ensure `make check` passes with exit code 0
- [ ] 7.3 全ファイルが200行以下（テスト除外）であることをast_linterで最終確認
- [ ] 7.4 全関数が30行以下であることをast_linterで最終確認
- [ ] 7.5 Merge the intermediate base branch into the `master` branch
- [ ] 7.6 Create a PR targeting `master`
- [ ] 7.7 Merge into master (※ `--admin` is permitted)
- [ ] 7.8 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md`
- [ ] 7.9 Archive this change by leveraging OpenSpec skills like `/opsx-archive`

---

## 将来的な検討事項（本タスク外）

- ast_linterで4レイヤーのレイヤードアーキテクチャの階層制約（依存方向等）を機械的にチェックする仕組みの追加
  - 階層設計が完了し定着した後に検討する