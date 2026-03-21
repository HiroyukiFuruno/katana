## Why

Markdown 内のローカル画像が表示されず、READMEの閲覧品質が劣る。また、アプリとしてのブランディング（スプラッシュスクリーン）やメニューの充実が不足しており、「製品」としての完成度が低い。画像表示・スプラッシュ・メニュー拡充を実装し、v0.4.0 で「見栄えの良いプロダクト」としての完成度を引き上げる。

## What Changes

### ローカル画像プレビュー & ビューアコントロール
- comrak AST から画像ノードを解析し、ローカル相対パスを検出
- バックグラウンドで全画像を読み込み（完了までプレースホルダー表示）
- PNG, JPG, GIF, SVG 対応
- 画像キャッシュ（`HashMap<PathBuf, TextureHandle>`）
- 画像および各ダイアグラム（mermaid, drawio, plantuml）上に、拡大・縮小・パン操作・モーダル別表示が可能なサブコントロールUIをオーバーレイ追加

### スプラッシュスクリーン
- メインウィンドウのフレーム0〜N でスプラッシュ画面を描画
- アイコン + バージョン番号の中央表示
- 約1秒後にフェードアウト、クリックでスキップ

### メニュー拡充
- About ダイアログの最適化（アイコン表示含む）
- Help メニュー（GitHub Repository へのブラウザ遷移）
- 寄付メニュー（「準備中」表示）
- macOS ネイティブメニュー + 非macOS フォールバック対応

## Capabilities

### New Capabilities
- `local-asset-preview`: ローカル画像の遅延読み込みプレビュー
- `diagram-image-viewer-controls`: 画像・ダイアグラムの拡大縮小・パン・モーダル表示コントロール
- `app-branding`: スプラッシュスクリーン
- `menu-enhancement`: About/Help/寄付メニュー

## Impact

- **katana-core**: 画像パス解析ユーティリティ
- **katana-ui** (`preview_pane.rs`): 画像読み込み・キャッシュ・プレースホルダー
- **katana-ui** (`shell.rs`): スプラッシュ画面、メニュー項目追加
- **Cargo.toml**: `open` クレート依存追加
- **macOS**: `macos_menu.m` 更新
