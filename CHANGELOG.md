# Changelog

All notable changes to KatanA Desktop will be documented in this file.

## [0.0.4] - 2026-03-18

CI improvements and workflow optimizations.

### ✨ Improvements

- CI now mirrors the local `make coverage` gate with full exclusion rules, replacing the simple `--fail-under-lines 100` flag
- CI workflow now skips on documentation-only changes (`paths-ignore` for `**.md`, `docs/**`, etc.)

## [0.0.3] - 2026-03-18

Bug fixes for diagram rendering readability and .app bundle compatibility, plus testing improvements.

### 🐛 Bug Fixes

- Fixed DrawIo diagram text being unreadable in dark theme due to insufficient contrast — introduced `drawio_label_color` in presets with proper fallback
- Fixed `mmdc` (Mermaid CLI) not being found when launched from .dmg/.app bundle on systems with nvm lazy-loading or non-standard PATH configurations
- Fixed CI snapshot test failures caused by environment-dependent rendering differences
- Fixed i18n test flakiness caused by global state contention between parallel tests

### ✨ Improvements

- Expanded `mmdc` binary resolution to a 6-tier fallback chain: `MERMAID_MMDC` env var → Homebrew (`/opt/homebrew/bin`, `/usr/local/bin`) → nvm/volta/fnm direct filesystem probe → `which` → `/bin/zsh -l -c` login shell → bare `mmdc`, with `OnceLock` process-wide caching for sub-millisecond subsequent lookups
- Refined coverage exclusion rationale — replaced vague "planned for DI" comments with precise technical reasons (egui frame context dependency, OnceLock cache behavior)
- Extracted magic numbers into named constants (`CHANNEL_MAX`, `LUMA_R/G/B`, `RENDER_POLL_INTERVAL_MS`) for improved readability

### 🧪 Testing

- Added diagram rendering snapshot tests covering all diagram types (Mermaid, PlantUML, DrawIo) and fallback states (CommandNotFound, RenderError, Pending)
- Added sample fixture integration tests for EN/JA Markdown documents with HTML, badges, and diagrams
- Added AST linter test for `#[cfg(test)]` impl method skip logic
- Added `drawio_label_color` and `relative_luminance` unit tests in color_preset module

## [0.0.2] - 2026-03-17

Second release focusing on HTML rendering fidelity, test coverage, and documentation.

### 🚀 Features

- Native support for `align="center"` HTML attributes within Markdown (e.g., `<p align="center">`, `<h1 align="center">`)

### 🐛 Bug Fixes

- Fixed unreachable code paths in UI pane components to achieve 100% LLVM coverage
- Resolved rendering bugs where multi-element centered blocks (like README badges and links) collided horizontally due to egui layout ID reuse
- Fixed image resolution paths and Markdown processing for standalone inline tags
- Resolved Linux cross-compilation errors for GitHub Actions

### 📚 Documentation

- Added explicit TDD (Test-Driven Development) enforcement policies to coding rules (EN/JA)
- Restored `xattr` command instructions for initial launch on macOS

### 🧪 Testing

- Expanded integration tests with precision bounds assertions to prevent UI layout regressions
- Increased image snapshot tolerance to absorb CI/local macOS text rendering differences

### 🔧 Miscellaneous

- Disabled `sccache` to prevent cross-compilation cache issues while keeping workflow cache optimizations
- Refined CI pipeline for better stability on snapshot tests

## [0.0.1] - 2026-03-16

First public release of KatanA Desktop 🎉

### 🚀 Features

- Rust + egui based native macOS Markdown workspace
- Live split-view preview with scroll synchronization
- Diagram rendering (Mermaid, PlantUML, Draw.io)
- GitHub Flavored Markdown support (tables, strikethrough, task lists, footnotes)
- Workspace-aware file tree navigation
- Tab bar for multi-document editing
- i18n support (English & Japanese)
- Settings persistence (workspace, language)
- macOS .app bundle and .dmg installer
- Automated release pipeline with changelog generation
- Ad-hoc code signing for smooth installation
- AST Linter for coding standards enforcement

### ♻️ Refactoring

- Separated egui rendering logic from event routing
- Extracted magic numbers into named constants
- Migrated tests from src/ to tests/ directory
- Externalized language definitions to locales/languages.json
- Translated source code comments and strings to English

### 🐛 Bug Fixes

- Fixed code block rendering inside lists (pre-processing de-indent)
- Fixed flaky snapshot tests
- Fixed lazy loading, Mermaid font, and desktop forced-move issues
- macOS sed compatibility fix for Info.plist updates

### 📚 Documentation

- User-facing README with installation guide (EN/JA)
- "What is KatanA" section explaining the name origin (Katana × Agent)
- Developer guide moved to docs/
- Architecture Decision Records (ADR)
- Coding rules and i18n conventions
- GitHub Sponsors integration

### 🧪 Testing

- 100% line coverage gate (cargo-llvm-cov)
- Integration tests for preview synchronization
- Settings persistence round-trip tests
- CodeQL security scanning

### 🔧 Miscellaneous

- CI/CD pipeline (GitHub Actions) with sccache optimization
- Release automation with git-cliff
- lefthook pre-commit/pre-push hooks
- FUNDING.yml for GitHub Sponsors
