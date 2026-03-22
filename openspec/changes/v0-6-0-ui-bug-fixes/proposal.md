## Why

- [P0] タブ切り替え時に毎度Markdown全体を強制再レンダリング処理（full_refresh_preview）してしまうことで、CPUリソースが100%に張り付く構造的欠陥が存在した。
- [P1] KatanaApp 初期化時のタイマー開始タイミングが不適切で、重い読み込み処理によりスプラッシュ画面が即座に消去されメインUIが先行表示されてしまっていた。
- [P3] Blockquote表示時や、空でないコードブロック時に不要なエラーアイコンが出るなど、UI上の描画不備があった。

## What Changes

本パッチ(v0.6.0)は上記P0/P1を中心としたクリティカルなバグ修正を含む。

- `KatanaSvgLoader` の無効な監視ループを削除し、適切なエラー（`LoadError::NotSupported`）を返すよう修正。
- `shell.rs` において、`tab.hash` （キャッシュ）を判定し、同一ハッシュなら早期リターンするよう最適化。
- `KatanaApp::update` 初回パスで `splash_start` を初期化するように改修。
- Blockquote周りのパディング見直しおよびSVGアイコンのロード不備改善。

## Capabilities

### New Capabilities
<!-- Capabilities being introduced. -->

### Modified Capabilities

- `desktop-viewer`: タブ切り替え時のリロードロジック最適化とスプラッシュ起動順序の改善

## Impact

KatanaUIの `shell_ui.rs` と `shell.rs` 周辺、および `egui_commonmark` 内部のパーサーにおける微細な改修。
