## 1. Schema & Backend Logic

- [x] 1.1 `crates/katana-platform/src/settings/types.rs` の `ThemeSettings` に `ui_contrast_offset: f32` を追加する
- [x] 1.2 同ファイルの `PerformanceSettings` に `http_image_cache_retention_days: u32` を追加する
- [x] 1.3 `crates/katana-ui/src/theme_bridge.rs` (および types.rs) に、オフセット率（-100%〜+100%）を受け取りアルファを加算・減算する `with_contrast_offset` 関数を実装する
- [x] 1.4 `crates/katana-ui/src/http_cache_loader.rs` に、OSエラーを緩和しながら `http-image-cache` ディレクトリ内のファイルを安全削除し全削除する `forget_all` 改修を実装する

## 2. Linter & Test Adjustments

- [x] 2.1 `settings_window.rs` の Linterルールによるカバレッジを維持するため、新規追加されたスキーマをUIに適切にマッピングする
- [x] 2.2 Serde互換テストやデフォルト値に関する単体テストを拡張・修正する

## 3. UI Integration

- [x] 3.1 `crates/katana-ui/src/settings_window.rs` の「外観 (Appearance) > テーマ」セクションに「UI透過度補正 (Contrast Adjustment)」のスライダーUIを追加し、設定値と連動させる
- [x] 3.2 同画面の「振る舞い (Behavior)」等セクションに「画像キャッシュの一括削除」ボタンを配置し、クリック時に `forget_all` を実行するロジックをバインドする
