# Changelog

All notable changes to KatanA Desktop. This file records the major changes to KatanA Desktop.

## [0.7.8] - 2026-03-27 13:03:25 (UTC)

### 🚀 Features

- **Theme Contrast Offset**: Introduced a `ui_contrast_offset` slider in the Appearance settings, allowing fine-grained control over UI contrast limits.
- **Cache Management**: Added a dedicated 'Clear HTTP Cache' button within the System settings tab for on-demand cache directory purging.

### 🎨 UI/UX

- **Settings Layout Alignment**: Vertically centered and right-aligned color swatches in the Custom Themes grid, improving visual consistency and scanability across all settings panes.

### 🔧 Miscellaneous

- **Internal**: Enforced absolute source code sanitization across the entire crate workspace by strictly prohibiting all Japanese comments and unescaped characters outside of locale definitions.
- **Internal**: Upgraded the generic AST Linter infrastructure with `ignore::WalkBuilder` for parallelized recursive directory scanning, significantly improving CI inspection speed.
- **Ops**: Restructured OpenSpec workflow documents and configuration models within the repository layout.


## [0.7.7] - 2026-03-27 08:30:00 (UTC)

### 🎨 UI/UX

- **Theme Contrast Baseline**: Standardize code and preview hover/current-line background colors to an alpha baseline of 50 across all 30+ Dark themes for optimal visibility.
- **Preview Rendering**: Fix additive blending (white-out) issues when rendering semi-transparent markdown highlights by transitioning to unmultiplied RGBA structures in the egui_commonmark pipeline.
- **Color Settings Architecture**: Overhaul the theme settings architecture by introducing `ColorSettingDef` schemas, resolving rigid hardcoded color mappings and dynamically exposing multi-layered themes (System/Code/Preview) directly to the UI.

### 🐛 Bug Fixes

- **Image Cache Directory Deletion**: Resolve severe macOS filesystem conflicts (`os error 66`) during HTTP cache clearances. Replaced the error-prone root directory deletion with safe, recursive internal content deletion strategies.

### 🔧 Miscellaneous

- **Internal**: Introduce the `ast_linter_no_hardcoded_colors` strict AST Lint rule within `katana-linter` to entirely eradicate unmanageable hardcoded coloration from the UI codebase, maintaining 100% test coverage limits.
- **Ops**: Bake OpenSpec change auto-archival into the official pipeline for seamless transitions.

## [0.7.6] - 2026-03-26 21:20:00 (UTC)

### 🚀 Features

- **Behavior Settings**: Provide granular control to adjust the auto-save interval with 0.1-second precision.
- **Workspace Settings**: Implement toggles for hiding file extensions and add a fully functioning list-style UI for managing scan exclusion paths easily.
- **Custom Theme Management**: Support complete lifecycle actions for custom themes, including named saving, duplication, and explicit deletion.

### 🎨 UI/UX

- **ComboBox Standardization**: Deprecate raw `egui::ComboBox` usage in favor of the newly introduced, unified `StyledComboBox` component globally.
- **Preview Sync Configuration**: Transition scroll synchronization configuration to an intuitive toggle switch and streamline layout order.
- **Theme Builder Polish**: Rearrange custom theme color pickers vertically to dramatically improve visibility and usability within constrained side panels.

### 🐛 Bug Fixes

- **i18n Validation**: Synchronize translation tokens across all 10 locales to eliminate strict AST linter panics regarding identical string declarations.
- **Lint Stabilisation**: Apply necessary UI layout test clipping fixes and resolve Markdown Lint discrepancies.

### 🔧 Miscellaneous

- **Internal**: Comprehensive Unit and Integration test expansions to strictly maintain the 100% coverage quality gate.
- **Ops**: Archive finished OpenSpec documentation artifacts (`v0.7.4`, `v0.7.5`) and properly segment subsequent workflow proposals (`v0.7.7`, `v0.8.0`).

## [0.7.5] - 2026-03-26 05:43:00 (UTC)

### 🐛 Bug Fixes

- **Tab Navigation**: Resolve an issue where closing a tab would fail to load and render the preview for the newly activated tab if it was previously opened in the background without being rendered.

## [0.7.4] - 2026-03-26 05:15:00 (UTC)

### 🚀 Features

- **In-App Updater**: Integrate a visual progress bar during the update download phase and correct the GitHub API release URL.

### 🎨 UI/UX

- **About Dialog**: Redesign link rows to display aligned icons (e.g., External Link) and make the build version commit hash clickable.
- **Settings Window**: Conditionally hide the markdown preview panel in System settings to optimize space.
- **Settings Window**: Center accordion icons optically for a more balanced layout.

### 🐛 Bug Fixes

- **Tab Navigation**: Resolve a critical degradation where the close tab button was rendered unclickable due to the drag-and-drop interaction overlay.
- **Settings Window**: Fix layout regressions where the window stretched horizontally out-of-bounds while the title bar rendered insufficiently short by enforcing rigorous geometry combinations (`fixed_size` with `min_width`).

## [0.7.3] - 2026-03-26 00:30:00 (UTC)

### 🚀 Features

- **Source Tree**: Add context menus (Open, Rename, Delete, Copy) to directory and file items via right-click interaction.

### 🎨 UI/UX

- **Tab Navigation**: Improve bidirectional drag-and-drop tab movement by unifying drop points to exact midpoints between tabs.
- **Tab Navigation**: Support auto-scrolling when dragging a tab to the edges of the visible scroll area.

### 🐛 Bug Fixes

- **Code Blocks**: Fix syntax highlight background color rendering issue caused by `syntect` override by switching to direct `Painter::rect_filled` layer drawing.

## [0.7.2] - 2026-03-26

### 🐛 Bug Fixes

- **UI Layout**: Fix update checker dialog stretching vertically in the “up to date” state. Changed `ScrollArea::auto_shrink` from `[false; 2]` to `[true, true]` and added `default_height(0.0)` to the `egui::Window` so the window height follows content size. Added regression test to prevent recurrence.

## [0.7.1] - 2026-03-26 05:00:00 (JST)

### 🐛 Bug Fixes

- **Auto-Updater**: Eliminated reliance on `api.github.com`, resolving the fundamental architecture flaw that forced rate-limited `HTTP 403 Forbidden` API crashes on consumer networks.
- **UI Layout**: Repaired an `egui::Window` memory constraint bug that caused the update checker modal to unpredictably stretch vertically with blank whitespace.

## [0.7.0] - 2026-03-26 03:00:00 (UTC)

### ✨ Features

- **In-App Updater**: Implement interactive UI for the auto-update release framework, incorporating Markdown-rendered release notes and integrated extraction logic.

## [0.6.4] - 2026-03-25 09:50:20 (UTC)

### 🐛 Bug Fixes

- **Underline Rendering**: Manually draw underlines using proportional geometry bounds to bypass macOS CJK font metric corruption, ensuring `<u>` tags are consistently visible across all environments.

### 🔧 Miscellaneous

- **Regression Tests**: Implement `egui_kittest` integration tests and `pulldown-cmark` AST unit tests to permanently guarantee inline formatting integrity.
- **Ops**: Minor workflow refinements for the OpenSpec process.

## [0.6.3] - 2026-03-25 08:26:00 (UTC)

### 🐛 Bug Fixes

- **i18n**: Remove trailing space from the Homebrew update command in localized update notification messages.

## [0.6.2] - 2026-03-25 08:05:30 (UTC)

### 🚀 Features

- **Interactive Task Lists**: Support for custom states (`[/]`), context menu interactions, and precision vertical alignment.
- **MathJax Support**: High-fidelity TeX/LaTeX equation rendering leveraging the MathJax pipeline for native-quality formatting.

### 🎨 UI/UX

- **Split-View Scroll Synchronization**: Bidirectional scroll tracking between the editor and preview pane with exact block-level precision.
- **Hover Highlights**: Visual highlight of the corresponding markdown structure under the cursor in Split-View mode.

### 🐛 Bug Fixes

- **Scroll Sync Drift**: Resolve multi-byte character panics (`byte index is not a char boundary`) and long-document drift by switching to char-iterator coordinate maps.

### 🔧 Miscellaneous

- **CI Stability**: Fix flaky integration tests and address race conditions to guarantee stable CI runs.

## [0.6.1] - 2026-03-24 03:15:14 (UTC)

### 🎨 UI/UX

- **Strikethrough Rendering**: Replace `allocate_painter` with `Label` + `ui.painter()` overlay rendering. Resolves text clipping (left-side truncation) and misalignment within list items, with Y-position tuned for CJK glyph centering.
- **Table Centering**: Replace manual margin calculations with native `Layout::top_down(Align::Center)`, providing stable CSS-like `margin: 0 auto` behavior for tables.
- **Workspace Click Areas**: Expand clickable regions to cover both icon and name for directories/files, with hover effects and context menu support across the full row.
- **Tab Scroll Following**: Active tab scrolls into view only on navigation button press, preventing unwanted scroll jumps during manual scrolling.
- **Light Mode Icons**: Apply consistent gray background to all sidebar icons (filter, TOC toggle, etc.) for improved visibility in light theme.
- **Preview Padding**: Remove extraneous padding on preview and main window outer frames, and equalize left/right inner margins.

### 🐛 Bug Fixes

- **Inline Code Alignment**: Change `egui::Align` from `BOTTOM` to `TOP` for inline code and strikethrough, correcting vertical positioning of background fills.
- **Table Header Border**: Fix header-row border rendering that was being cut off midway.
- **Table Column Alignment**: Implement per-column text alignment (left/center/right) as specified in Markdown alignment syntax.
- **Force Reload**: Explicitly read file content from disk and call `mark_clean()` on `AppAction::RefreshDiagrams` to prevent stale UI state.
- **Multi-Tab Open**: Ensure truly asynchronous/parallel tab loading when opening multiple files from a workspace directory, with the first file activated immediately.

### 🔧 Miscellaneous

- **Coverage**: Remove conditional branch around `katana_fonts_loaded` flag — always write the value unconditionally to guarantee coverage under `cargo llvm-cov`.
- **Fixture Relocation**: Move test fixture files from `crates/katana-ui/tests/fixtures/` to `assets/fixtures/` for centralized resource management.

## [0.6.0] - 2026-03-22 21:52:53 (UTC)

### 🐛 Bug Fixes

- **Heavy CPU Load**: Fix 100% idle CPU utilization and spinner UI freeze by optimizing rendering and SVG load logic.
- **Blockquote Rendering**: Fix list item line breaks inside blockquotes and remove unnecessary vertical whitespace around code blocks.
- **Code Block Copy Button**: Transition from SVG icons to direct `Painter` API drawing for reliability. Adjusted button positioning for better visibility and UX.
- **Settings Window Layout**: Stabilize centrally squeezed layout by enforcing fixed widths on side panels.
- **Integration Test Stability**: Skip splash screen natively in test harness context without causing false positives.
- **Quality Assurance**: Expand `svg_loader` coverage with fallback logic tests to maintain 100% line coverage standards.

### 🔧 Miscellaneous

- **OpenSpec Workflow Integration**: [UI Rendering Bug Fixes (v0.6.0)](./openspec/changes/archive/2026-03-23-v0-6-0-ui-bug-fixes/tasks.md) Specification-driven development (SDD) workflow with consolidated task management.

## [0.5.2] - 2026-03-22 12:44:52 (UTC)

### 🚀 Features

- **Project Scan Settings**: Add "Workspace" settings tab to configure `max_depth` and `ignored_directories` for directory scanning.

### ⚡ Performance

- **Idle CPU Optimization**: Significant reduction in idle CPU usage (from 75%+ to <5%) by optimizing window title updates, splash screen repaints, and font rebuilding logic.
- **Background Thread Stabilization**: Ensure rendering engine resources are properly released on workspace switch to prevent thread proliferation.

### 🐛 Bug Fixes

- **Workspace History**: Fix persistence and ordering of recently opened workspaces.
- **Rendering Crash Recovery**: Fix infinite spinner loop caused by unhandled rendering thread panic.

### 🔧 Miscellaneous

- **Linter Guardrails**: Add `ast_linter` rule to prevent future performance regressions (e.g., unconditional repaints).
- **Quality**: Fix borrow checker errors, synchronize all i18n locale files, and achieve 100% test coverage gate.
- **CI/CD**: Add concurrency control to GitHub Actions workflow

## [0.5.1] - 2026-03-22 09:41:24 (UTC)

### 🐛 Bug Fixes

- Fix GitHub release creation by pushing the tag before creating the release

### 🔧 Miscellaneous

- Update Rust dependencies and resolve Linux lint errors

## [0.5.0] - 2026-03-22 09:16:29 (UTC)

### 🚀 Features

- Add Terms of Service agreement with version tracking
- Implement Markdown export (HTML, PDF, PNG, JPG)

### 🎨 UI/UX

- Polish Terms modal with language ComboBox and better centering
- Workspace sidebar filter icon changed to ∇ (Nabla) for better semantics

## [0.4.0] - 2026-03-21 13:05:00 (UTC)

### 🚀 Features

- Add App Branding (Icon & Splash Screen)
- Implement Check For Updates functionality
- Add native menus for Checking for Updates, Help, and Donations
- Optimize Diagram Texture implementation with cache
- Add Trackpad support (Pan and Zoom) to Preview and Full-screen Viewers

### 🐛 Bug Fixes

- Fix Native Fullscreen on macOS displaying black background
- Support relative image resolution in Markdown
- Fix integration TOC bugs

## [0.3.1] - 2026-03-21 04:32:00 (UTC)

### 🚀 Features

- Add `FORCE=1` option to `make release` to skip all interactive confirmation prompts
- Implement `USE_GITHUB_WORKFLOW` flag to conditionally trigger GitHub Actions release

### ♻️ Refactoring

- Modularize release logic into independent scripts under `scripts/release/`
- Move main release control script to `scripts/release/release.sh`

### 🔧 Miscellaneous

- Skip Git hooks (`--no-verify`) during release push as quality checks are pre-verified
- Enable full local release flow (DMG build, GitHub publication, Homebrew update) as default

## [0.3.0] - 2026-03-21 03:52:24 (UTC)

### 🚀 Features

- Implement Tab Context Menu (Close, Close Others, Close All) and Tab Restoration actions
- Support automatic restoration of previously opened workspace tabs on startup
- Add Editor Table of Contents (TOC) side panel with setting persistence and i18n support
- Enable Editor Line Numbers and Current Line Highlighting features

### 🐛 Bug Fixes

- Resolve Japanese CJK font baseline jitter (ガタツキ) in UI components
- Prevent TOC side panel auto-expansion and ignore YAML frontmatter in outline
- Allow dead_code for macOS specific emoji rendering constants on Linux CI

### 📚 Documentation

- Add implementation plan workflow and exclude related artifacts
- Enforce prohibition rule against deleting flaky tests in coding rules

### 👷 CI/CD

- Fix missing tool argument for cargo-llvm-cov in install-action

### 🔧 Miscellaneous

- Restore signed tag generation config after GPG environment setup
- Update dependencies (rustls-webpki)

## [0.2.1] - 2026-03-21 00:53:02 (UTC)

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

## [0.2.0] - 2026-03-20 19:16:37 (UTC)

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

## [0.1.6] - 2026-03-19 23:57:28 (UTC)

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

## [0.1.5] - 2026-03-19 21:12:34 (UTC)

### ♻️ Refactoring

- Unify HashMap and fixed-length arrays into Vec, and apply collectively including AST rules and migration functionality

### 🔧 Miscellaneous

- Bump version to 0.1.4

### 🚀 Features

- Apply v0.1.5 changes and bump version to 0.1.5

### 🧪 Testing

- Fix tests broken by workspace methods renaming
- Add missing tests to meet 100% coverage gate

## [0.1.4] - 2026-03-19 21:03:35 (UTC)

### 🧪 Testing

- Completely abolish UI image snapshot tests that were causing repository bloat and CI failures, and migrate to semantic assertions

## [0.1.3] - 2026-03-19 19:59:23 (UTC)

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

## [0.1.2] - 2026-03-19 16:54:57 (UTC)

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

## [0.1.1] - 2026-03-19 10:54:34 (UTC)

### 🐛 Bug Fixes

- Add error handling to Homebrew Cask update step
- Prevent contamination of cached old DMG files
- Add cfg guards to macOS-specific code in emoji.rs (fix Linux CI Lint)

### 🚀 Features

- Support hidden directory display in workspace tree and add directory refresh button

## [0.1.0] - 2026-03-19 09:33:46 (UTC)

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

## [0.0.3] - 2026-03-18 02:50:23 (UTC)

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

## [0.0.2] - 2026-03-17 09:20:28 (UTC)

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

## [0.0.1] - 2026-03-16 23:16:22 (UTC)

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
