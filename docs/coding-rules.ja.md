# Katana Rust コーディングルール

> [!IMPORTANT]
> **Bi-directional Update Required / 双方向アップデート必須**
> 本ファイル（`coding-rules.ja.md`）を更新する際は、必ず英語版（`coding-rules.md`）も同時に同期更新してください。
> When updating this file (`coding-rules.ja.md`), you MUST also update the English version (`coding-rules.md`) simultaneously to keep them in sync.

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

### 1.3 ステート分離アーキテクチャ (Global, Session, Tab)

状態（State）は影響範囲とそのライフサイクルに基づいて明確に分離設計を行わなければなりません。
単一の巨大な構造体への融合や、場当たり的な `HashMap` の乱立は技術的負債となります。

| State Layer | 説明 (Description) | 例 (Example) |
|-------------|--------------------|--------------|
| **Global**  | 永続化が必要な全体設定。常に最新として扱われる。 | `SettingsService` (分割方向の基本設定など) |
| **Session** | 一時的なアプリ実行状態。揮発する。 | `pending_action`, ウィンドウのリサイズ状態 |
| **Tab (固有管理)** | ドキュメントタブごとに独立した固有のキャッシュ。 | `SplitViewState` (タブ固有の分割状態のオーバーライド) |

**実装の原則:**
- タブが開かれた瞬間に Global 設定から Initial Value をコピー・キャッシュして Tab 固有の状態（`SplitViewState`等）を生成・分離します。
- UIの個別要素（トグルボタン等）は **固有管理 (Tab)** の状態のみをミューテートします。
- Global 設定が変更された際は、既存の Tab 状態を再初期化（最新のGlobal値で上書き同期）して状態の不一致を防ぎます。

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
コメントは **英語** で記載する（English First Policy に従う）。

```rust
// ✅ Good — WHY のみ、英語
// comrak disables GFM by default, so we explicitly enable the extension here.
opts.extension.table = true;

// ❌ Bad — WHAT をコメントしている（コードを読めばわかる）
// Enable the table extension
opts.extension.table = true;
```

ドキュメンテーションコメント（`///`）は公開 API に対して英語で記載する（crates.io / rustdoc 慣習に従う）。

---

## 7. テスト規約

### 7.1 テスト命名

**テストの `fn` 名はスネークケース英語で意味を表す。**
`describe` に相当するグルーピングは `mod` で行う。

```rust
// ✅ Good
#[test]
fn unsaved_buffer_does_not_write_to_disk() { ... }
```

### 7.2 テストファイル配置（更新 2026-03-19）

Rust コミュニティのベストプラクティスに合わせ、テストを **種別** で分離する。

| 種別 | 配置 | 説明 |
|------|------|------|
| **Unit Test (UT)** | `src/` 内 `#[cfg(test)] mod tests { ... }` | 同一クレート内の private API を直接テストできる。Rust 標準スタイル。 |
| **Integration Test (IT)** | `tests/` ディレクトリ | クレートの公開 API のみアクセス可能。外部利用者視点の検証。 |

```text
crates/katana-core/
  src/
    document.rs           # 実装 + #[cfg(test)] mod tests { ... }  (UT)
    workspace.rs          # 実装 + #[cfg(test)] mod tests { ... }  (UT)
  tests/
    integration.rs        # 公開 API を通じた結合テスト (IT)
    markdown_renderer.rs  # レンダラーの結合テスト (IT)
```

**UT のガイドライン**

- `src/` モジュールと同一ファイル内の `#[cfg(test)] mod tests { ... }` に記述する。
- private 関数・内部状態のテストに使用してよい。
- `#[allow(clippy::unwrap_used)]` は `mod tests` ブロックにのみ許容する。

**IT のガイドライン**

- `tests/` ディレクトリに配置する。
- クレートの `pub` API のみを使用する（private アクセス不可）。
- `use crate_name::...;` 形式でインポートし、外部利用者と同等の視点でテストする。

### 7.3 テストピラミッド

| 種別 | 配置 | カバレッジ目標 |
|------|------|--------------|
| Unit Test (UT) | `src/` 内インライン `#[cfg(test)]` | **100%（例外なし）** |
| Integration Test (IT) | `tests/` ディレクトリ | 主要公開 API フロー網羅 |
| UI Integration Test | `tests/integration/` (egui_kittest) | 全MVPシナリオ、および実レスポンス（Node, Rect等）のアサーション。**Snapshotテストは禁止（NG）** |

カバレッジ測定: `cargo llvm-cov --workspace --fail-under-lines 100`（CI 強制）

### 7.4 TDD 強制（テスト駆動開発）

**すべての新機能・バグ修正は TDD サイクル: Red → Green → Refactor に従わなければならない。**
本プロジェクトの哲学「定義はあるが守られる保証がない規約は無意味」はここにも適用される。

#### 必須プロセス

1. **Red**: 実装コードを書く**前に**、期待する挙動を定義する失敗するテストを書く。
2. **Green**: テストを通す最小限の実装を書く。
3. **Refactor**: すべてのテストがグリーンのまま実装をクリーンアップする。

#### 実践での意味

| シナリオ | 必須アクション |
|---|---|
| 新機能（例: HTML要素の中央寄せ） | 期待するレンダリング位置をアサートするテストを書く。実装前にテストが**失敗する**ことを確認する。 |
| バグ修正（例: 要素が中央寄せにならない） | バグを再現するテストを書く（正しい挙動をアサート）。テストが**失敗する**ことを確認してバグが捕捉できていることを検証。その後コードを修正する。 |
| リファクタリング | リファクタリングの前後で既存のテストがすべて通ることを確認する。インターフェースが変更されない限りテストの修正は不可。 |

#### アンチパターン（厳禁）

```text
❌ 先に実装してからテストを書く（または書かない）
❌ 実装コードを変更しユーザーに「目視で確認してください」と依頼する
❌ 自動検証なしに複数回の実装試行を繰り返す
❌ 実装を通すためにテストを削除または弱体化する
```

#### egui_kittest による UI テスト (Snapshotは禁止（NG）— 実レスポンス検証)

**ベストプラクティス：UIの検証は「実レスポンス（出力）」を検証すること。**
UIにおける「差分しか検出できないSnapshotテスト (`.png` 比較)」への依存は技術的負債として**禁止（NG）**とする。
Snapshotテストは実行が遅く、失敗しても開発者が「ただ画像がズレただけ」と脳死で `UPDATE_SNAPSHOTS=1` を叩いてしまうため、デグレードの担保として全く機能しない。

**正しいITパターン — ロジックを直接検証する：**

| 検証対象 | 正しいアプローチ (✅) | アンチパターン (❌) |
|---|---|---|
| ウィジェット配置 | AccessKitノードから `Rect` の位置・幅をアサート | レンダリング画像のSnapshot比較 |
| クリック動作 | クリック → 状態変化をアサート（例: `active_doc_idx`） | Snapshotでホバー/ハイライトを目視 |
| 色・視認性 | 色チャンネル値をコントラスト閾値と比較 | スクリーンショートのピクセル比較 |
| ツールチップ | i18nキー解決をアサート（`assert_ne!(text, key)`） | ホバーSnapshotの比較 |

**不具合・デグレード発生時の鉄則：**
不具合やデグレードが発生した際は、**コードを修正する前に、まずTDD（RED）でその失敗（バグの挙動）を再現・制限すること**を絶対の義務とします。
その際の検証には、必ず **「実際のUIエンジンのレスポンス（出力としての `Rect` の幅・高さ、`Z-index` の描画順レイヤー など）」をプログラム的に直接アサート** し、Snapshotに逃げないでください。

```rust
// ✅ Good — アサーション・ベースのUI実レスポンス検証 (TDD/REDでバグを再現・証明)
#[test]
fn new_horizontal_split_starts_at_half_width() {
    let mut harness = Harness::new_ui(|ui| { /* ... */ });
    harness.run();
    
    // パネルのRectを直接取得し、画面幅全体の50%（期待値）と一致するかをピクセル単位でアサートする
    let panel_rect = egui::containers::panel::PanelState::load(&ctx, id).unwrap().rect;
    assert!((panel_rect.width() - 600.0).abs() <= 4.0, "must start at 50%");
}

// ❌ Bad — 思考停止を招くSnapshot画像比較 (技術的負債)
#[test]
fn split_looks_correct() {
    let mut harness = Harness::new_ui(|ui| { /* ... */ });
    harness.step();
    // 意味がなく、CI遅延の要因。技術的負債となる。
    harness.snapshot_options("split_looks_correct", ...);
}
```

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
3. **テスト (ロジック)**: `cargo test --workspace` 全パス
4. **テスト (統合)**: `make test-integration` パス（実レスポンス・アサーションのみ、Snapshot不使用）
5. **テスト配置**: 新規ロジックには `src/` 内インライン UT、または境界横断シナリオ向けの `tests/` 配下 IT が付随している
6. **カバレッジ**: `cargo llvm-cov --workspace --fail-under-lines 100` パス

一括チェック: `make check`（pre-push フックと同等）

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

## 11. i18n（国際化）規約 【最重要・違反ゼロ維持】

**UI に表示するすべての文字列は `i18n::t()` または `i18n::tf()` を経由しなければならない。**
ハードコーディングは言語問わず **一切禁止**（英語・日本語・記号以外のすべて）。

本プロジェクトの哲学「定義はあるが守られる保証がない規約は無意味」に基づき、このルールは第12章で定義される **カスタム AST Linter によって機械的に強制** される。

### 11.1 対象となる呼び出しと AST 検知

以下の呼び出しメソッド群の引数において、文字列リテラル（`"..."` や `String::from("...")` などのハードコード）を直接渡すことは AST 解析によってエラーとして弾かれる。

| メソッド名 / 検知対象 | 正しいパターン (Linter通過) | ❌ 禁止パターン (Linterで弾かれる) |
|---|---|---|
| `ui.label(...)` | `ui.label(i18n::t("status_ready"))` | `ui.label("Ready")` |
| `ui.heading(...)` | `ui.heading(i18n::t("preview_title"))` | `ui.heading("Preview")` |
| `ui.button(...)` | `ui.button(i18n::t("menu_save"))` | `ui.button("Save")` |
| `RichText::new(...)` | `RichText::new(i18n::t("alert"))` | `RichText::new("Alert")` |
| `.on_hover_text(...)` | `.on_hover_text(i18n::t("expand"))` | `.on_hover_text("Expand all")` |
| 文字列の合成（`format!`等） | `i18n::tf("saved", &[("key", val)])` | `format!("Saved: {}", val)` |

### 11.2 i18n 例外（自動許可リスト）

純粋な記号のみの文字列などは「AST解析器自体が許可リスト（Allowlist）として判定」し、エラーをバイパスする。

- **単一記号**: `"*"`, `"x"`, `"+"`, `"-"`, `"▼"`, `"▶"` など
- **アイコン絵文字**: `"🔄"` などの UI コントロール用単独記号
- **パス区切り・レイアウト空白**: `"/"`, `" "`, `"\n"` など
- **デバッグ文字列**: `tracing::info!` 内など、egui非依存の出力

### 11.3 ロケールファイル管理

- 新しいキーは **en.json と ja.json に同時追加** する（片方だけの追加は禁止）。
- キー漏れは統合テスト（`tests/i18n.rs`等）により自動検知する。

---

## 12. カスタム静的解析 (AST Linter) による規約強制

本コーディング規約（i18nルールや禁止型制約などを含む）を人手ではなくCI上で自動強制するため、Rust の `syn` クレートによる AST（抽象構文木）トラバースを用いたカスタムLinterを運用する。

> ※ 仕様詳細は `docs/ast-linter-plan.md` を参照のこと。

### 12.1 強制のフロー (`pre-commit` / `pre-push`)

開発者がコードを変更してコミットする際、以下のハードゲートを通る。規約違反があった場合、コミット自体が `lefthook` に拒否される。

```text
[コード変更] → [lefthook 検査] → [cargo test (ast_linter.rs)] → [AST 解析・合否]
```
