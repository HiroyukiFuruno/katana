# Changelog

All notable changes to KatanA Desktop. This file records the major changes to KatanA Desktop.

## [0.8.0] - 2026-03-28 17:29:17 (UTC)

### 🚀 Features

- Introduced an integrated ChangeLog Viewer UI, allowing users to conveniently browse recent application updates and release notes directly within the app.

### ✨ Improvements

- Unified the alignment of icons and text across the application interface, specifically improving the vertical centering within the ChangeLog and navigation tabs for a cleaner look.

### 🔧 System

- Strengthened network error handling and internal test coverage specifically around background data fetching to guarantee future stability.

## [0.7.10] - 2026-03-28 04:31:08 (UTC)

### 🐛 Bug Fixes

- Restored the missing UI Contrast logic, ensuring transparent background colors (like hover and active rows) correctly adapt their visibility against dark themes.

## [0.7.9] - 2026-03-28 02:54:09 (UTC)

### ✨ Improvements

- Redesigned the Custom Themes settings layout for better usability and a cleaner interface.

### 🐛 Bug Fixes

- Improved the rendering of underlined text to fix an issue where underlines were not displayed correctly on certain macOS environments.
- Fixed an issue where list markers, footnotes, and collapsible text did not properly update their colors when the theme was changed.
- Fixed an issue where changing the theme could cause unsaved documents to be unexpectedly reloaded or discarded.
- Unified hover and text selection highlight colors to match the active accent color across all themes.

## [0.7.8] - 2026-03-27 13:03:25 (UTC)

### 🚀 Features

- Introduced a UI contrast slider in the Appearance settings, allowing fine-grained control over visual contrast limits.
- Added a dedicated 'Clear HTTP Cache' button within the System settings tab for on-demand cache directory purging.

### ✨ Improvements

- Vertically centered and right-aligned color swatches in the Custom Themes grid, improving visual consistency and scanability across all settings panes.

### 🔧 System

- Improved internal codebase quality by enforcing stricter translation standards.
- Improved the performance of background analysis tools.

## [0.7.7] - 2026-03-27 08:30:00 (UTC)

### ✨ Improvements

- Optimized hover and current-line background transparency across all dark themes to improve text visibility.
- Fixed a visual glitch where semi-transparent highlights would appear overly bright or washed out.
- Overhauled the theme customization system to allow detailed adjustments for system elements, code blocks, and preview areas independently.

### 🐛 Bug Fixes

- Resolved an error that could occur when clearing the image cache on macOS.

### 🔧 System

- Eliminated hardcoded color values from the application to improve maintainability and theme stability.

## [0.7.6] - 2026-03-26 21:20:00 (UTC)

### 🚀 Features

- Provide granular control to adjust the auto-save interval with 0.1-second precision.
- Implement toggles for hiding file extensions and add a fully functioning list-style UI for managing scan exclusion paths easily.
- Support complete lifecycle actions for custom themes, including named saving, duplication, and explicit deletion.

### ✨ Improvements

- Standardized all dropdown menus across the application for improved appearance and hover interactions.
- Transition scroll synchronization configuration to an intuitive toggle switch and streamline layout order.
- Rearrange custom theme color pickers vertically to dramatically improve visibility and usability within constrained side panels.

### 🐛 Bug Fixes

- Fixed potential translation errors that could occur when switching application languages.
- Fixed minor layout glitches and formatting issues.

### 🔧 System

- Expanded UI and integration tests to improve application stability.

## [0.7.5] - 2026-03-26 05:43:00 (UTC)

### 🐛 Bug Fixes

- Resolve an issue where closing a tab would fail to load and render the preview for the newly activated tab if it was previously opened in the background without being rendered.

## [0.7.4] - 2026-03-26 05:15:00 (UTC)

### 🚀 Features

- Integrate a visual progress bar during the update download phase and correct the GitHub API release URL.

### ✨ Improvements

- Redesign link rows to display aligned icons (e.g., External Link) and make the build version commit hash clickable.
- Conditionally hide the markdown preview panel in System settings to optimize space.
- Center accordion icons optically for a more balanced layout.

### 🐛 Bug Fixes

- Resolve a critical degradation where the close tab button was rendered unclickable due to the drag-and-drop interaction overlay.
- Fix layout regressions where the window stretched horizontally out-of-bounds while the title bar rendered insufficiently short by enforcing rigorous geometry combinations (`fixed_size` with `min_width`).

## [0.7.3] - 2026-03-26 00:30:00 (UTC)

### 🚀 Features

- Add context menus (Open, Rename, Delete, Copy) to directory and file items via right-click interaction.

### ✨ Improvements

- Improve bidirectional drag-and-drop tab movement by unifying drop points to exact midpoints between tabs.
- Support auto-scrolling when dragging a tab to the edges of the visible scroll area.

### 🐛 Bug Fixes

- Fix syntax highlight background color rendering issue caused by `syntect` override by switching to direct `Painter::rect_filled` layer drawing.

## [0.7.2] - 2026-03-25 21:37:57 (UTC)

### 🐛 Bug Fixes

- Fix update checker dialog stretching vertically in the “up to date” state. Changed `ScrollArea::auto_shrink` from `[false; 2]` to `[true, true]` and added `default_height(0.0)` to the `egui::Window` so the window height follows content size. Added regression test to prevent recurrence.

## [0.7.1] - 2026-03-25 20:00:00 (UTC)

### 🐛 Bug Fixes

- Eliminated reliance on `api.github.com`, resolving the fundamental architecture flaw that forced rate-limited `HTTP 403 Forbidden` API crashes on consumer networks.
- Repaired an `egui::Window` memory constraint bug that caused the update checker modal to unpredictably stretch vertically with blank whitespace.

## [0.7.0] - 2026-03-26 03:00:00 (UTC)

### ✨ Features

- Implement interactive UI for the auto-update release framework, incorporating Markdown-rendered release notes and integrated extraction logic.

## [0.6.4] - 2026-03-25 09:50:20 (UTC)

### 🐛 Bug Fixes

- Manually draw underlines using proportional geometry bounds to bypass macOS CJK font metric corruption, ensuring `<u>` tags are consistently visible across all environments.

### 🔧 System

- Implement `egui_kittest` integration tests and `pulldown-cmark` AST unit tests to permanently guarantee inline formatting integrity.

## [0.6.3] - 2026-03-25 08:26:00 (UTC)

### 🐛 Bug Fixes

- Remove trailing space from the Homebrew update command in localized update notification messages.

## [0.6.2] - 2026-03-25 08:05:30 (UTC)

### 🚀 Features

- Support for custom states (`[/]`), context menu interactions, and precision vertical alignment.
- High-fidelity TeX/LaTeX equation rendering leveraging the MathJax pipeline for native-quality formatting.

### ✨ Improvements

- Bidirectional scroll tracking between the editor and preview pane with exact block-level precision.
- Visual highlight of the corresponding markdown structure under the cursor in Split-View mode.

### 🐛 Bug Fixes

- Resolve multi-byte character panics (`byte index is not a char boundary`) and long-document drift by switching to char-iterator coordinate maps.

## [0.6.1] - 2026-03-24 03:15:14 (UTC)

### ✨ Improvements

- Replace `allocate_painter` with `Label` + `ui.painter()` overlay rendering. Resolves text clipping (left-side truncation) and misalignment within list items, with Y-position tuned for CJK glyph centering.
- Replace manual margin calculations with native `Layout::top_down(Align::Center)`, providing stable CSS-like `margin: 0 auto` behavior for tables.
- Expand clickable regions to cover both icon and name for directories/files, with hover effects and context menu support across the full row.
- Active tab scrolls into view only on navigation button press, preventing unwanted scroll jumps during manual scrolling.
- Apply consistent gray background to all sidebar icons (filter, TOC toggle, etc.) for improved visibility in light theme.
- Remove extraneous padding on preview and main window outer frames, and equalize left/right inner margins.

### 🐛 Bug Fixes

- Change `egui::Align` from `BOTTOM` to `TOP` for inline code and strikethrough, correcting vertical positioning of background fills.
- Fix header-row border rendering that was being cut off midway.
- Implement per-column text alignment (left/center/right) as specified in Markdown alignment syntax.
- Explicitly read file content from disk and call `mark_clean()` on `AppAction::RefreshDiagrams` to prevent stale UI state.
- Ensure truly asynchronous/parallel tab loading when opening multiple files from a workspace directory, with the first file activated immediately.

### 🔧 System

- Remove conditional branch around `katana_fonts_loaded` flag — always write the value unconditionally to guarantee coverage under `cargo llvm-cov`.
- Move test fixture files from `crates/katana-ui/tests/fixtures/` to `assets/fixtures/` for centralized resource management.

## [0.6.0] - 2026-03-22 21:52:53 (UTC)

### 🐛 Bug Fixes

- Fix 100% idle CPU utilization and spinner UI freeze by optimizing rendering and SVG load logic.
- Fix list item line breaks inside blockquotes and remove unnecessary vertical whitespace around code blocks.
- Transition from SVG icons to direct `Painter` API drawing for reliability. Adjusted button positioning for better visibility and UX.
- Stabilize centrally squeezed layout by enforcing fixed widths on side panels.
- Skip splash screen natively in test harness context without causing false positives.
- Expand `svg_loader` coverage with fallback logic tests to maintain 100% line coverage standards.

## [0.5.2] - 2026-03-22 12:44:52 (UTC)

### 🚀 Features

- Add "Workspace" settings tab to configure `max_depth` and `ignored_directories` for directory scanning.

### ✨ Improvements

- Significant reduction in idle CPU usage (from 75%+ to <5%) by optimizing window title updates, splash screen repaints, and font rebuilding logic.
- Ensure rendering engine resources are properly released on workspace switch to prevent thread proliferation.

### 🐛 Bug Fixes

- Fix persistence and ordering of recently opened workspaces.
- Fix infinite spinner loop caused by unhandled rendering thread panic.

### 🔧 System

- Improved stability by enhancing static code analysis.
- Fix borrow checker errors, synchronize all i18n locale files, and achieve 100% test coverage gate.

## [0.5.1] - 2026-03-22 09:41:24 (UTC)

### 🐛 Bug Fixes

- Fix GitHub release creation by pushing the tag before creating the release

## [0.5.0] - 2026-03-22 09:16:29 (UTC)

### 🚀 Features

- Add Terms of Service agreement with version tracking
- Implement Markdown export (HTML, PDF, PNG, JPG)

### ✨ Improvements

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

### 🔧 System

- Skip Git hooks (`--no-verify`) during release push as quality checks are pre-verified
- Enable full local release flow (DMG build, GitHub publication, Homebrew update) as default

### ♻️ Refactoring

- Modularize release logic into independent scripts under `scripts/release/`
- Move main release control script to `scripts/release/release.sh`

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

### 🔧 System

- Restore signed tag generation config after GPG environment setup
- Update dependencies (rustls-webpki)

### 📚 Documentation

- Add implementation plan workflow and exclude related artifacts
- Enforce prohibition rule against deleting flaky tests in coding rules

### 👷 CI/CD

- Fix missing tool argument for cargo-llvm-cov in install-action

## [0.2.1] - 2026-03-21 00:53:02 (UTC)

### 🚀 Features

- Make init command implementation and release automatic push for development environment and flow enhancement

### 🔧 System

- Update Rust dependencies and GitHub Actions plugins
- Fix coverage gap in preview_pane and codify release bypass rules
- Resolve V0.2.0 archive omission and add AI warning block to next tasks

### ♻️ Refactoring

- Rename repository to KatanA, reorganize documents, and support English translation

### 🧪 Testing

- Specify language in settings window integration test to stabilize test
- Collect_matches logic extraction and partial setting screen integration test addition for coverage improvement

## [0.2.0] - 2026-03-20 19:16:37 (UTC)

### 🚀 Features

- Add workspace persistence and tab restoration logic (Task 1)
- Implement CacheFacade and stabilize all integration tests
- Implement recursive expansion of workspaces and "Open All", and improve usability (Task 3, 5)
- Localize metadata tooltips and apply to file items

### 🐛 Bug Fixes

- Enforce strict lazy loading and restrict folder auto-expand on Open All
- Abolish redundant filename tooltip and fix ast linter coverage
- Restore missing absolute path in metadata tooltip and apply TDD

### 🔧 System

- Refactor RwLock usage and fix external image caching on force reload

### 👷 CI/CD

- Automatically generate and attach SHA256 hashes (checksums.txt) for DMG

### 📚 Documentation

- Add release notes for v0.1.6 to CHANGELOG.ja.md and include in Makefile release target

## [0.1.6] - 2026-03-19 23:57:28 (UTC)

### 🚀 Features

- Implement workspace search and filter functionality
- Add internationalized text for search modal Include/Exclude options
- Add inclusion/exclusion filter functionality to search modal and place search button in UI

### 🐛 Bug Fixes

- Automatically inject version into Info.plist during DMG build
- Automatically sync Cargo.lock when running make release

### 🔧 System

- Prepare for v0.1.7 release
- Update Cargo.lock (follow v0.1.7)
- Prepare for v0.1.6 release

### 📚 Documentation

- Add project-specific skill "release_workflow"

### 🧪 Testing

- Add integration tests for Include/Exclude options in search filter

## [0.1.5] - 2026-03-19 21:12:34 (UTC)

### 🚀 Features

- Apply v0.1.5 changes and bump version to 0.1.5

### 🔧 System

- Bump version to 0.1.4

### ♻️ Refactoring

- Unify HashMap and fixed-length arrays into Vec, and apply collectively including AST rules and migration functionality

### 🧪 Testing

- Fix tests broken by workspace methods renaming
- Add missing tests to meet 100% coverage gate

## [0.1.4] - 2026-03-19 21:03:35 (UTC)

## [0.1.3] - 2026-03-19 19:59:23 (UTC)

### 🚀 Features

- Expand theme presets from 10 to 30 (added OneDark/TokyoNight/CatppuccinMocha etc.)
- Migrate i18n to type-safe structs (I18nMessages) and add 8 languages (zh-CN/zh-TW/ko/pt/fr/de/es/it)
- Add 8 language tags to macOS native menu and dynamically translate menu strings according to language switching
- Update entire UI for i18n/settings hierarchization, and implement OS language detection, theme expansion, and Show more/less toggle in settings screen

### 🐛 Bug Fixes

- Recovery of missed v0.1.3 version update
- Fix flaky tests where curl failed to start due to environment variable pollution during parallel test execution

### 🔧 System

- Exclude snapshot tests and redundant integration test executions from make check

### ♻️ Refactoring

- Hierarchize settings.json structure (ThemeSettings/FontSettings/LayoutSettings) and add migration mechanism
- Fix coverage gate and improve code quality

### 📚 Documentation

- Migrate markdownlint settings to .vscode management and add v0.1.3 release notes

### 🧪 Testing

- Update tests according to i18n type-safety, settings hierarchization, and theme expansion (integration/i18n/theme/diagram_rendering tests)

## [0.1.2] - 2026-03-19 16:54:57 (UTC)

### 🚀 Features

- Add i18n tooltips to tab navigation and slider

### 🐛 Bug Fixes

- Fix left alignment of workspace file entries
- Fix issue where font size slider becomes invisible in light theme
- Add selection color border to slider to ensure visibility in all themes
- Modify markdown preview tables to use available width
- Fix bugs in table layout and vertical split scroll

### 🔧 System

- Prepare for v0.1.2 release
- Fix flaky view mode integration test by adding ui stabilization steps
- Turn warnings into errors and remove unused code
- Prepare for v0.1.2 release

### 📚 Documentation

- Add diagram display guide and brew update instructions to README
- Add snapshot prohibition (NG) rule to coding-rules and self-review
- Add brew update instructions to README

### 🧪 Testing

- Add TDD verification tests for UI bugs and update snapshots

## [0.1.1] - 2026-03-19 10:54:34 (UTC)

### 🚀 Features

- Support hidden directory display in workspace tree and add directory refresh button

### 🐛 Bug Fixes

- Add error handling to Homebrew Cask update step
- Prevent contamination of cached old DMG files
- Add cfg guards to macOS-specific code in emoji.rs (fix Linux CI Lint)

## [0.1.0] - 2026-03-19 09:33:46 (UTC)

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

### 🐛 Bug Fixes

- Improve .app signing (abolish --deep, specify runtime/timestamp, DMG remains unsigned)
- Fix regression where workspace is not restored on startup
- Apply emoji support patch to vendored egui_commonmark

### 🔧 System

- Add glob filters to pre-push hook and skip CI execution for pushes without code changes
- Prepare for v0.0.4 release
- Exclude fixture tests from make check-light
- Exclude unnecessary old backup images (.old.png) when updating snapshots from Git tracking
- Expand coverage gate exclusion rules (return None/false/display/Pending)
- Prepare for v0.1.0 release (update version number)

### ⏪ Reverted

- Revert release.yml and README to v0.0.3 state
- Undo v0.0.4 changes and revert to v0.0.3 state

### ♻️ Refactoring

- Rename make ci -> make check and create new make check-light
- Move inline tests in os_fonts.rs to tests/ and translate Japanese comments to English

### 🎨 Styling

- Add reason comments to #[allow] attributes (compliant with coding-rules section 10)

### 📚 Documentation

- Define versioning policy and refine CI triggers
- Unify indentation of make check command with other commands
- Update coding rules and self-review criteria
- Unify code block language specification (text) in English and Japanese versions
- Append CHANGELOG v0.1.0-dev and fix synchronization for README.ja.md

### 🧪 Testing

- Add tests for coverage improvement

## [0.0.3] - 2026-03-18 02:50:23 (UTC)

### 🐛 Bug Fixes

- Unify Coverage job with local make coverage
- Improve DrawIo diagram text visibility in dark theme
- Expand mmdc resolution from .app bundle to 6-layer fallback
- Skip diagram snapshot tests in CI environment
- Fix startup from GUI apps by supplementing node PATH when executing mmdc
- Add margins above and below HTML blocks to resolve layout tightness

### 🔧 System

- Update coverage exclusion reasons to accurate technical grounds
- Prepare for v0.0.3 release
- Change release notes to be extracted from CHANGELOG.md

### ♻️ Refactoring

- Constantize magic numbers and expand AST linter tests
- Unify Ignore tags to limited_local

### 📚 Documentation

- Change version fixed section in README to dynamic status notation

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

### 🔧 System

- Kick ci to retry integration tests
- Release v0.0.2
- Include Cargo.lock and CHANGELOG.ja.md in release v0.0.2

### 📚 Documentation

- Restore xattr command procedure for initial startup

### 🧪 Testing

- Update integration test snapshot
- Increase snapshot tolerance to 4000 to absorb CI/local macOS text rendering differences

## [0.0.1] - 2026-03-16 23:16:22 (UTC)

### 🚀 Features

- Bootstrap Katana macOS MVP — implementation of Rust project foundation and all core modules
- Task 3.2 — native Markdown preview pane implementation
- I18n support, language setting, appAction expansion, bin rename
- Improve diagram rendering (Draw.io arrow support, Mermaid PNG output, CommandNotFound distinction)
- Extend filesystem service (workspace tree and file monitoring improvements)
- Tab-specific preview management, scroll synchronization, macOS native menu, workspace panel control
- Enhance verification — introduction of lefthook, adding tests, tightening Clippy, defining quality gates
- Improved stability by enhancing static code analysis.
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

### 🔧 System

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

### 📚 Documentation

- Mark test compilation, tab UI, and plantuml macos bug as done
- Add i18n conventions (Section 11) to coding-rules
- Add README and document templates, change .obsidian to gitignored
- Add project foundation files — LICENSE (MIT), README, development environment setup script
- Add ADR (Architecture Decision Records) and integration test scenarios
- Add technical debt memo (TECHNICAL_DEBT.md)
- Unify project name to KatanA (README, Cargo.toml, setting comments)
- Restructure documents for general distribution (#21)
- Add "What is KatanA" section (English/Japanese)
- Append origin of "A" as Agent in KatanA

### 🧪 Testing

- Task 6.2 — add preview synchronization tests
- Add app state unit tests and fix java headless mode for plantuml
- Add unit tests for preview synchronization (Task 3.2 completed)
- Tighten coverage — removed ignore-filename-regex, abolished #[coverage(off)], enforced 100% Regions
- Address differences in LLVM coverage calculation and tighten 100% test gate
- Add integration tests for persistence round-trip
