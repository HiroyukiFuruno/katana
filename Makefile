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

# ---------- CI / 品質ゲート ----------

.PHONY: ci
ci: fmt-check lint test ## CI 再現 (fmt + clippy + test の一括実行)
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
