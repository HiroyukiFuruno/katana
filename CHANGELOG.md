# Changelog

All notable changes to KatanA Desktop will be documented in this file.

## [0.2.1] - 2026-03-21

### 🚀 Features

- Make init command implementation and release automatic push for development environment and flow enhancement

### ♻️ Refactoring

- Rename repository to KatanA, reorganize documents, and support English translation

### 🧪 Testing

- Specify language in settings window integration test to stabilize test
- Collect_matches logic extraction and partial setting screen integration test addition for coverage improvement

### 🔧 Miscellaneous

- Update Rust dependencies and GitHub Actions plugins
- Fix coverage gap in preview_pane and codify release bypass rules
- Resolve V0.2.0 archive omission and add AI warning block to next tasks
- Integrate GitHub Actions CI build command into Makefile target

## [0.2.0] - 2026-03-20

### 🐛 Bug Fixes

- Enforce strict lazy loading and restrict folder auto-expand on Open All
- Abolish redundant filename tooltip and fix ast linter coverage
- Restore missing absolute path in metadata tooltip and apply TDD

### 👷 CI/CD

- Automatically generate and attach SHA256 hashes (checksums.txt) for DMG

### 📚 Documentation

- Add release notes for v0.1.6 to CHANGELOG.ja.md and include in Makefile release target

### 🔧 Miscellaneous

- Refactor RwLock usage and fix external image caching on force reload

### 🚀 Features

- Add workspace persistence and tab restoration logic (Task 1)
- Implement CacheFacade and stabilize all integration tests
- Implement recursive expansion of workspaces and "Open All", and improve usability (Task 3, 5)
- Localize metadata tooltips and apply to file items

## [0.1.6] - 2026-03-19

### 🐛 Bug Fixes

- Automatically inject version into Info.plist during DMG build
- Automatically sync Cargo.lock when running make release

### 📚 Documentation

- Add project-specific skill "release_workflow"

### 🔧 Miscellaneous

- Prepare for v0.1.7 release
- Update Cargo.lock (follow v0.1.7)
- Prepare for v0.1.6 release

### 🚀 Features

- Implement workspace search and filter functionality
- Add internationalized text for search modal Include/Exclude options
- Add inclusion/exclusion filter functionality to search modal and place search button in UI

### 🧪 Testing

- Add integration tests for Include/Exclude options in search filter

## [0.1.5] - 2026-03-19

### ♻️ Refactoring

- Unify HashMap and fixed-length arrays into Vec, and apply collectively including AST rules and migration functionality

### 🔧 Miscellaneous

- Bump version to 0.1.4

### 🚀 Features

- Apply v0.1.5 changes and bump version to 0.1.5

### 🧪 Testing

- Fix tests broken by workspace methods renaming
- Add missing tests to meet 100% coverage gate

## [0.1.4] - 2026-03-19

### 🧪 Testing

- Completely abolish UI image snapshot tests that were causing repository bloat and CI failures, and migrate to semantic assertions

## [0.1.3] - 2026-03-19

### ♻️ Refactoring

- Hierarchize settings.json structure (ThemeSettings/FontSettings/LayoutSettings) and add migration mechanism
- Fix coverage gate and improve code quality

### 🐛 Bug Fixes

- Recovery of missed v0.1.3 version update
- Fix flaky tests where curl failed to start due to environment variable pollution during parallel test execution

### 📚 Documentation

- Migrate markdownlint settings to .vscode management and add v0.1.3 release notes

### 🔧 Miscellaneous

- Exclude snapshot tests and redundant integration test executions from make check

### 🚀 Features

- Expand theme presets from 10 to 30 (added OneDark/TokyoNight/CatppuccinMocha etc.)
- Migrate i18n to type-safe structs (I18nMessages) and add 8 languages (zh-CN/zh-TW/ko/pt/fr/de/es/it)
- Add 8 language tags to macOS native menu and dynamically translate menu strings according to language switching
- Update entire UI for i18n/settings hierarchization, and implement OS language detection, theme expansion, and Show more/less toggle in settings screen

### 🧪 Testing

- Update tests according to i18n type-safety, settings hierarchization, and theme expansion (integration/i18n/theme/diagram_rendering tests)

## [0.1.2] - 2026-03-19

### 🐛 Bug Fixes

- Fix left alignment of workspace file entries
- Fix issue where font size slider becomes invisible in light theme
- Add selection color border to slider to ensure visibility in all themes
- Modify markdown preview tables to use available width
- Fix bugs in table layout and vertical split scroll

### 📚 Documentation

- Add diagram display guide and brew update instructions to README
- Add snapshot prohibition (NG) rule to coding-rules and self-review
- Add brew update instructions to README

### 🔧 Miscellaneous

- Prepare for v0.1.2 release
- Fix flaky view mode integration test by adding ui stabilization steps
- Turn warnings into errors and remove unused code
- Prepare for v0.1.2 release

### 🚀 Features

- Add i18n tooltips to tab navigation and slider

### 🧪 Testing

- Add TDD verification tests for UI bugs and update snapshots

## [0.1.1] - 2026-03-19

### 🐛 Bug Fixes

- Add error handling to Homebrew Cask update step
- Prevent contamination of cached old DMG files
- Add cfg guards to macOS-specific code in emoji.rs (fix Linux CI Lint)

### 🚀 Features

- Support hidden directory display in workspace tree and add directory refresh button

## [0.1.0] - 2026-03-19

### ⏪ Reverted

- Revert release.yml and README to v0.0.3 state
- Undo v0.0.4 changes and revert to v0.0.3 state

### ♻️ Refactoring

- Rename make ci -> make check and create new make check-light
- Move inline tests in os_fonts.rs to tests/ and translate Japanese comments to English

### 🎨 Styling

- Add reason comments to #[allow] attributes (compliant with coding-rules section 10)

### 🐛 Bug Fixes

- Improve .app signing (abolish --deep, specify runtime/timestamp, DMG remains unsigned)
- Fix regression where workspace is not restored on startup
- Apply emoji support patch to vendored egui_commonmark

### 📚 Documentation

- Define versioning policy and refine CI triggers
- Unify indentation of make check command with other commands
- Update coding rules and self-review criteria
- Unify code block language specification (text) in English and Japanese versions
- Append CHANGELOG v0.1.0-dev and fix synchronization for README.ja.md

### 🔧 Miscellaneous

- Add glob filters to pre-push hook and skip CI execution for pushes without code changes
- Prepare for v0.0.4 release
- Exclude fixture tests from make check-light
- Exclude unnecessary old backup images (.old.png) when updating snapshots from Git tracking
- Expand coverage gate exclusion rules (return None/false/display/Pending)
- Prepare for v0.1.0 release (update version number)

### 🚀 Features

- Add Homebrew Cask support
- Implement 10 theme presets and ThemeColors foundation (Task 1) (#23)
- Implement foundation for font size and family settings (Task 2)
- Implement theme linking and settings screen, and update snapshots (WIP)
- Add dynamic acquisition of OS fonts and reflection in UI
- Implement Task 4: editor/preview layout settings
- Implement Task 5: OS theme linking (initial default auto-selection)
- Implement Task 6: font setting expansion (search function + Apple Color Emoji)
- Add strict quality checks to linter (prohibit use of todo! macro, etc.)
- Improve UI functions such as font search, emoji support, and preview
- Implement emoji inline rendering foundation and separate SVG/HTTP cache loaders
- Add lazy code detection tests and #[cfg(test)] module exclusion to AST Linter

### 🧪 Testing

- Add tests for coverage improvement

## [0.0.3] - 2026-03-18

### ♻️ Refactoring

- Constantize magic numbers and expand AST linter tests
- Unify Ignore tags to limited_local

### 🐛 Bug Fixes

- Unify Coverage job with local make coverage
- Improve DrawIo diagram text visibility in dark theme
- Expand mmdc resolution from .app bundle to 6-layer fallback
- Skip diagram snapshot tests in CI environment
- Fix startup from GUI apps by supplementing node PATH when executing mmdc
- Add margins above and below HTML blocks to resolve layout tightness

### 📚 Documentation

- Change version fixed section in README to dynamic status notation

### 🔧 Miscellaneous

- Update coverage exclusion reasons to accurate technical grounds
- Prepare for v0.0.3 release
- Change release notes to be extracted from CHANGELOG.md

### 🧪 Testing

- Fix CI environment dependent errors in snapshot tests
- Fix global state conflict errors in multiple i18n tests
- Add integration tests for diagram rendering and sample fixtures

## [0.0.2] - 2026-03-17

### 🐛 Bug Fixes

- Resolve linux cross-compilation errors for github actions
- Resolve markdown rendering, i18n label update, and CI coverage flakiness
- Support CenteredMarkdown for raw HTML alignment reproduction
- Fix CenteredMarkdown alignment, image path resolution, and badge display

### 📚 Documentation

- Restore xattr command procedure for initial startup

### 🔧 Miscellaneous

- Kick ci to retry integration tests
- Release v0.0.2
- Include Cargo.lock and CHANGELOG.ja.md in release v0.0.2

### 🧪 Testing

- Update integration test snapshot
- Increase snapshot tolerance to 4000 to absorb CI/local macOS text rendering differences

## [0.0.1] - 2026-03-16

### ⏪ Reverted

- Retract sccache, maintain only cache path optimization

### ♻️ Refactoring

- Fix clippy warnings in drawio_renderer
- Migrate tests from src/ to tests/ directory and tighten Clippy
- Refactor katana-ui into lib/binary structure and extract logic
- Extract magic numbers into named constants with clear purpose
- Externalize language definitions to locales/languages.json
- Unify span_location duplication into free functions (self-review fix)
- Separate egui rendering logic and event routing
- Translate Japanese comments and strings in source code and tests to English
- Improve UI layout and add linter module

### ⚡ Performance

- Introduce sccache and cache optimization into CI/CD

### 🐛 Bug Fixes

- Fix Clippy warnings, formatting, and 30-line limit
- Fix issues confirmed via screenshots
- Stabilize tests by making PLANTUML_JAR an exclusive override
- Fix 3 issues — lazy loading, Mermaid fonts, and forced desktop move
- Fix flaky issues in snapshot tests
- De-indent code blocks in lists during preprocessing to avoid egui layout constraints
- Change Info.plist update to Perl for macOS sed compatibility
- Add ad-hoc code signing to Release CD
- Fix CI trigger branch name to master and update Cargo.lock
- Fix sccache-action SHA and organize English/Japanese versions of CHANGELOG
- Temporarily disable sccache during cargo install (conflict avoidance)
- Disable RUSTC_WRAPPER in CI Lint job (clippy compatibility)

### 📚 Documentation

- Mark test compilation, tab UI, and plantuml macos bug as done
- Add i18n conventions (Section 11) to coding-rules
- Add README and document templates, change .obsidian to gitignored
- Add project foundation files — LICENSE (MIT), README, development environment setup script
- Add ADR (Architecture Decision Records) and integration test scenarios
- Add technical debt memo (TECHNICAL_DEBT.md)
- Add OpenSpec for organize-documents
- English translation of common documents, parallel maintenance of Japanese versions (*.ja.md), and OpenSpec archiving
- Unify project name to KatanA (README, Cargo.toml, setting comments)
- Restructure documents for general distribution (#21)
- Add "What is KatanA" section (English/Japanese)
- Append origin of "A" as Agent in KatanA

### 🔧 Miscellaneous

- Bootstrap katana repository
- Remove opsx prompt files
- Align gitignore with official templates
- Mark Task 6.2 as completed — bootstrap-katana-macos-mvp all tasks completed
- Exclude openspec directory from git control
- Update gitignore (openspec, obsidian settings, katana-core .gitignore integration)
- Delete unnecessary document templates and README
- Add CI coverage job and document quality gates
- Tighten CI requirements for desktop-viewer-polish and delete unnecessary assets
- Integrate lefthook validation commands into Makefile and automate fixes
- Update dependencies (dirs-sys 0.5.0, rfd 0.17.2, egui_commonmark features added)
- Add GitHub Sponsors URL settings and Japanese version of README
- Add configuration to exclude CI bot commits from cliff.toml
- Prepare for v0.0.1 release

### 🚀 Features

- Bootstrap Katana macOS MVP — implementation of Rust project foundation and all core modules
- Task 3.2 — native Markdown preview pane implementation
- I18n support, language setting, appAction expansion, bin rename
- Improve diagram rendering (Draw.io arrow support, Mermaid PNG output, CommandNotFound distinction)
- Extend filesystem service (workspace tree and file monitoring improvements)
- Tab-specific preview management, scroll synchronization, macOS native menu, workspace panel control
- Enhance verification — introduction of lefthook, adding tests, tightening Clippy, defining quality gates
- Introduce AST Linter (katana-linter) — detection of i18n hardcoded strings and magic numbers
- Apply Katana app icon and version for native About panel (#15)
- Implement settings persistence foundation (JsonFileRepository + SettingsService)
- Auto-save settings when workspace or language changes
- Restore saved settings (workspace, language) on startup
- Improve preview functionality (image path resolution, section splitting starting fence support, diagram renderer improvements)
- Improve About screen and unify app display name to KatanA
- Add macOS app bundle (.app) packaging (#18)
- Add macOS DMG installer generation (#19)
- Automate release (git-cliff + make release) (#20)
- Create new Release CD workflow (.github/workflows/release.yml) (#22)
- Add GitHub Sponsors URL settings and Japanese version of README

### 🧪 Testing

- Task 6.2 — add preview synchronization tests
- Add app state unit tests and fix java headless mode for plantuml
- Add unit tests for preview synchronization (Task 3.2 completed)
- Tighten coverage — removed ignore-filename-regex, abolished #[coverage(off)], enforced 100% Regions
- Address differences in LLVM coverage calculation and tighten 100% test gate
- Add integration tests for persistence round-trip
