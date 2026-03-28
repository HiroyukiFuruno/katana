## Why

v0.7.10で修正すべきデグレードが発生しています。

1. **テーマ透過度補正の欠落**: `ThemeColors` スキーマのリファクタリング時（v0.7.7 / commit a0b288a）に、UIコントラスト設定（`ui_contrast_offset`）を用いて色情報（`Rgba`）のアルファ値を調整する共通関数 `ThemeColors::with_contrast_offset` および `Rgba::with_offset` が誤って削除されました。これにより、ホバー行やアクティブ行など半透明を期待する背景色が適切な視認性を持たず、ダークテーマ等で背景に埋もれる原因となっています。

## What Changes

- **透過度補正ロジックの復元**: `crates/katana-platform/src/theme/types.rs` へアルファ値補正計算処理を復旧し、`crates/katana-platform/src/settings/impls.rs` にてUIへテーマカラーを適用する最終段階（`effective_theme_colors()`）で、グローバルな `ui_contrast_offset` をもとに補正をかける設計に戻します。

## Capabilities

### Modified Capabilities

- `ui-visibility`: 透過度補正がすべてのRgbaフィールドで適切に作動するよう修正

## Impact

- `crates/katana-platform/src/theme/types.rs`
- `crates/katana-platform/src/settings/impls.rs`
