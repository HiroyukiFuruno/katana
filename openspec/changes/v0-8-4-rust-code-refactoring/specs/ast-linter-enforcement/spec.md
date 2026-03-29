## ADDED Requirements

### Requirement: `coding-rules.ja.md` のルールをast_linterで網羅的に機械化

`docs/coding-rules.ja.md` に記載されたルールのうち、機械的にチェック可能なものをすべてast_linterまたはclippy `#![deny]` で強制する。「定義はあるが守られる保証がない規約は無意味」の哲学に基づく。

---

### 1. ファイル行数制限（★新規 ast_linter）

- テストコード除外で200行を超えるソースファイルをエラーとする
- `#[cfg(test)]` モジュール内の行数は計測から除外する
- エラーメッセージに分離設計のガイダンスを含める（`types.rs` / `impls.rs` パターン等）
- プロジェクト単位の除外リスト（`emoji.rs` 等のデータファイル）をサポートする

### 2. 関数行数制限（★新規 ast_linter、clippy補完）

- 30行を超える関数をエラーとする
- テスト関数（`#[cfg(test)]` / `#[test]`）は除外する
- coding-rules §2 の「関数サイズ: 30行を上限」を機械的に強制
- 注: clippy `too_many_lines` でも検出可能だが、ast_linterで統一管理する

### 3. pub free function 禁止（★新規 ast_linter）

- coding-rules §1.1 の「ドメインロジックは必ず struct + impl ブロック」を強制
- `pub fn`（`pub(crate)` 含む）がモジュールトップレベルに定義されている場合にエラー
- 除外: `main()`, `#[test]` 関数, `mod tests` 内の関数

### 4. ネスト深度制限（★新規 ast_linter）

- coding-rules §3 の「ネスト最大3レベル」を強制
- `if` / `match` / `for` / `while` / `loop` のネスト深度が3を超える場合にエラー
- 注: clippy `cognitive_complexity` は認知的複雑さの指標であり、純粋なネスト深度とは異なる

### 5. clippy `#![deny]` 設定の統一（★設定修正）

- 各クレートの `lib.rs` / `main.rs` に以下の `#![deny]` を設定する:
  ```rust
  #![deny(
      clippy::too_many_lines,
      clippy::cognitive_complexity,
      clippy::wildcard_imports,
      clippy::unwrap_used,
      clippy::panic,
      clippy::todo,
      clippy::unimplemented,
  )]
  ```
- 現状は `#![deny(warnings)]` のみで、個別ルールの明示的な `#![deny]` が欠落している
- `coding-rules.ja.md` §9 に記載されているが実装されていない状態を解消する

### 6. linterルールファイルの分割（★構造改善）

- 既存の `rules/rust.rs`（969行）を個別ファイルに分離する
- `rules/rust/mod.rs` で公開APIを集約する
- 各Visitor を独立ファイルとする:
  - `magic_numbers.rs` (MagicNumberVisitor)
  - `prohibited_types.rs` (ProhibitedTypesVisitor)
  - `lazy_code.rs` (LazyCodeVisitor)
  - `font_normalization.rs` (FontNormalizationVisitor)
  - `performance.rs` (PerformanceVisitor)
  - `prohibited_attrs.rs` (ProhibitedAttributeVisitor)
  - `file_length.rs` (★新規)
  - `function_length.rs` (★新規)
  - `pub_free_fn.rs` (★新規)
  - `nesting_depth.rs` (★新規)

#### Scenario: platform クレートへ構造ルールを拡大する

- **WHEN** `katana-platform` レイヤーのリファクタリングを進める
- **THEN** `file_length`, `function_length`, `nesting_depth`, `error_first`, `pub_free_fn` を含む構造/コーディングルールが `katana-platform/src` に適用される

#### Scenario: ui クレートへ構造ルールを拡大する

- **WHEN** `katana-ui` レイヤーのリファクタリングを進める
- **THEN** `file_length`, `function_length`, `nesting_depth`, `error_first`, `pub_free_fn` を含む構造/コーディングルールが `katana-ui/src` に適用される

#### Scenario: `pub free fn` 禁止ルールを最終的に有効化する

- **WHEN** 既存の公開 free function 違反が解消された後に `make check` を実行する
- **THEN** `pub_free_fn` の統合テストは `#[ignore]` なしで有効になっている
- **THEN** `struct + impl` ベースのルールが linter/core/platform/ui の対象クレートで機械的に検証される
