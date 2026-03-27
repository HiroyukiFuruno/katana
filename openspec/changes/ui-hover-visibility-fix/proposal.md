## Why

KatanAのエディタ側カレント行・ホバー行の背景色は、現在すべてのテーマ（ライト/ダーク問わず）において「黒（RGB: 0, 0, 0）の低透過度（Alpha: 10〜15）」が設定されています。この設定はライトテーマでは機能しますが、ダークテーマ（背景色がすでに黒に近い状態）においてはほとんど視認できず、どこがホバー/アクティブ化されているか判別できないUX上の問題を引き起こしています。
また、プレビュー側においても行ホバーの背景色が `vendor/egui_commonmark` 内でハードコードされた極端に薄い色（Alpha: 8）であり、ユーザーにとって不可視化されている状態です。これを根本から解決し、テーマ設定に紐づいた適切なハイライトを提供する機能改善が必要です。

## What Changes

- **全ダークテーマの透過背景色の最適化**: `CodeColors` における `current_line_background` と `hover_line_background` について、ダークテーマの場合は「白（RGB: 255, 255, 255）の低透過度（Alpha: 15〜25）」に変更し、明瞭にハイライトされるように修正します。ライトテーマも視認性のためにAlphaをわずかに引き上げます（Alpha: 15〜20）。
- **プレビュー側のホバー/アクティブ行背景のテーマ連動**: `PreviewColors` に `current_line_background` と `hover_line_background` の定義を追加し、`vendor/egui_commonmark` へ背景色を外部から注入（Builderパターン）可能に改造します。
- **PreviewPane UIの描画ロジック更新**: `preview_pane_ui.rs` にてプレビュー側のMarkdown描画時、上記の設定色を `egui_commonmark` のAPIへ流し込み、エディタ・プレビュー間の色味を統一させます。

## Capabilities

### New Capabilities

- `ui-visibility`: エディタおよびプレビューの実践的な行背景・ハイライト視認性管理

### Modified Capabilities

## Impact

- `crates/katana-platform/src/theme/types.rs` の `PreviewColors` スキーマ拡張。
- `crates/katana-platform/src/theme/presets/` 以下の全約30テーマの配色の自動または手動修正。
- `vendor/egui_commonmark/src/lib.rs` および `vendor/egui_commonmark/src/parsers/pulldown.rs` における背景色ハードコードの撤廃とビルダーAPIの追加。
- `crates/katana-ui/src/preview_pane_ui.rs` と `crates/katana-ui/src/preview_pane.rs` の連携アップデート。
