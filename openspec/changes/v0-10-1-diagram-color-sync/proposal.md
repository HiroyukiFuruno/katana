# OpenSpec: v0.10.1 Diagram Color Dynamic Sync

## Change Category

- [ ] Feature
- [ ] Refactor
- [x] Bugfix
- [ ] Technical Debt

## Objective

ユーザーがテーマ（Dark/Light等）およびテキストカラーを変更した際、MermaidおよびPlantUMLのレンダリング結果の色が連動して変わらない不具合を解消する。

## Background & Problem

現在、MermaidやPlantUML等のダイアグラムの配色は `DiagramColorPreset` などの `OnceLock` なグローバルステートによって初期化時に固定されてしまう。そのため、アプリケーション起動後に設定画面からテーマを動的に変更しても、ダイアグラム内の色付きテキスト（エラーメッセージ、ノードラベルなど）が古いテーマの配色のまま維持されてしまう問題（デグレードの再発）が発生している。

## Proposal

`OnceLock` を用いた設計を廃止・見直し、以下のいずれかのアプローチでダイアグラムへ動的なテーマ色情報を注入する。

1. **Theme Propagation:**
   `egui_commonmark` のレンダリングサイクル（または `AppAction::RefreshDiagrams` 等のキャッシュ無効化フロー）において、リクエストごとに現在の `ThemeStore` または `ui.visuals().text_color()` から生成した Hex カラーコードをバックエンド (Mermaid CLI / PlantUML Server) 側にフォワードする仕組みを導入する。

2. **Diagram Component Initialization Refactor:**
   グローバル変数への依存を止め、プレビュー更新時にコンテキストとしてカラースキームを渡せるようにする。

これにより、テーマ変更時に即座に一貫したカラーリングでダイアグラムが正しく再レンダリングされるようにする。

## Related Issues

- The user detected degradation in plantuml and mermaid color after implementing dynamic theme preview coloring.

## Definition of Ready (DoR)

- [x] 本Issueの目的が合意されていること。
- [x] Mermaid CLIおよびPlantUMLの配色注入の技術的実現可能性が検証されていること（もしくは調査タスクが切られていること）。
