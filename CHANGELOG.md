# Changelog

All notable changes to KatanA Desktop will be documented in this file.

## [0.1.3] - 2026-03-20

A major update revamping the i18n (internationalization) system, supporting 10 languages globally and fully integrating with macOS native menus. It also introduces hierarchical settings and substantially expands theme presets.

### ✨ Features

- **Massive i18n Expansion**: Added built-in support for 10 languages including Traditional/Simplified Chinese, Korean, Portuguese, French, German, Spanish, and Italian alongside English and Japanese.
- **macOS Native Integration**: Added automatic system language detection and application on the first launch. Also, dynamic FFI integration ensures the OS native menu bar (e.g., File, Settings) translates correctly when switching languages in-app.
- **Enhanced Theme Presets**: Added 20 new theme presets (Dark/Light) bringing the total to 30. Implemented a "Show more/less" toggle UI to cleanly manage the expanded list.
- **Type-Safe i18n Architecture**: Deprecated string-based dictionary lookups in favor of Serde object-mapped `I18nMessages`, ensuring structural completeness and preventing missing keys across languages via test and compile-time checks.
- **Hierarchical Settings and Migration Framework**: Refactored the flat `settings.json` structure into a nested configuration. Includes a built-in backward-compatibility runner that automatically and securely migrates configurations from v0.1.2.

### 🐛 Bug Fixes

- **CRITICAL**: Resolved a side-effect bug where parallel UI integration tests overwrote and corrupted the host's actual profile settings (`~/Library/Application Support/KatanA/settings.json`). Tests are now safely isolated in temporary directories.
- Removed arbitrary hardcoded magic numbers within the `settings_window.rs` UI layout calculations.

### 🔧 Miscellaneous

- Eradicated inline Markdown Lint (`MD024`) suppressions from documents like `CHANGELOG.md`, offloading management into `.vscode/settings.json` configuration blocks.

## [0.1.2] - 2026-03-19

### 🐛 Bug Fixes

- **Workspace File Entry Alignment**: Fixed an issue where the file entry labels in the workspace panel were right-aligned (stretched to full width) instead of being correctly left-aligned.
- **Font Size Slider Visibility**: Improved the contrast of the slider rail on dark themes and applied styling using the accent color.

### ✨ Features

- **Tab Navigation Tooltips**: Added i18n tooltips to the ◀/▶ tab navigation buttons ("Move to previous tab", "Move to next tab").
- **Font Slider Hover Hint**: Added an i18n tooltip to the font size slider explaining the drag operation.

### 📚 Documentation

- Added a guide for **Preparing for Diagram Rendering** (installation instructions for Mermaid CLI and PlantUML) to the README (English/Japanese).

## [0.1.1] - 2026-03-19

### ✨ Features

- **Hidden Directory Display**: Updated the workspace file tree to display hidden directories (such as `.github`, `.vscode`, etc.) including their `.md` files.
- **Workspace Refresh Button**: Added a 🔄 button to the workspace panel toolbar to allow reloading the directory tree from the disk.

## [0.1.0] - 2026-03-19

A major update that introduces a dedicated settings window, allowing free customization of themes, fonts, and layouts, significantly improving UI/UX.

### ✨ Features

- **10 Theme Presets and Color Customization**: Built-in 10 famous dark/light themes (Katana-Dark, Dracula, Nord, etc.) with support for overriding custom colors.
- **OS Theme Sync**: Automatically selects the optimal theme based on macOS system settings (Dark/Light mode) on first launch.
- **Enhanced Font Settings**: Customizable font family and size (8px to 32px) for both the editor and preview panes.
- **Incremental Search**: Added a real-time filtering UI to quickly find specific fonts from the font list.
- **Apple Color Emoji Support**: Ensured emojis in the preview are correctly rendered on macOS environments.
- **Dedicated Settings UI**: Added an overlay "Settings Window" to preview and save changes in real time.
- **Flexible Layouts**: Allows responsive 50:50 window splitting with interchangeable vertical/horizontal directions and editor/preview pane ordering.

### 🐛 Bug Fixes

- Fixed a regression where the previous workspace state was not correctly restored upon application restart.
- Fixed an issue where `.app` bundle execution failed to find Mermaid (`mmdc`) due to missing Node PATHs.
- Adjusted layout margins for headings and HTML blocks to resolve cramped spacing issues.

### 🔧 Miscellaneous

- Added seamless installation and update support via Homebrew (`brew install --cask`).
- Enforced strict quality check rules (e.g., forbidding `todo!` macros) in the Linter and updated coding standards.

## [0.0.3] - 2026-03-18

A release focused on diagram rendering visibility, .app bundle compatibility improvements, and test expansion.

### 🐛 Bug Fixes

- Fixed illegible DrawIo diagram text in dark themes due to insufficient contrast by introducing `drawio_label_color` to presets as an appropriate fallback.
- Fixed an issue where `mmdc` (Mermaid CLI) was not found when launching from .dmg/.app bunches due to `nvm` lazy loading or non-standard PATHs.
- Fixed CI snapshot test failures caused by environment-dependent rendering differences.
- Fixed instable i18n tests caused by global state race conditions during parallel test execution.

### ✨ Improvements

- Expanded `mmdc` binary resolution to a 6-layer fallback chain: `MERMAID_MMDC` env variable → Homebrew (`/opt/homebrew/bin`, `/usr/local/bin`) → nvm/volta/fnm direct filesystem search → `which` → `/bin/zsh -l -c` login shell → bare `mmdc` (with sub-millisecond subsequent lookups via `OnceLock` process caching).
- Clarified coverage exclusion reasoning—replaced vague "DI planned" comments with concrete technical justifications (e.g., egui frame context dependency, OnceLock caching behavior).
- Extracted magic numbers into named constants (`CHANNEL_MAX`, `LUMA_R/G/B`, `RENDER_POLL_INTERVAL_MS`) to improve readability.

### 🧪 Testing

- Added snapshot tests covering all diagram types (Mermaid, PlantUML, DrawIo) and fallback states (CommandNotFound, RenderError, Pending).
- Added sample fixture integration tests for EN/JA Markdown documents (including HTML, badges, and diagrams).
- Added tests for the `#[cfg(test)]` impl method skipping logic in the AST Linter.
- Added unit tests for `drawio_label_color` and `relative_luminance` in the color_preset module.

## [0.0.2] - 2026-03-17

A second release focused on improving HTML rendering fidelity, maintaining test coverage, and expanding documentation.

### 🚀 Features

- Native support for the `align="center"` HTML attribute within Markdown (e.g., `<p align="center">`, `<h1 align="center">`).

### 🐛 Bug Fixes

- Fixed unreachable code in UI pane components, achieving 100% LLVM coverage.
- Resolved an issue with horizontally colliding center-aligned blocks (like README badges and links) caused by egui layout ID reuse.
- Fixed Markdown processing bugs regarding image path resolution and isolated inline tags.
- Resolved Linux cross-compilation errors on GitHub Actions.

### 📚 Documentation

- Explicitly noted mandatory TDD (Test-Driven Development) processes in the coding guidelines (English/Japanese).
- Restored the `xattr` command procedure for the first launch.

### 🧪 Testing

- Added pixel-perfect boundary verification to integration tests to prevent UI layout regressions.
- Increased image snapshot tolerance to absorb text rendering differences between the CI environment and local macOS.

### 🔧 Miscellaneous

- Disabled `sccache` to prevent caching-related errors during cross-compilation (while maintaining workflow-level caching).
- Optimized CI pipelines to improve the stability of snapshot tests.

## [0.0.1] - 2026-03-16

First public release of KatanA Desktop 🎉

### 🚀 Features

- Native macOS Markdown workspace built with Rust + egui
- Live split-view preview with synchronized scrolling
- Diagram rendering (Mermaid, PlantUML, Draw.io)
- GitHub Flavored Markdown support (tables, strikethrough, task lists, footnotes)
- Workspace-aware file tree navigation
- Multi-document aware tab bar
- Internationalization support (English/Japanese)
- Settings persistence (workspace, language)
- macOS .app bundle and .dmg installer
- Release pipeline with automated changelog generation
- Ad-hoc code signing for smooth installation
- AST Linter for coding standards compliance

### ♻️ Refactoring

- Separated egui drawing logic and event routing
- Extracted magic numbers into named constants
- Migrated tests from src/ to tests/ directories
- Externalized language definitions to locales/languages.json
- Localized source code comments and strings to English

### 🐛 Bug Fixes

- Fixed code block display inside lists (preprocessing de-indentation)
- Fixed flakiness in snapshot tests
- Fixed lazy loading, Mermaid fonts, and forced desktop layout issues
- Altered Info.plist update method for macOS sed compatibility

### 📚 Documentation

- User-facing README with installation guides (English/Japanese)
- "What is KatanA" section - naming origin (Katana × Agent)
- Moved developer guides to docs/
- Architecture Decision Records (ADR)
- Coding standards and i18n conventions
- GitHub Sponsors integration

### 🧪 Testing

- 100% line coverage gate (cargo-llvm-cov)
- Integration tests for preview synchronization
- Roundtrip tests for settings persistence
- CodeQL security scans

### 🔧 Miscellaneous

- CI/CD pipelines (GitHub Actions) with sccache optimization
- Release automation via git-cliff
- Lefthook pre-commit/pre-push hooks
- FUNDING.yml for GitHub Sponsors
