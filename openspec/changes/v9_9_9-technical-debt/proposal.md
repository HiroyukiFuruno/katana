# 技術的負債・将来改善メモ

> 発見した技術的負債を記録する。口約束で放置しない。

## クレート構成

- [ ] **katana-common クレートの検討**: 現在 `katana-core` がドメインロジックと共通基盤を兼務。i18n、エラー型、設定管理などクロスカッティングな関心事が複数クレートにまたがる兆候が出た時点で `katana-common` への分離を実施する。
  - 発見契機: AST Linter をワークスペース共通化する際、共通クレートが存在しない構造的な違和感が指摘された。

## i18n 設計

- [ ] **ロケール JSON の読み込み方式の拡張性**: 現在 `include_str!` で en/ja を静的埋め込み。言語追加時に `i18n.rs` の `get_dictionary()` に1行追加が必要。`languages.json` のリストから動的にロケールファイルを読み込む方式にすれば、Rust コードの変更が完全に不要になる。

## CI / カバレッジ

- [ ] **CI coverage ゲートの厳格化**: 現在 CI の coverage ジョブは `make coverage` を呼び出しており、GUI 描画系・外部ツール依存ファイル（`shell_ui.rs`, `preview_pane_ui.rs`, `html_renderer.rs`, `main.rs`, `mermaid_renderer.rs`, `plantuml_renderer.rs`）を除外してゲートしている。本来は除外に頼らず、以下の方向でカバレッジギャップを埋めるべき：
  - GUI 描画関数のビジネスロジックをさらに薄い描画シェルから分離し、テスト可能にする
  - 外部ツール依存（mermaid, plantuml）を DI 化してモック可能にする
  - `main.rs` のロジックを lib.rs 側に移動し、エントリーポイントを最小限にする
  - 最終目標: 除外パターンなしの `cargo llvm-cov --workspace --fail-under-lines 100` でパスすること
  - 発見契機: CI に `--fail-under-lines 100` を設定したが 92.29% で失敗。応急処置として `make coverage` に統一（2026-03-17）

- [ ] **coverage 実行の再現性とゲート妥当性の安定化**: 2026-03-21 のリポジトリ品質評価では、最初の `make check-local` 実行で `cargo llvm-cov` 配下の `ast_linter` テストバイナリ起動が `No such file or directory (os error 2)` で失敗した一方、同じコマンドの再実行は通過した。また `cargo llvm-cov report --text` の現在の除外条件でも 0 カウント行が 4870 行残っており、`Makefile` が意図する「all meaningful lines executed」と計測結果の整合が取れていない。
  - 対応案: `cargo llvm-cov clean --workspace` を含めた実行手順の固定化、target-dir の衝突回避、`cargo-llvm-cov` バージョン固定、ゲート用コマンドとレポート用コマンドの差分解消を行う。
  - 発見契機: repository 全体の品質評価で `make check-local`, `cargo llvm-cov --workspace --lib --tests`, `cargo llvm-cov report --text` を突き合わせた際に発覚（2026-03-21）

- [x] **`make check-local` → `make check-local` リネームと `make check-local` 追加**: ~~`make check-local` は CI/CD パイプラインでは使用されておらず、ローカルの事前検証ターゲットとして使われている。~~ `make check-local` にリネーム済。カバレッジを除いた軽量版 `make check-local`（fmt + clippy + test のみ）を新設済。
  - 発見契機: フォント設定実装中のセルフレビューで `make check-local` の実行時間が長いことが指摘された（2026-03-18）

## コメント言語

- [ ] **既存ソースコード内の日本語コメントの英語化**: 複数のソースファイル（`shell.rs`, `shell_ui.rs`, `settings.rs`, `preview_pane_ui.rs` 等）に日本語コメントが残存している。プロジェクト規約では `_ja` サフィックスなしの Git 管理ファイルは英語で記述する。
  - 発見契機: テーマ設定実装時のセルフレビューで差分内に日本語コメントが混在していることを検出（2026-03-18）

## ルール運用

- [ ] **`coding-rules.md` と `coding-rules.ja.md` の規約差分解消**: 英語版では「コメントは英語」「テスト名は英語 snake_case」と定義している一方、日本語版では「コメントは日本語」「テスト名は日本語または英語」となっており、同一規約の英日版が矛盾している。どちらを正とするかを決め、両文書と関連する README / self-review 手順を同期させる必要がある。
  - 発見契機: repository 全体の品質評価で `docs/coding-rules.md` と `docs/coding-rules.ja.md` を参照した際に、コメント規約とテスト命名規約の差分を検出（2026-03-21）

- [ ] **`unwrap` 禁止ルールと実装・lint 強制の乖離解消**: ルール文書では `deny(clippy::unwrap_used)` を前提にしているが、各 crate ルートは `#![deny(warnings)]` のみで、実コードにも `RwLock` や辞書初期化の `unwrap()` が残っている（例: `katana-platform/src/cache.rs`, `katana-ui/src/i18n.rs`）。規約を維持するなら crate 単位で lint を有効化し、許容例外を明文化した上で段階的に除去するべき。
  - 発見契機: repository 全体の品質評価で規約文書、crate ルート、実コードを照合した際に検出（2026-03-21）

## クレート分離

- [ ] **katana-types クレートの検討**: `Rgb` / `Rgba` 等の汎用値オブジェクトが現在 `katana-platform::theme` に同居している。クレート数が増えた場合（例: `katana-renderer` の分離）、複数クレートが `katana-platform` 全体を依存することになるため、`katana-types` クレートへの分離を検討すべき。
  - 発見契機: テーマ設定の `[u8; 3]` → `Rgb` struct リファクタリング時に、色型の配置先について議論が発生（2026-03-18）

## Markdown プレビュー・設定

- [ ] **コードブロック背景色が自動上書きされる問題**: `egui_commonmark` と `syntect` の制約により、シンタックスハイライトテーマ（`.tmTheme`）に定義された背景色が強制的に使用され、Katana側で設定した `code_background` が無視される。
  - カスタ定義ファイルから背景色の記述を削り注入するハックを試みたが、XMLパーサーの崩壊や将来的な破壊リスクが強いため却下とした。今後は `egui_commonmark` 本家への機能追加PR等で解決を図る。
  - 発見契機: カスタムテーマ設定・プレビューカラー修正のタスクにて（2026-03-18）

- [ ] **カスタマイズしたテーマの永続化と名前付き保存**: 現在、ユーザーが任意の色変更を行った状態（Custom テーマ）は設定として反映・自動保存されるが、「名前をつけて複数保存し、ドロップダウンから選択できる」といった高度なカスタムテーマ管理機能がない。
  - よりリッチなテーマカスタマイズ体験を実現するため、将来的な拡張要件として劣後。
  - 発見契機: カスタムテーマ設定・プレビューカラー修正のタスクにて（2026-03-18）
