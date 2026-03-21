# ============================================================
# KatanA — Development Makefile
# ============================================================
# Usage: make <target>
#   make help   — Show a list of available commands
# ============================================================

.DEFAULT_GOAL := help

# Force all warnings to be treated as errors for every cargo command run via make
export RUSTFLAGS=-D warnings

.PHONY: init
init: ## Bootstrap the development environment interactively
	scripts/setup.sh

# ---------- Build / Run ----------

.PHONY: build
build: ## Build the entire workspace (debug)
	cargo build --workspace

.PHONY: build-release
build-release: ## Release build (optimized)
	cargo build --workspace --release

.PHONY: run
run: build ## Run the application (KatanA)
	cargo run --bin KatanA

.PHONY: run-release
run-release: ## Run the application in release mode
	cargo run --bin KatanA --release

VERSION      := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')

.PHONY: package-mac
package-mac: ## Build macOS .app bundle (release)
	scripts/package-mac.sh $(VERSION)

.PHONY: dmg
dmg: package-mac ## Build macOS .dmg installer from .app bundle
	@FORCE=$(FORCE) scripts/dmg.sh $(VERSION)

# ---------- Release ----------

.PHONY: release
release: ## Create a versioned release (usage: make release VERSION=x.y.z USE_GITHUB_WORKFLOW=1 FORCE=1)
ifndef VERSION
	$(error VERSION is required. Usage: make release VERSION=x.y.z)
endif
	@USE_GITHUB_WORKFLOW=$(USE_GITHUB_WORKFLOW) FORCE=$(FORCE) scripts/release/release.sh $(VERSION)

# ---------- Formatters ----------

.PHONY: fmt
fmt: ## Apply code formatting (rustfmt)
	cargo fmt --all

.PHONY: fmt-check
fmt-check: ## Check format differences (for CI)
	cargo fmt --all -- --check

# ---------- Linters ----------

.PHONY: lint
lint: ## Run Clippy (forces zero warnings)
	cargo clippy --workspace -- -D warnings

.PHONY: lint-fix
lint-fix: ## Run Clippy and apply automatic fixes
	cargo clippy --workspace --fix --allow-dirty --allow-staged -- -D warnings

.PHONY: typecheck
typecheck: ## cargo check (type check only, fast)
	cargo check --workspace

# ---------- Testing ----------

.PHONY: test
test: ## Run all unit tests
	cargo test --workspace

.PHONY: test-core
test-core: ## Run tests for katana-core only
	cargo test -p katana-core

.PHONY: test-ui
test-ui: ## Run tests for katana-ui only
	cargo test -p katana-ui

.PHONY: test-verbose
test-verbose: ## Run tests with verbose output
	cargo test --workspace -- --nocapture

.PHONY: test-specific
test-specific: ## Run a specific test (e.g., make test-specific T=test_name)
	cargo test --workspace -- $(T)

.PHONY: test-integration
test-integration: ## Run integration tests (UI tests, semantic assertions only) (requires: egui_kittest)
	cargo test --workspace --test integration

# ---------- CI / Quality Gates ----------

.PHONY: coverage
coverage: ## Run tests and verify 100% test coverage (requires cargo-llvm-cov)
	scripts/coverage.sh

.PHONY: check
check: fmt-check lint test-integration coverage ## Full verification (fmt + clippy + IT + 100% coverage enforced)
	@echo "✅ All checks passed"

check-local: fmt lint test-integration coverage ## Full verification (fmt + clippy + IT + 100% coverage enforced)
	@echo "✅ All checks passed"

.PHONY: check-light
check-light: fmt-check lint ## Quick verification (skip slow fixture tests)
	cargo test --workspace -- --skip fixture
	@echo "✅ Light checks passed"

.PHONY: pre-push
pre-push: check ## Pre-push hook equivalent checks

# ---------- Documentation ----------

.PHONY: doc
doc: ## Generate API documentation
	cargo doc --workspace --no-deps

.PHONY: doc-open
doc-open: ## Generate & open API documentation in browser
	cargo doc --workspace --no-deps --open

# ---------- Maintenance ----------

.PHONY: clean
clean: ## Remove build artifacts
	cargo clean

.PHONY: update
update: ## Update dependency crates
	cargo update

.PHONY: outdated
outdated: ## List outdated dependencies (requires cargo-outdated)
	cargo outdated --workspace

.PHONY: tree
tree: ## Display dependency tree
	cargo tree --workspace

# ---------- Development Utilities ----------

.PHONY: watch
watch: ## Watch file changes & auto check (requires cargo-watch)
	cargo watch -x 'check --workspace' -x 'test --workspace'

.PHONY: watch-run
watch-run: ## Watch file changes & auto restart (requires cargo-watch)
	cargo watch -x 'run --bin KatanA'

.PHONY: bloat
bloat: ## Binary size analysis (requires cargo-bloat)
	cargo bloat --release --bin KatanA

.PHONY: loc
loc: ## Count lines of code (requires tokei)
	tokei crates/

# ---------- Help ----------

.PHONY: help
help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}'
