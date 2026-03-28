# 設計: Rustコードベース全体リファクタリング

## Context

KatanAの4レイヤー（core, linter, platform, ui）の保守性と拡張性を根本的に高めるためのリファクタリング設計。肥大化ファイルの分離、SOLID原則違反の解消、レイヤードアーキテクチャの再設計を含む。

## Goals / Non-Goals

**Goals:**

- 各レイヤー内部の階層設計とサブモジュール分割
- 肥大化ファイルの責務分離（150行推奨上限 / 200行ハードリミット、テストコード除外）
- SOLID原則違反の解消（God Object の解体、責務の明確化）
- ast_linterによる200行制限の機械的な強制
- 4レイヤー共通処理の切り出し可否の分析
- コーディングルール（`coding_rules.md`）の完全適用

**Non-Goals:**

- 機能追加や動作変更
- 外部APIの破壊的変更
- パフォーマンス最適化（ただし劣化は禁止）
- テストカバレッジの拡大（既存テストの維持のみ）

## 横断的設計原則: データ/型/ロジックの分離

> JavaのDTO/Entity、TypeScriptの `@types/*.d.ts` に相当する概念。
> ロジックファイルからデータ定義と型定義を分離することで、ロジックの可読性と保守性が飛躍的に向上する。

### 原則

| 分類 | 配置 | 例 |
| --- | --- | --- |
| **Data Objects (DTO/Entity)** | `types.rs` or `models/` | 構造体定義、enum定義、データモデル |
| **Type Aliases / Newtypes** | `types.rs` | type aliases, newtype wrappers |
| **Domain Logic** | `*.rs` (struct + impl) | ビジネスロジック、変換処理 |
| **Re-export** | `mod.rs` | 公開API集約 |

### Rustでの適用パターン

```rust
// ❌ 現在: 1ファイルに型もロジックも混在
// violation.rs (または lib.rs)
pub struct Violation { pub file: PathBuf, pub line: usize, ... }
impl Violation { fn format(&self) -> String { ... } }
pub fn run_lint(path: &Path) -> Vec<Violation> { ... }

// ✅ 改善後: 型はtypes.rs、ロジックは別ファイル
// types.rs — DTOに集約
pub struct Violation { pub file: PathBuf, pub line: usize, ... }
pub enum JsonNodeKind { Object, Array(usize), String, ... }

// impls.rs — 型のメソッド実装
impl fmt::Display for Violation { ... }
impl JsonNodeKind { pub fn from_value(v: &Value) -> Self { ... } }

// runner.rs — ドメインロジック
pub fn run_ast_lint(...) -> Vec<Violation> { ... }
```

### レイヤー別適用

- **katana-linter**: `lib.rs` の `Violation`, `JsonNodeKind` → `types.rs` に移動
- **katana-core**: `document.rs` のモデル型 → 現状OK（型中心ファイル）
- **katana-platform**: `settings/types.rs` → `settings/types/app.rs`, `editor.rs` 等に分割
- **katana-ui**: `app_state.rs` の57フィールド → `state/types.rs` + サブ構造体に分割

## ファイルサイズのガイドライン

| 基準 | 行数（テスト除外） | 扱い |
| --- | --- | --- |
| 推奨上限 | 150行 | 設計時の目安。これを超える場合は分割を検討 |
| ハードリミット | 200行 | ast_linterでエラー。分離設計を促すメッセージを出力 |

## 現状分析: レイヤー間依存関係

```text
katana-core    → (外部クレートのみ)
katana-linter  → (独立。katana-coreに依存しない)
katana-platform → katana-core
katana-ui       → katana-core, katana-platform
```

---

## レイヤー別 階層設計

### Layer 1: katana-core（ドメインロジック層）

**現在の構造と問題:**

```text
katana-core/src/
  lib.rs           (モジュール定義のみ - OK)
  ai/mod.rs        (AI機能 - OK)
  document.rs      (ドキュメントモデル - OK)
  emoji.rs         (399行 - 絵文字データ定義。データ量起因で例外検討)
  html/
    mod.rs         (OK)
    node.rs        (339行 - HTMLノード定義)
    parser.rs      (676行 - SRP違反: HTMLパーサーに正規表現初期化・属性抽出・複数パース戦略が混在)
  markdown/
    mod.rs         (273行 - SRP違反: レンダリングロジック＋ダイアグラム処理＋フェンスブロック処理が同居)
    color_preset.rs(495行 - データ量起因だが分割検討)
    diagram.rs     (OK)
    drawio_renderer.rs  (408行 - 要分割)
    export.rs      (316行 - 要分割)
    mermaid_renderer.rs (270行 - 要分割)
    outline.rs     (OK)
    plantuml_renderer.rs(208行 - ボーダーライン)
    svg_rasterize.rs    (OK)
    test_comrak.rs (OK)
  plugin/mod.rs    (OK)
  preview.rs       (472行 - SRP違反: セクション分割＋画像パス解決＋数式処理が同居)
  update.rs        (646行 - SRP違反: バージョン比較・HTTPダウンロード・DMG展開・リランチャースクリプト生成が1ファイル)
  workspace.rs     (OK)
```

**提案するサブモジュール構造:**

```text
katana-core/src/
  html/
    node.rs         → html/node/types.rs + html/node/impls.rs (339行分割)
    parser.rs       → html/parser/mod.rs + html/parser/regex.rs + html/parser/inline.rs
  markdown/
    mod.rs          → markdown/mod.rs(re-exportのみ) + markdown/renderer.rs + markdown/fence.rs
    color_preset.rs → markdown/color_preset/dark.rs + light.rs
    drawio_renderer.rs → 内部メソッド分割で200行以下を目指す
    export.rs       → markdown/export/html.rs + markdown/export/pdf.rs
  preview/
    mod.rs          (re-export)
    section.rs      (セクション分割ロジック)
    image.rs        (画像パス解決)
    math.rs         (数式処理)
  update/
    mod.rs          (re-export)
    version.rs      (バージョン比較)
    download.rs     (HTTPダウンロード)
    installer.rs    (DMG展開・リランチャー)
```

---

### Layer 2: katana-linter（静的解析層）

**現在の構造と問題:**

```text
katana-linter/src/
  lib.rs           (99行 - OK)
  utils.rs         (406行 - SRP違反: ファイル収集・パース・違反レポートが混在)
  rules/
    mod.rs         (モジュール定義のみ - OK)
    rust.rs        (969行 - SRP違反: 6個の独立したVisitorが1ファイルに全展開)
    locales.rs     (549行 - 要分割)
    i18n.rs        (391行 - 要分割)
    theme.rs       (313行 - 要分割)
    changelog.rs   (OK)
    markdown.rs    (204行 - ボーダーライン)
```

**提案するサブモジュール構造:**

```text
katana-linter/src/
  utils/
    mod.rs           (re-export)
    file_collector.rs(ファイル収集)
    parser.rs        (synパース)
    reporter.rs      (違反レポート)
  rules/
    rust/
      mod.rs              (公開API。全ルールの集約)
      magic_numbers.rs    (MagicNumberVisitor)
      prohibited_types.rs (ProhibitedTypesVisitor)
      lazy_code.rs        (LazyCodeVisitor)
      font_normalization.rs(FontNormalizationVisitor)
      performance.rs      (PerformanceVisitor)
      prohibited_attrs.rs (ProhibitedAttributeVisitor)
      file_length.rs      (★新規: 200行制限チェッカー)
    locales/
      mod.rs + 責務ごとに分割
    i18n/
      mod.rs + 責務ごとに分割
```

---

### Layer 3: katana-platform（プラットフォーム抽象層）

**現在の構造と問題:**

```text
katana-platform/src/
  lib.rs           (モジュール定義のみ - OK)
  cache.rs         (291行 - 要分割)
  filesystem.rs    (279行 - 要分割)
  os_fonts.rs      (OK)
  os_theme.rs      (OK)
  settings.rs      (653行 - 分離開始済みだが旧ファイルが残存)
  settings/
    types.rs       (256行 - 要分割)
    impls.rs       (OK)
    defaults.rs    (201行 - ボーダーライン)
    service.rs     (OK)
    repository.rs  (OK)
    migration/     (OK)
  theme/
    builder.rs     (473行 - 要分割)
    impls.rs       (OK)
    migration.rs   (262行 - 要分割)
    mod.rs         (OK)
    types.rs       (241行 - 要分割)
    presets/       (OK - 個別プリセットファイル)
```

**提案するサブモジュール構造:**

```text
katana-platform/src/
  cache/
    mod.rs         (re-export)
    memory.rs      (インメモリキャッシュ)
    disk.rs        (ディスクキャッシュ)
  filesystem/
    mod.rs + 責務ごとに分割
  settings/
    mod.rs         (re-export。旧settings.rsの完全移行)
    types/
      mod.rs + app.rs + editor.rs + window.rs + behavior.rs
    ...（既存の分離構造を維持）
  theme/
    builder/
      mod.rs + color_builder.rs + font_builder.rs
    types/
      mod.rs + colors.rs + fonts.rs
    migration/
      mod.rs + 責務ごとに分割
```

---

### Layer 4: katana-ui（プレゼンテーション層） — 最重要

**現在の構造と問題（深刻）:**

```text
katana-ui/src/
  main.rs           (595行 - SRP違反: フォントセットアップ・プラグイン登録・ロケール検出が1ファイル)
  shell.rs          (3,144行 - God Object: KatanaAppに40+メソッド)
  shell_ui.rs       (5,118行 - 46個のfree関数が1ファイルに集中)
  shell_logic.rs    (404行 - ロジックの一部は分離済みだが不十分)
  app_state.rs      (795行 - AppStateに57フィールド。God Object)
  preview_pane.rs   (1,816行 - SRP違反: レンダリング＋キャッシュ＋フルスクリーン)
  preview_pane_ui.rs(1,270行 - SRP違反: セクション描画＋モーダル＋画像表示)
  settings_window.rs(1,666行 - SRP違反: 全設定タブが同居)
  i18n.rs           (1,092行 - 翻訳文字列定義とロジック混在)
  widgets.rs        (948行 - 4種のウィジェットが1ファイル)
  font_loader.rs    (838行 - OK方向だが分割余地あり)
  svg_loader.rs     (795行 - 要分割)
  http_cache_loader.rs(786行 - 要分割)
  html_renderer.rs  (635行 - 要分割)
  changelog.rs      (515行 - 要分割)
  about_info.rs     (335行 - 要分割)
  theme_bridge.rs   (313行 - 要分割)
  icon.rs           (220行 - ボーダーライン)
  diagram_controller.rs(OK)
  lib.rs            (OK)
```

**UIコンポーネントパターン（React的設計）:**

現在の `shell_ui.rs` は46個のfree functionの集合であり、状態を外部から渡す手続き型スタイル。
これをReact的な **自己完結コンポーネント** に変換する。

```rust
// ❌ 現在のパターン: free function（手続き型）
pub(crate) fn render_status_bar(
    ui: &mut egui::Ui,
    state: &AppState,
    workspace: &Option<WorkspaceInfo>,
    encoding: &str,
    line_count: usize,
    // ... 引数がどんどん増える
) { ... }

// ✅ 新パターン: React的コンポーネント（struct + impl）
pub(crate) struct StatusBar<'a> {
    workspace: Option<&'a WorkspaceInfo>,
    encoding: &'a str,
    line_count: usize,
}

impl<'a> StatusBar<'a> {
    pub fn new(workspace: Option<&'a WorkspaceInfo>, encoding: &'a str, line_count: usize) -> Self {
        Self { workspace, encoding, line_count }
    }

    /// Renders the component and returns any user actions.
    pub fn show(&self, ui: &mut egui::Ui) -> StatusBarResponse {
        // ... 描画ロジック
    }
}

/// Component output (like React's event callbacks)
pub(crate) enum StatusBarResponse {
    None,
    EncodingClicked,
}
```

**コンポーネントパターンの原則:**

| React概念 | egui/Rustでの対応 |
| --- | --- |
| Props | コンポーネント `struct` のフィールド（不変参照 `&'a T`） |
| Render | `show(&self, ui: &mut egui::Ui) -> Response` メソッド |
| Event callbacks | 戻り値の `Response` enum（ユーザーアクションを表現） |
| Children | `show` メソッド内で子コンポーネントを構成 |
| State (useState) | `&mut` で渡される外部状態 or egui の `ctx.data()` |

**適用対象:**

- `shell_ui.rs` の46個のfree function → 各コンポーネント `struct`
- `settings_window.rs` の各タブ → タブコンポーネント `struct`
- `widgets.rs` の4ウィジェット → 個別ウィジェットコンポーネント
- `preview_pane_ui.rs` のセクション描画 → セクションコンポーネント

**提案するサブモジュール構造:**

```text
katana-ui/src/
  main.rs            → main.rs(最小限) + setup/fonts.rs + setup/plugins.rs + setup/locale.rs

  # KatanaApp (God Object) の解体
  app/
    mod.rs           (KatanaApp定義 + eframe::App impl)
    workspace.rs     (ワークスペース操作: open/close/refresh)
    document.rs      (ドキュメント操作: select/save/update)
    export.rs        (HTML/PDF/PNGエクスポート)
    download.rs      (外部ツールダウンロード)
    update.rs        (アプリ更新チェック・インストール)
    preview.rs       (プレビューキャッシュ管理)
    action.rs        (AppAction処理ディスパッチ)

  # AppState (God Object) の解体
  state/
    mod.rs           (AppState定義 + 基本メソッド)
    workspace.rs     (ワークスペース関連フィールド)
    editor.rs        (エディタ関連フィールド)
    search.rs        (検索関連フィールド)
    settings_ui.rs   (設定UI関連フィールド)
    scroll.rs        (スクロール同期フィールド)

  # shell_ui.rs (5,118行) の完全解体
  ui/
    menu/
      mod.rs         (メニューバー全体)
      file_menu.rs
      settings_menu.rs
      help_menu.rs
    header.rs        (ヘッダー右側)
    status_bar.rs    (ステータスバー)
    workspace/
      mod.rs         (ワークスペースパネル)
      tree.rs        (ファイルツリー)
      context_menu.rs
      breadcrumb.rs
    tab_bar.rs       (タブバー)
    view_mode.rs     (ビューモード切り替え)
    editor.rs        (エディターコンテンツ)
    split/
      mod.rs         (スプリットビュー)
      horizontal.rs
      vertical.rs
    search_modal.rs  (検索モーダル)
    toc_panel.rs     (ToC パネル)
    modals/
      about.rs       (About ウィンドウ)
      meta_info.rs   (メタ情報)
      update.rs      (アップデートウィンドウ)
      create_node.rs (ファイル/フォルダ作成)
      rename.rs      (リネームモーダル)
      delete.rs      (削除モーダル)
      terms.rs       (利用規約モーダル)

  # preview_pane の分割
  preview/
    mod.rs           (PreviewPane定義)
    render.rs        (セクションレンダリング)
    diagram.rs       (ダイアグラム管理)
    image_cache.rs   (画像キャッシュ)
    fullscreen.rs    (フルスクリーン表示)
    viewer.rs        (ViewerState + ズーム/パン制御)

  # settings_window の分割
  settings/
    mod.rs           (設定ウィンドウフレーム)
    theme_tab.rs     (テーマ設定タブ)
    font_tab.rs      (フォント設定タブ)
    layout_tab.rs    (レイアウト設定タブ)
    workspace_tab.rs (ワークスペース設定タブ)
    updates_tab.rs   (更新設定タブ)
    behavior_tab.rs  (動作設定タブ)

  # widgets の分割
  widgets/
    mod.rs           (re-export)
    modal.rs         (Modal)
    combobox.rs      (StyledComboBox)
    toggle.rs        (LabeledToggle + toggle_switch)
    color_picker.rs  (LabeledColorPicker)

  # i18n の分割
  i18n/
    mod.rs           (ロジック + tf関数)
    strings/
      en.rs          (英語文字列)
      ja.rs          (日本語文字列)

  # その他の分割
  loaders/
    font.rs          (NormalizeFonts + SystemFontLoader)
    svg.rs           (KatanaSvgLoader)
    http_cache.rs    (PersistentHttpLoader)
  html_renderer/
    mod.rs + 責務ごとに分割
  changelog/
    mod.rs + 責務ごとに分割
```

---

## 4レイヤー共通処理の分析

### 共通化の候補

| パターン | 出現箇所 | 共通化の判断 |
| --- | --- | --- |
| エラー型定義パターン | core, platform, ui | **検討**: 共通のエラーtrait導入は有効だが、レイヤー固有のエラー型は維持すべき |
| ファイルI/Oパターン | platform, core | **不要**: platformが抽象化すべき責務。coreへの共通化は依存方向違反 |
| Visitor パターン (linter) | linter内のみ | **linter内共通化**: 共通trait `LintVisitor` の抽出は有効 |
| serde共通パターン | platform, core | **不要**: 各レイヤーが独立して管理すべき |

### 結論

4レイヤー横断の共通クレート（`katana-common` 等）は**不要**。理由：

- 現在の依存方向（core ← platform ← ui, linterは独立）は健全
- 共通クレートを作ると依存の方向が曖昧になるリスク
- 各レイヤー内部の共通化で十分

---

## ast_linter: 200行制限ルールの設計

### 新規ルール: `lint_file_length`

```rust
// crates/katana-linter/src/rules/rust/file_length.rs
pub fn lint_file_length(path: &Path, syntax: &syn::File) -> Vec<Violation> {
    // テストモジュール (#[cfg(test)] mod tests) の行数を除外して計測
    // 200行超でエラー
}
```

**エラーメッセージ:**

```text
File exceeds 200 lines (excluding tests): {actual} lines.
Consider splitting by responsibility:
  - Extract types to `types.rs`
  - Extract implementations to `impls.rs`
  - Group related functions into submodules
See: docs/coding-rules.ja.md §責務分離の原則
```

### 除外対象

- テストコード（`#[cfg(test)]` モジュール内の行数）
- 純粋なデータ定義ファイル（`emoji.rs` 等）→ プロジェクト単位の除外リストで管理

---

## Risks / Trade-offs

**大規模なモジュールパス変更** → `pub use` による re-export で外部APIの互換性を維持。段階的移行。

**リファクタリング中のデグレード** → レイヤーごとに作業し、各フェーズで `make check` を通す。

**作業量の大きさ** → 最も深刻なuiレイヤーを最後に回し、coreやlinterで手法を確立してからuiに着手する。

## Resolved Questions

- ✅ `emoji.rs`（399行）→ **外部データファイル化**（Phase 6で実施）
- ✅ `i18n.rs`の翻訳文字列定義 → **Rustコードのまま `i18n/` サブモジュール分割**（Phase 5で実施）
- ✅ 型/データオブジェクトの分離 → **横断的設計原則**として各Phaseで適用
