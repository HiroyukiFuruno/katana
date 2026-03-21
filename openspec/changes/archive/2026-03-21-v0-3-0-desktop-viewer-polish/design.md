## Context

Katana Desktop は Rust + egui で構築されたMarkdownエディタ / Viewer。現在のアーキテクチャは3クレート構成:

- **katana-core**: ドメインロジック（Workspace, Document, Markdown変換, Preview）
- **katana-ui**: egui ベースのUI（shell, preview_pane, app_state, i18n）
- **katana-platform**: OSレベルサービス（ファイルシステム, 設定）

設定は現在インメモリのみ（`SettingsService`）。ワークスペースは単一ディレクトリのみ対応。プレビューはcomrak経由のHTML→eguiレンダリング。テーマ設定・永続化・エクスポート機能は未実装。

## Goals / Non-Goals

**Goals:**
- Viewer として「見やすい・使いやすい」を実現する（v0.1.0 のリリース品質）
- 設定とワークスペースの永続化を実装し、再起動時に状態を復元する
- テーマ（Dark/Light）とフォント設定でユーザー好みにカスタマイズ可能にする
- Markdown をPDF/画像/HTMLでエクスポートできるようにする
- アプリとしてのブランディング（アイコン、スプラッシュ）を整える

**Non-Goals:**
- 本格的なエディタ機能（LSP連携、autocomplete、Git統合等）は今回スコープ外
- クラウド同期の実装（アーキテクチャ準備のみ）
- プラグインシステムの拡充
- Windows / Linux 固有のUI最適化（macOS優先）

## Decisions

### 1. 永続化フォーマット: JSON

**決定**: 設定ファイルとワークスペース情報はJSONで保存する。

**理由**: `serde_json`が既にworkspace依存にあり追加コスト0。TOMLも候補だったが、設定UIから動的に読み書きする用途にはJSONの方がシンプル。将来的にCloud Storage（DB等）に切り替える想定で、Repositoryパターン（DIP）で抽象化する。

**代替案**: TOML（人間が直接編集しやすいが、動的書き込みのecosystemが弱い）

### 2. Repositoryパターンによる永続化抽象

**決定**: `SettingsRepository` trait を導入し、`JsonFileRepository`をデフォルト実装とする。

```
trait SettingsRepository {
    fn load(&self) -> Result<AppSettings>;
    fn save(&self, settings: &AppSettings) -> Result<()>;
}
```

**理由**: ユーザーの要望通り、Cloud Storage（DBなど）への切り替えをDIPで素早く行えるようにする。初期実装は`~/Library/Application Support/katana/config.json`（`dirs`クレートで解決）への読み書き。

### 3. シンタックスハイライト: syntect

**決定**: `syntect`クレートを採用。

**理由**: Rust製ハイライトライブラリの事実上の標準。Sublime Textの`.tmTheme`と`.tmLanguage`に対応し、テーマ切替との相性が良い。`tree-sitter`も検討したが、Viewer特化のv0.1ではパフォーマンス要件が低く、syntectの方が導入コストが低い。

### 4. テーマシステム: CSS変数風のカラーパレット方式

**決定**: `ThemeColors` structに全色を集約し、Dark/Lightの定義済みパレットを提供する。

**理由**: egui は CSS変数をサポートしないため、struct内の色定義をViewModeのようにランタイムで切り替える。ハードコーディングされた色定数をAST Linterで検出する既存の仕組みと整合する。

### 5. PDF/画像エクスポート: comrak HTML → 外部変換

**決定**: Markdown → HTML（comrak）→ PDF/PNG/JPG は外部ツール連携で実現。

**手法**: 
- HTML出力: comrakのHTML出力をファイルに書き出し、デフォルトブラウザで開く。保存はブラウザに委譲。
- PDF/画像: ヘッドレスブラウザ（weasyprint または wkhtmltopdf）を外部コマンドとして呼び出す。インストールされていない場合はエラーメッセージでガイド。

**理由**: 特にPDFレンダリングを純Rustで高品質に実現するのはv0.1の工数に見合わない。ブラウザエンジンのレンダリング品質を活用する。

### 6. 目次（ToC）: comrak AST からの見出し抽出

**決定**: comrak の AST を走査して見出し階層を構築し、サイドパネルまたはオーバーレイで表示。

**理由**: 既にcomrakを使用しているため、追加の解析コストが低い。見出しクリックでプレビューのスクロール位置をジャンプさせる。デフォルトOFF。

### 7. アイコン: 画像生成 + icns変換

**決定**: AIによるアイコン画像生成 → macOSの`.icns`フォーマットに変換 → `Info.plist`で設定。

**理由**: デザイナー不在のため。生成後に`iconutil`でicns化。ビルドスクリプト（build.rs）でバンドルに含める。

### 8. スプラッシュスクリーン: egui ウィンドウとして実装

**決定**: 起動時に約1秒のスプラッシュウィンドウを表示（アイコン + バージョン）。egui の別ウィンドウではなく、メインウィンドウのフレーム0-N で条件分岐して描画する。

**理由**: 別ウィンドウだとウィンドウ遷移のチラつきが発生する。メインウィンドウ内でスプラッシュ→メインUIへのフェードにすることで自然な体験になる。

### 9. タブコンテキストメニュー: egui の `context_menu`

**決定**: eguiの`Response::context_menu`APIを使用。

**理由**: egui標準機能で、追加依存なし。右クリックでコンテキストメニューを表示し、閉じる系操作を提供する。

### 10. ワークスペースコンテキストメニュー: 同様にegui標準

**決定**: ツリーエントリへの右クリックで`context_menu`表示。

**理由**: フォルダに対して「全て開く」、ファイルに対して「メタ情報表示」を提供。
### 11. 設定UIパネル: サイドパネル方式

**決定**: 設定はサイドパネル（右側）として実装する。モーダルではない。

**理由**: モーダルはプレビューを遮る。右サイドパネルであればプレビューを見ながら設定変更の効果をリアルタイムに確認できる。プレビューペインの幅を縮めて設定パネルを表示する。egui の `SidePanel::right` で実現。

### 12. ワークスペースローディング: std::thread + channel 方式

**決定**: ワークスペースのツリー構築は `std::thread::spawn` でバックグラウンドスレッドに移し、`std::sync::mpsc::channel` で結果をメインスレッドに返す。

**理由**: eguiのイベントループはシングルスレッドで、tokio のランタイムとの統合がoverkill。std::threadで十分であり、依存も増えない。`AppState` にローディング状態（`enum WorkspaceLoadState { Loading, Loaded(TreeEntry), Error(String) }`）を追加し、毎フレームchannelを `try_recv` して完了を検知する。

### 13. ローカル画像プレビュー: 初回レンダリング時に全読み込み（キャッシュ付き）

**決定**: eguiにはビューポート概念がないため、遅延読み込みではなく、Markdown解析時に画像パスを抽出し、初回レンダリング時にバックグラウンドで全画像をロードする。ロード完了まではプレースホルダーを表示する。

**理由**: eguiの `ScrollArea` にはスクロール位置ベースのビューポートイベントがない。画像キャッシュ（HashMap<PathBuf, TextureHandle>）を保持し、2回目以降は即時表示。大量画像の場合はメモリ使用量に注意が必要だが、Viewerとしてのv0.1では実用上問題ない範囲。

### 14. シンタックスハイライト: egui_commonmark の syntax_theme_dark/light 機能

**決定**: `egui_commonmark` クレート自体がsyntectベースのシンタックスハイライト機能を内蔵している（`CommonMarkViewer` の `syntax_theme_dark` / `syntax_theme_light` メソッド）。追加で `syntect` を直接依存に入れる必要はない。

**理由**: egui_commonmark v0.22 は内部で syntect を使用しており、テーマ名を指定するだけでコードブロックのハイライトが有効になる。Dark/Light テーマ連動は `syntax_theme_dark("base16-ocean.dark")` と `syntax_theme_light("base16-ocean.light")` の切替で実現。独自統合より圧倒的にシンプルで、バイナリサイズ増加も最小限。

**影響**: Cargo.toml の `egui_commonmark` の features に `syntect` を追加する必要がある。タスク9のsyntect直接依存は不要になる。

### 15. カレント行表示: egui TextEdit layouter + カスタム背景描画

**決定**: egui の `TextEdit` が提供する `layouter` コールバックでカーソル位置を追跡し、`Painter::rect_filled` でカレント行の背景をハイライトする。

**理由**: egui TextEdit は `CursorRange` を `output.cursor_range` で返すため、カーソル行を特定できる。行番号列は `TextEdit` の左隣に `Label` カラムを配置して実装する。完全なカスタムエディタを作るよりも、既存 TextEdit を拡張する方がv0.1のスコープに適合する。

## Risks / Trade-offs

**[PDF出力の品質ばらつき]** → 外部ツール依存のため、ユーザー環境によって出力品質が異なる。初回リリースではweasyprint/wkhtmltopdfの手動インストールを要求。将来的にバンドル化を検討。

**[syntect のバイナリサイズ増加]** → egui_commonmark 経由で syntect が入るが、直接依存よりは軽量。feature flags で制御可能。

**[永続化パスのクロスプラットフォーム]** → macOS規約に従い`dirs`クレートで `~/Library/Application Support/katana/` を使用。将来的にLinux/Windows対応時に分岐。

**[テーマ切替のちらつき]** → 全ウィジェットの色を一括変更するため、一瞬ちらつく可能性。egui の `Visuals` 切替で対応可能だが、カスタムカラーとの整合に注意が必要。

**[スプラッシュスクリーンの固定時間]** → 起動が速い場合は無駄な待ち時間になる。タイマーはconfigurableにしつつ、デフォルト1秒とする。
