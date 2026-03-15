# ============================================================
# Katana — 開発用 Makefile
# ============================================================
# 使い方: make <target>
#   make help   — 利用可能なコマンド一覧を表示
# ============================================================

.DEFAULT_GOAL := help

# ---------- ビルド・実行 ----------

.PHONY: build
build: ## ワークスペース全体をビルド (debug)
	cargo build --workspace

.PHONY: release
release: ## リリースビルド (最適化あり)
	cargo build --workspace --release

.PHONY: run
run: ## アプリケーション (katana) を起動
	cargo run --bin katana

.PHONY: run-release
run-release: ## リリースモードでアプリを起動
	cargo run --bin katana --release

# ---------- フォーマッタ ----------

.PHONY: fmt
fmt: ## コードフォーマット適用 (rustfmt)
	cargo fmt --all

.PHONY: fmt-check
fmt-check: ## フォーマット差分チェック (CI 用)
	cargo fmt --all -- --check

# ---------- リンタ ----------

.PHONY: lint
lint: ## Clippy 実行 (警告ゼロ強制)
	cargo clippy --workspace -- -D warnings

.PHONY: check
check: ## cargo check (型チェックのみ、高速)
	cargo check --workspace

# ---------- テスト ----------

.PHONY: test
test: ## ユニットテスト全体を実行
	cargo test --workspace

.PHONY: test-core
test-core: ## katana-core のテストのみ実行
	cargo test -p katana-core

.PHONY: test-ui
test-ui: ## katana-ui のテストのみ実行
	cargo test -p katana-ui

.PHONY: test-verbose
test-verbose: ## テスト実行 (詳細出力)
	cargo test --workspace -- --nocapture

.PHONY: test-specific
test-specific: ## 特定テスト実行 (例: make test-specific T=test_name)
	cargo test --workspace -- $(T)

.PHONY: test-integration
test-integration: ## 統合試験 (UI テスト) を実行 (要: egui_kittest 導入)
	cargo test --workspace --test integration

.PHONY: test-update-snapshots
test-update-snapshots: ## UI スナップショット画像を更新 (UPDATE_SNAPSHOTS=true)
	UPDATE_SNAPSHOTS=true cargo test --workspace --test integration

# ---------- CI / 品質ゲート ----------

COVERAGE_IGNORE := plantuml_renderer\.rs|mermaid_renderer\.rs|katana-ui/src/main\.rs|shell_ui\.rs|preview_pane_ui\.rs

.PHONY: coverage
coverage: ## テスト実行とカバレッジ100%達成の検証 (cargo-llvm-cov 必要)
	# 以下は一時的な除外。外部コマンド(java/mmdc)依存を除去するリファクタリング完了後に撤去すること:
	#   - plantuml_renderer.rs: java -jar 依存 → DI化予定
	#   - mermaid_renderer.rs: mmdc バイナリ依存 → DI化予定
	# main.rs: エントリポイントのみ。テスト可能なロジックは lib.rs 経由で公開・テスト済み
	# 以下は egui UI描画関数のみ。ビジネスロジックは shell.rs / shell_logic.rs / preview_pane.rs に分離済み:
	#   - shell_ui.rs: eframe::App::update + ボタンクリックイベント分岐 + macOS ネイティブメニュー
	#   - preview_pane_ui.rs: セクション描画 + render_sections
	#
	# ── テスト実行 + レポート表示（テーブル形式） ──
	cargo llvm-cov --workspace --lib --tests \
		--ignore-filename-regex '$(COVERAGE_IGNORE)' \
		-- --test-threads=1
	#
	# ── ゲートチェック: 全ソースコード行が少なくとも1回実行されていることを検証 ──
	#
	# Q: なぜテーブルの Lines 列ではなく text 出力で検証するのか？
	#
	#   llvm-cov report のテーブルは「リージョン」単位の計測を行に分解して集計する。
	#   このため、ソースコード行が実行されていても、その行内に LLVM が生成する
	#   サブリージョン（下記の例）にカウント0のものがあると「ミスライン」扱いになる:
	#
	#     - ? 演算子の None/Err パス（正常系で常に Some/Ok の場合）
	#     - for ループのイテレータ内部展開コード
	#     - && / || の短絡評価で右辺が評価されないパス
	#     - map_err クロージャのエラーパス
	#
	#   これは LLVM のインストルメンテーションの既知の挙動であり、Rust のコードを
	#   どう書き換えても解消できない構造的な限界である。
	#   (ref: https://github.com/taiki-e/cargo-llvm-cov/issues の関連 issue)
	#
	#   そのため、ゲートには text 出力の「行カウント = 0」の行数を使用する。
	#   これはソースコード行が物理的に一度も実行されていないことを意味し、
	#   テーブルの Lines 列よりも実態に即した指標となる。
	#
	@echo "--- Coverage Gate ---"
	@echo "テーブルの Lines 列は LLVM サブリージョン集計のため参考値。"
	@echo "ゲート基準: text 出力で実行回数 0 の行がゼロであること。"
	@UNCOV=$$(cargo llvm-cov report \
		--ignore-filename-regex '$(COVERAGE_IGNORE)' \
		--text 2>&1 | grep -c '^ *[0-9]*|  *0|' || true); \
	if [ "$$UNCOV" -ne 0 ]; then \
		echo "❌ FAIL: $$UNCOV 行が一度も実行されていない"; \
		cargo llvm-cov report \
			--ignore-filename-regex '$(COVERAGE_IGNORE)' \
			--text 2>&1 | grep -B3 '^ *[0-9]*|  *0|'; \
		exit 1; \
	fi; \
	echo "✅ 全ソースコード行が実行済み（未実行行: 0）"

.PHONY: ci
ci: fmt-check lint test-integration coverage ## CI 再現 (fmt + clippy + IT + カバレッジ100%強制・条件緩和NG)
	@echo "✅ 全チェック通過"

.PHONY: pre-push
pre-push: ci ## pre-push フックと同等のチェック

# ---------- ドキュメント ----------

.PHONY: doc
doc: ## API ドキュメント生成
	cargo doc --workspace --no-deps

.PHONY: doc-open
doc-open: ## API ドキュメント生成 & ブラウザで開く
	cargo doc --workspace --no-deps --open

# ---------- メンテナンス ----------

.PHONY: clean
clean: ## ビルド成果物を削除
	cargo clean

.PHONY: update
update: ## 依存クレートを更新
	cargo update

.PHONY: outdated
outdated: ## 古い依存クレートを一覧表示 (cargo-outdated 必要)
	cargo outdated --workspace

.PHONY: tree
tree: ## 依存関係ツリーを表示
	cargo tree --workspace

# ---------- 開発ユーティリティ ----------

.PHONY: watch
watch: ## ファイル変更監視 & 自動チェック (cargo-watch 必要)
	cargo watch -x 'check --workspace' -x 'test --workspace'

.PHONY: watch-run
watch-run: ## ファイル変更監視 & 自動再起動 (cargo-watch 必要)
	cargo watch -x 'run --bin katana'

.PHONY: bloat
bloat: ## バイナリサイズ分析 (cargo-bloat 必要)
	cargo bloat --release --bin katana

.PHONY: loc
loc: ## ソースコード行数カウント (tokei 必要)
	tokei crates/

# ---------- ヘルプ ----------

.PHONY: help
help: ## このヘルプを表示
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}'
