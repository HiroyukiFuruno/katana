# ============================================================
# KatanA — Development Makefile
# ============================================================
# Usage: make <target>
#   make help   — Show a list of available commands
# ============================================================

.DEFAULT_GOAL := help

# ---------- Build / Run ----------

.PHONY: build
build: ## Build the entire workspace (debug)
	cargo build --workspace

.PHONY: release
release: ## Release build (optimized)
	cargo build --workspace --release

.PHONY: run
run: ## Run the application (KatanA)
	cargo run --bin KatanA

.PHONY: run-release
run-release: ## Run the application in release mode
	cargo run --bin KatanA --release

APP_NAME     := KatanA Desktop
APP_BUNDLE   := target/release/bundle/osx/$(APP_NAME).app
CONTENTS     := $(APP_BUNDLE)/Contents

.PHONY: package-mac
package-mac: ## Build macOS .app bundle (release)
	cargo bundle --release --format osx --package katana-ui
	@# Overlay project-specific Info.plist (cargo-bundle generates its own)
	cp crates/katana-ui/Info.plist "$(CONTENTS)/Info.plist"
	@# Copy icon into Resources/ (cargo-bundle does not handle icns correctly)
	mkdir -p "$(CONTENTS)/Resources"
	cp assets/icon.icns "$(CONTENTS)/Resources/icon.icns"
	@echo "✅ $(APP_BUNDLE) created"

VERSION      := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
DMG_NAME     := KatanA-Desktop-$(VERSION).dmg
DMG_OUT      := target/release/$(DMG_NAME)

.PHONY: dmg
dmg: package-mac ## Build macOS .dmg installer from .app bundle
	@rm -f "$(DMG_OUT)"
	@if command -v create-dmg >/dev/null 2>&1; then \
		echo "Building DMG with create-dmg..."; \
		create-dmg \
			--volname "KatanA Desktop $(VERSION)" \
			--window-pos 200 120 \
			--window-size 600 400 \
			--icon-size 100 \
			--icon "KatanA Desktop.app" 150 190 \
			--app-drop-link 450 190 \
			--no-internet-enable \
			"$(DMG_OUT)" \
			"$(APP_BUNDLE)"; \
	else \
		echo "create-dmg not found, falling back to hdiutil..."; \
		TMP_DMG=$$(mktemp -d)/staging; \
		mkdir -p "$$TMP_DMG"; \
		cp -R "$(APP_BUNDLE)" "$$TMP_DMG/"; \
		ln -s /Applications "$$TMP_DMG/Applications"; \
		hdiutil create -volname "KatanA Desktop $(VERSION)" \
			-srcfolder "$$TMP_DMG" -ov -format UDZO "$(DMG_OUT)"; \
		rm -rf "$$TMP_DMG"; \
	fi
	@echo "✅ $(DMG_OUT) created"

# ---------- Release ----------

.PHONY: release
release: ## Create a versioned release (usage: make release VERSION=x.y.z)
ifndef VERSION
	$(error VERSION is required. Usage: make release VERSION=x.y.z)
endif
	@echo "🚀 Releasing v$(VERSION)..."
	@# 1. Update workspace version in root Cargo.toml
	sed -i '' 's/^version = ".*"/version = "$(VERSION)"/' Cargo.toml
	@# 2. Update Info.plist version
	sed -i '' '/<key>CFBundleShortVersionString<\/key>/{n;s|<string>.*</string>|<string>v$(VERSION)</string>|}' crates/katana-ui/Info.plist
	@# 3. Generate/update CHANGELOG.md
	git-cliff --tag "v$(VERSION)" --output CHANGELOG.md
	@# 4. Stage and commit
	git add Cargo.toml Cargo.lock crates/*/Cargo.toml crates/katana-ui/Info.plist CHANGELOG.md
	git commit -m "chore: v$(VERSION) リリース準備"
	@# 5. Create annotated tag
	git tag -a "v$(VERSION)" -m "Release v$(VERSION)"
	@echo "✅ Release v$(VERSION) committed and tagged"
	@echo "   Next steps:"
	@echo "     make dmg                  # Build the DMG installer"
	@echo "     git push && git push --tags  # Push to remote"

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

.PHONY: check
check: ## cargo check (type check only, fast)
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
test-integration: ## Run integration tests (UI tests) (requires: egui_kittest)
	cargo test --workspace --test integration

.PHONY: test-update-snapshots
test-update-snapshots: ## Update UI snapshot images (UPDATE_SNAPSHOTS=true)
	UPDATE_SNAPSHOTS=true cargo test --workspace --test integration

# ---------- CI / Quality Gates ----------

COVERAGE_IGNORE := plantuml_renderer\.rs|mermaid_renderer\.rs|katana-ui/src/main\.rs|shell_ui\.rs|preview_pane_ui\.rs

.PHONY: coverage
coverage: ## Run tests and verify 100% test coverage (requires cargo-llvm-cov)
	# The following are temporary exclusions. They should be removed after refactoring away from external command dependencies:
	#   - plantuml_renderer.rs: depends on java -jar -> planned for DI
	#   - mermaid_renderer.rs: depends on mmdc binary -> planned for DI
	# main.rs: Entry point only. Testable logic is exposed and tested via lib.rs
	# The following are egui UI rendering functions only. Business logic separated to shell.rs / shell_logic.rs / preview_pane.rs:
	#   - shell_ui.rs: eframe::App::update + button click event branching + macOS native menu
	#   - preview_pane_ui.rs: drawing sections + render_sections
	#
	# ── Test Execution + Table Report ──
	cargo llvm-cov --workspace --lib --tests \
		--ignore-filename-regex '$(COVERAGE_IGNORE)' \
		-- --test-threads=1
	#
	# ── Gate Check: Verify that all source code lines are executed at least once ──
	#
	# Q: Why use the text output validation instead of the Lines column in the table?
	#
	#   The llvm-cov report table aggregates coverage based on "regions".
	#   Therefore, even if a source code line is executed, if a sub-region
	#   generated by LLVM within that line has a count of 0 (see examples below),
	#   it's treated as a "missed line":
	#
	#     - The None/Err path of the `?` operator (when it's always Some/Ok in the happy path)
	#     - Internally expanded code for iterators in `for` loops
	#     - Short-circuit evaluated paths in `&&` / `||` where the right side is not evaluated
	#     - Error paths of `map_err` closures
	#
	#   This is a known behavior of LLVM's instrumentation, a structural limitation
	#   that cannot be resolved regardless of how the Rust code is written.
	#   (ref: https://github.com/taiki-e/cargo-llvm-cov/issues related issues)
	#
	#   Therefore, the number of lines with "line count = 0" from the text output
	#   is used for the gate. This means the source code line has physically never
	#   been executed, serving as a metric more reflective of reality than the table's Lines column.
	#
	@echo "--- Coverage Gate ---"
	@echo "The Lines column in the table is a reference value due to LLVM sub-region aggregation."
	@echo "Gate Criteria: Target 0 lines with 0 execution count in text output."
	@UNCOV=$$(cargo llvm-cov report \
		--ignore-filename-regex '$(COVERAGE_IGNORE)' \
		--text 2>&1 | grep -c '^ *[0-9]*|  *0|' || true); \
	if [ "$$UNCOV" -ne 0 ]; then \
		echo "❌ FAIL: $$UNCOV lines were never executed"; \
		cargo llvm-cov report \
			--ignore-filename-regex '$(COVERAGE_IGNORE)' \
			--text 2>&1 | grep -B3 '^ *[0-9]*|  *0|'; \
		exit 1; \
	fi; \
	echo "✅ All source code lines executed (Unexecuted lines: 0)"

.PHONY: ci
ci: fmt-check lint test-integration coverage ## Reproduce CI (fmt + clippy + IT + 100% coverage enforced/no relaxations)
	@echo "✅ All checks passed"

.PHONY: pre-push
pre-push: ci ## Pre-push hook equivalent checks

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
