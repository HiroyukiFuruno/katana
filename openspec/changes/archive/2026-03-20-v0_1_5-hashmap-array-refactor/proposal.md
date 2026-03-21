## Why

KatanA ワークスペース内で `HashMap` および固定長配列 `[T; N]` / リテラル配列 `[a, b]` が使用されていることによる、イテレーション順序の非決定性や型安全性の低下を防ぐためです。より厳密な構造化（ドメイン特化の構造体（Data Class）の List (`Vec<T>`)）を採用し、堅牢性と予測可能性を高めることを目的とします。

## What Changes

- AST Linter（`ast_linter.rs`）に `HashMap` とプリミティブ配列の仕様禁止ルールを追加します。
- 既存の `HashMap<K, V>` 使用箇所（例: `settings.json` の `extra`、AIプロバイダ、i18nディクショナリ等）をすべて `Vec<Struct>` (Data Classのリスト) へ全面的に置換します。
- 既存の配列 (`[T; N]`) および配列リテラル使用箇所を `Vec<T>` / `vec![]` に置換します。

## Capabilities

### New Capabilities
- `linter-rules`: AST Linterに追加される `HashMap` 禁止および固定長・リテラル配列禁止ルール。

### Modified Capabilities
- `settings`: `settings.json` における設定の構造変更とマイグレーション（`extra: HashMap` から `extra: Vec<ExtraSetting>` への移行）。
- `i18n`: 辞書構造のリスト化（`HashMap` から `Vec<I18nDictionaryEntry>` への移行）。
- `ai-providers`: `params` および `metadata` などのマップ構造のリスト化（`Vec<AiParam>`等への移行）。

## Impact

- `katana-linter` (Linterへのルール追加)
- `katana-platform` / `katana-core` / `katana-ui` 全域の各種状態モデル（`settings.rs`, `app_state.rs`, `i18n.rs` 等）
- JSONシリアライズ/デシリアライズのスキーマ変化とマイグレーション処理
- 自動テスト（各種フィクスチャの `Vec` 対応）
