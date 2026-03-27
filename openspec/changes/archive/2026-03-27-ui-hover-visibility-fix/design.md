## Context

ユーザーから「エディタ側のカレント行背景、ホバー行背景の透明度が低すぎて（あるいは配色が不適切で）視認性の問題を解消できていない」「プレビュー側にも（同様のハイライトとして）ホバー行背景が必要ではないか」との指摘を受けました。
調査の結果、KatanAの全テーマ共通でホバー/アクティブ背景に黒（`Rgba::new(0, 0, 0, alpha)`）が使用されており、ダークテーマにおいて地の文の背景色（暗いグレーや黒）との明度差がまったく出ない状態であることが判明しました。また、プレビュー描画を担う `vendor/egui_commonmark` では各背景色がハードコード（固定の薄い色）されており、現在のKatanAのテーマエンジンと連携していませんでした。

## Goals / Non-Goals

**Goals:**

- ダークテーマのホバー行・現在行の背景を「白透過（`Rgba(255,255,255, x)`）」へ移行し、視認性を飛躍的に高める。
- `PreviewColors` にホバー・現在行の背景の定義を含め、`egui_commonmark` の Builder へ色を注入する機能を開通させる。
- 既存のすべての組み込みテーマファイル（30種類弱）を一元的に修正しカバレッジ率・ビルドエラーを発生させないこと。

**Non-Goals:**

- `egui_commonmark` 自体の根本的な描画アーキテクチャや機能の変更。
- テーマエンジンそのもの（`ThemeColors` の管理機構やデータ構造）の抜本的リプレイス。

## Decisions

### 1. テーマの透過背景のRGBアプローチの適正化

すべての `ThemeMode::Dark` のテーマ定義に対し：

- `current_line_background`: `Rgba { r: 0, g: 0, b: 0, a: 15 }` → `Rgba { r: 255, g: 255, b: 255, a: 15 }`（または視認性の高い推奨値）
- `hover_line_background`: `Rgba { r: 0, g: 0, b: 0, a: 10 }` → `Rgba { r: 255, g: 255, b: 255, a: 10 }`
へと変更します（白をベースにすることで、暗い背景に薄いハイライトが当たります）。
ライトテーマ（`ThemeMode::Light`）の場合は既存の黒ベース透過のままですが、Alphaを `a: 15` (Hover), `a: 20` (Current) 程度に微増させ、視認性を補強します。

### 2. Preview 側の Hover/Active 背景の組み込みと連携

`PreviewColors` 構造体に `current_line_background: Rgba` と `hover_line_background: Rgba` を追加します（過去の保存データ互換性を保つために `#[serde(default)]` を設定）。
`vendor/egui_commonmark/src/lib.rs` において、`CommonMarkViewer` のビルダに:

- `.active_bg_color(egui::Color32)`
- `.hover_bg_color(egui::Color32)`
を追加し、`pulldown.rs` 内でハードコードされていた `egui::Color32::from_white_alpha(8)` 等の代わりに注入された設定色を使用します。

### 3. katana-ui層の実装

`preview_pane_ui.rs` にて、テーマ情報から取得した `preview.current_line_background` 等を `egui::Color32` に変換し、`CommonMarkViewer::new()` のビルダチェインに流し込み、エディタとの UI 反応を完全同期させます。

## Risks / Trade-offs

- **[Risk] `PreviewColors` の構造体変更による `settings.json` のデシリアライズ失敗**
  - **Mitigation**: Serde の `#[serde(default = "default_preview_current_line_background")]` などを用いてデフォルト値関数を指定し、JSONにキーが存在しない場合でもパースが成功し、安全な既定色が設定されるように防護策を講じます。
