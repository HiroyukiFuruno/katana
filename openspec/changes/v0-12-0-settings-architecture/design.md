# 設計: Settings アーキテクチャ分離設計

## アーキテクチャ概要

`theme.rs` の事例を踏襲し、`crates/katana-platform/src/settings` モジュールを作成して各責務を完全に独立させます。

### 1. ディレクトリ構造

```text
crates/katana-platform/src/settings/
  mod.rs          // 各モジュールの pub use
  types.rs        // AppSettings, ThemeSettings などの純粋な型定義
  impls.rs        // 各種構造体の Default やカスタムロジック
  service.rs      // SettingsService の実装
  repository.rs   // SettingsRepository などストレージ保存層
  migration/      // 旧 migration*.rs ファイル群の統合先
    mod.rs
    migration_xxx.rs
```

### 2. コンポーネント設計

#### `types`
- セーブデータのキーとなる `AppSettings` を筆頭に、`WindowSettings`, `EditorSettings`, `TerminalSettings`, `ThemeSettings` など全ての関連する struct と enum の定義を一つにまとめます。
- このファイル内には `impl` ブロックを一切定義せず、データの「形」だけを明確にします。

#### `impls`
- `types` で定義された構造体に対する `impl Default for ...` や、便利な生成メソッドなどを定義します。
- これによって「設定のデフォルト状態」がこのファイルに完全にまとまるため、UI 側からのデフォルト値確認が容易になります。

#### `migration`
- デシリアライズに失敗した場合に、古いバージョン（例: 0.2.x や 0.3.x）のフォーマットから最新の構造へ安全にパース・引き上げるロジック群。
- これまで `crates/katana-platform/src/` 配下に散らばっていたマイグレーションファイル群をまとめることで、プロジェクトルートが簡潔になり保守性が向上します。

### 3. 主な変更点

- `settings.rs` を `settings/mod.rs` にし、`pub use types::*;` などの re-export で API 互換性を外部に提供します。
- 外部（`katana-ui` 等）からの `katana_platform::settings::AppSettings` の利用パスは維持されます。
