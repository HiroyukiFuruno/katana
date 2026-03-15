# Katana Rust コーディングルール

本ドキュメントはプロジェクト全体で遵守すべき Rust コーディング規約を定義する。
linter で自動検査できるルールは `.clippy.toml` と各クレートの `#![deny(...)]` で強制する。

---

## 1. 構造と責務

### 1.1 struct + impl ベース設計

**ドメインロジックは必ず `struct` + `impl` ブロックで実装する。**
自由関数（free function）はモジュールプライベート（`pub` なし）の内部処理にのみ許容する。

```rust
// ✅ Good — struct + impl
pub struct DocumentLoader { ... }
impl DocumentLoader {
    pub fn load(&self, path: &Path) -> Result<Document, DocumentError> { ... }
}

// ❌ Bad — pub な外部向け処理を free function で実装
pub fn load_document(path: &Path) -> Result<Document, DocumentError> { ... }

// ✅ Good — モジュール内部の補助処理は free function OK
fn html_escape(s: &str) -> String { ... }
```

### 1.2 SOLID 原則

| 原則 | Rust での適用 |
|------|-------------|
| S (単一責務) | 1 つの `struct` / `impl` は 1 つの責務。30行超の `fn` は責務分離のサイン |
| O (開放閉鎖) | `trait` で拡張ポイントを定義し、`struct` への直接追加を避ける |
| L (リスコフ) | `trait` 実装は契約（ドキュメントの事前/事後条件）を破らない |
| I (インターフェース分離) | `trait` は小さく保ち、不要なメソッドをまとめない |
| D (依存関係逆転) | 上位レイヤーは具体型ではなく `trait` に依存する |

---

## 2. 関数サイズ

**1 関数（メソッド・自由関数問わず）は 30 行を上限とする。**
30 行を超える場合は SOLID の S 原則に従い責務を分離する。

- linter: `clippy::too_many_lines`（`too-many-lines-threshold = 30`）で自動検出
- `impl` ブロック自体の行数は対象外（メソッド単位で判定）

---

## 3. ネスト深度

**コードのネストは最大 3 レベルまで。努力目標は 2 レベル。**

```rust
// ✅ Good — let-else でアーリーリターン、ネスト 2
fn handle_save(&mut self) {
    let Some(doc) = &mut self.state.active_document else {
        return; // ← エラーファーストでネストを浅く保つ
    };
    match self.fs.save_document(doc) {
        Ok(()) => self.state.status_message = Some("Saved.".to_string()),
        Err(e) => self.state.status_message = Some(format!("Save failed: {e}")),
    }
}

// ❌ Bad — ネスト 4
fn handle_save(&mut self) {
    if let Some(doc) = &mut self.state.active_document {
        if doc.is_dirty {
            match self.fs.save_document(doc) {
                Ok(()) => {
                    if let Some(msg) = &mut self.state.status_message { ... }
                }
                ...
            }
        }
    }
}
```

- linter: `clippy::cognitive_complexity`（`cognitive-complexity-threshold = 10`）で自動検出
- `Result` の `?` 演算子を積極的に使い、`match` / `if let` のネストを減らす

---

## 4. エラーファースト

**後続処理に必要な値が得られない場合は即リターン・即エラーを返す。**
`?` 演算子と `let...else` が第一選択肢。

```rust
// ✅ Good
fn process(&self, path: &Path) -> Result<Output, MyError> {
    let content = std::fs::read_to_string(path)?;      // ? でアーリーリターン
    let parsed = parse(&content)?;
    Ok(transform(parsed))
}

// ❌ Bad — エラーを後回しにするネスト
fn process(&self, path: &Path) -> Result<Output, MyError> {
    if let Ok(content) = std::fs::read_to_string(path) {
        if let Ok(parsed) = parse(&content) {
            return Ok(transform(parsed));
        }
    }
    Err(MyError::Failed)
}
```

---

## 5. 型安全と非 null 設計

### 5.1 禁止型

TypeScript の `any` / `unknown` / `Record<string, unknown>` に相当する以下の使用を禁止する:

| 禁止 | 理由 | 代替 |
|------|------|------|
| `Box<dyn std::any::Any>` | 型消去 | 専用 `trait` / `enum` を定義する |
| `HashMap<String, serde_json::Value>` | 非構造化データ | 型付き `struct` を定義する |
| `serde_json::Value` (外部 API 境界以外) | 型安全の喪失 | 対応する `struct` + `#[derive(Deserialize)]` |

- linter: `clippy::wildcard_imports` で `use foo::*` を禁止

### 5.2 非 null 設計

**必ず存在する値を `Option` で包まない。**
`Option` が必要な個所は設計を見直し、`Option` が不要になるよう構造を変える。

```rust
// ✅ Good — 存在が保証される値は直接持つ
pub struct ActiveDocument {
    pub path: PathBuf,  // 常に存在する
    pub buffer: String, // 常に存在する
}

// ❌ Bad — 必ず初期化されるのに Option で包む
pub struct AppState {
    pub active_path: Option<PathBuf>, // 本当に Optional か？
}
```

- `unwrap()` は `deny(clippy::unwrap_used)` で禁止
  - テストコード内では `expect("明示的な理由")` のみ許容
- `panic!` は `deny(clippy::panic)` で禁止（テスト外）
- `todo!` / `unimplemented!` は `deny` で禁止（WIP ブランチを除く）

---

## 6. コメント規約

**コメントは「なぜ（WHY）」のみ。何をしているか（WHAT）はコードで表現する。**
コメントは **日本語** で記載する。

```rust
// ✅ Good — WHY のみ、日本語
// comrak はデフォルトで GFM 無効なので、拡張を明示的に有効化する。
opts.extension.table = true;

// ❌ Bad — WHAT をコメントしている（コードを読めばわかる）
// テーブル拡張を有効にする
opts.extension.table = true;
```

ドキュメンテーションコメント（`///`）は公開 API に対して英語で記載する（crates.io / rustdoc 慣習に従う）。

---

## 7. テスト規約

### 7.1 テスト命名

**テストの `fn` 名は日本語スネークケース（ascii のみ）またはスネークケース英語で意味を表す。**
`describe` に相当するグルーピングは `mod` で行う。

```rust
// ✅ Good
#[test]
fn 未保存バッファはディスクに書き込まれない() { ... }

// または英語スネークケース（長い日本語が難しい場合）
#[test]
fn unsaved_buffer_does_not_write_to_disk() { ... }
```

### 7.2 テストファイル配置

テストは **クレートルートの `tests/` ディレクトリ** に配置する。
`src/` 内の `#[cfg(test)] mod tests { ... }` は禁止（テストヘルパー関数の `#[cfg(test)]` アトリビュートは許容）。

```
crates/katana-core/
  src/              # 実装コードのみ
  tests/
    document.rs     # document.rs のテスト
    workspace.rs    # workspace.rs のテスト
    preview.rs      # preview.rs のテスト
    ai.rs           # AI モジュールのテスト
    markdown_*.rs   # 各レンダラーのテスト
    plugin.rs       # プラグインのテスト
```

### 7.3 テストピラミッド

| 種別 | 配置 | カバレッジ目標 |
|------|------|--------------| 
| Unit Test | `tests/` ディレクトリ | **100%（例外なし）** |
| Integration Test | `tests/` ディレクトリ | 主要フロー網羅 |
| E2E Test | `tests/e2e/` | MVP の全シナリオ |

カバレッジ測定: `cargo llvm-cov --workspace --fail-under-lines 100`（CI 強制）

---

## 8. 変数・型命名

**省略形は禁止。将来の読み手を意識した完全な名前を使う。**

```rust
// ✅ Good
let workspace_root = Path::new("/home/user/project");
let active_document = Document::new(path, content);

// ❌ Bad
let ws = Path::new("/home/user/project");
let doc = Document::new(path, content);  // ← 文脈が失われる
```

**クロージャ引数**: Kotlin の `it` イディオムに倣い、単一引数クロージャは `it` を使う。

```rust
// ✅ Good — 単一引数クロージャ
entries.iter().filter(|it| it.is_markdown())

// ✅ Good — for 式は可読性重視の命名
for entry in &ws.tree { ... }
for plugin_meta in registry.active_plugins_for(&point) { ... }
```

---

## 9. Linter 設定サマリ

各クレートの `lib.rs` / `main.rs` の先頭に以下を付与する:

```rust
#![deny(
    clippy::too_many_lines,         // 関数30行超を禁止
    clippy::cognitive_complexity,   // ネスト深度プロキシ
    clippy::wildcard_imports,       // use foo::* を禁止
    clippy::unwrap_used,            // unwrap() を禁止
    clippy::panic,                  // panic! を禁止
    clippy::todo,                   // todo! を禁止
    clippy::unimplemented,          // unimplemented! を禁止
    clippy::exhaustive_structs,     // 公開 struct の非網羅的なパターン警告 (warn にする場合あり)
)]
#![warn(
    clippy::expect_used,            // expect() は警告（理由付きなら許容）
    clippy::indexing_slicing,       // インデックスアクセスを警告
    clippy::missing_errors_doc,     // pub fn の Result に doc 必須
    missing_docs,                   // pub アイテムに doc コメント必須
)]
```

`.clippy.toml` で閾値を設定:

```toml
too-many-lines-threshold = 30
cognitive-complexity-threshold = 10
```

### 9.2 品質ゲート（完了の定義 / Definition of Done）

PR をマージ可能とするための必須条件:

1. **フォーマット**: `cargo fmt --all -- --check` パス
2. **Clippy**: `cargo clippy --workspace -- -D warnings` パス（warning ゼロ）
3. **テスト**: `cargo test --workspace` 全パス
4. **テスト配置**: 新規ロジックには `tests/` ディレクトリにテストが付随している
5. **カバレッジ**: `cargo llvm-cov --workspace --fail-under-lines 100` パス

一括チェック: `make ci`（pre-push フックと同等）

---

## 10. 例外申請プロセス

以下のいずれかに該当する場合のみ `#[allow(...)]` を許容する：

1. egui の `update()` など、フレームワーク都合で分割不能な場合
2. 生成コードやマクロ展開結果
3. PR レビューで合意を得た設計上の理由がある場合

`#[allow(...)]` には **必ず日本語コメントで理由を記載** すること：

```rust
// egui の App::update は単一エントリポイントのためフレームワーク制約で分割不能。
#[allow(clippy::too_many_lines)]
fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) { ... }
```

---

## 11. i18n（国際化）規約 【最重要・違反ゼロを維持すること】

**UI に表示するすべての文字列は `i18n::t()` または `i18n::tf()` を経由しなければならない。**
ハードコーディングは言語問わず **一切禁止**（英語・日本語・記号以外のすべて）。

### 11.1 対象となる呼び出し

以下の呼び出しに文字列リテラルを直接渡すことを禁止する：

| 禁止パターン | 正しいパターン |
|------------|--------------|
| `ui.heading("Preview")` | `ui.heading(i18n::t("preview_title"))` |
| `ui.label("Ready")` | `ui.label(i18n::t("status_ready"))` |
| `ui.button("Save")` | `ui.button(i18n::t("menu_save"))` |
| `.on_hover_text("Expand all")` | `.on_hover_text(i18n::t("expand_all"))` |
| `format!("Saved: {name}")` | `i18n::tf("status_saved_as", &[("name", &name)])` |

### 11.2 例外（記号・非テキスト）

以下は翻訳不要として許容する：

- 単一記号: `"*"`, `"x"`, `"+"`, `"-"`, `"▼"`, `"▶"` など
- アイコン絵文字: `"🔄"` など UI コントロール用記号
- パス区切り: `"/"` のみの文字列

### 11.3 ロケールファイル管理

- 新しいキーは **en.json と ja.json に同時追加** する（片方だけの追加は禁止）
- キーは `snake_case` で命名する
- パラメータは `{param_name}` 形式のプレースホルダを使う

```json
// ✅ Good
"status_opened_workspace": "Opened workspace: {name}"

// ❌ Bad — ハードコード
self.state.status_message = Some(format!("Opened workspace: {name}"));
```

### 11.4 UT による自動検出（ルール + UT の二重拘束）

`i18n.rs` の `#[cfg(test)]` に以下のテストを維持すること：

1. **`全ロケールキーが両言語に存在する`**: en.json と ja.json のキー差分をゼロにする
2. **`shellrsにi18n漏れがない`**: `shell.rs` のソースコードを静的解析し、高リスクな
   ハードコードパターン（`.on_hover_text("`, `ui.heading("` 等）を検出する

新しいファイルに UI を追加した場合は、対応する静的解析テストも追加すること。

### 11.5 違反検出フロー

```
コード変更
  ↓
cargo test (UT で自動検出)  ← 一次防衛
  ↓
PR レビュー (AI セルフレビューで確認)  ← 二次防衛
  ↓
マージ
```
