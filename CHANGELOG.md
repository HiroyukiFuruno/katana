# Changelog

All notable changes to KatanA Desktop will be documented in this file.

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
