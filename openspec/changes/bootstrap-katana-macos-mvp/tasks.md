## 1. Application foundation

- [x] 1.1 Create the initial Rust project layout for `ui`, `core`, and `platform` modules, add the MVP dependencies for `egui`, `comrak`, and preview rendering, and verify `cargo check` passes.
- [x] 1.2 Define the shared application state, action flow, and service interfaces for filesystem access, settings, clipboard, and OS integration, and document the boundary of each service.
- [x] 1.3 Wire macOS app startup so the shell can initialize services, registries, and an empty workspace session cleanly, and verify the app reaches the initial shell without panicking.

## 2. Workspace shell and document lifecycle

- [x] 2.1 Implement workspace selection and local directory loading with recoverable error handling for unreadable paths, and cover both success and failure paths with tests.
- [x] 2.2 Build the project tree model so workspace files and directories can be browsed and selected from the shell, and verify document selection switches the active file.
- [x] 2.3 Implement active document loading, in-memory buffer updates, dirty tracking, and explicit save-to-disk behavior for Markdown files, verify the save flow against a fixture workspace, and confirm no implicit source-file autosave occurs.
- [x] 2.4 Render the three-pane MVP shell with workspace navigation, editor, and preview regions plus reserved space for future menu and AI expansion, and verify the shell layout on macOS.

## 3. Markdown preview foundation

- [x] 3.1 Build the Markdown parsing pipeline that converts the active in-memory buffer into preview-ready output using `comrak`, and verify representative GFM fixtures render successfully.
- [ ] 3.2 Integrate an HTML/SVG-capable preview surface and synchronize it with editor changes so preview updates do not depend on saving files first, and prove unsaved edits update preview correctly.
- [x] 3.3 Add preview error handling and fallback states so malformed documents or render failures do not crash the application, and cover the fallback behavior with automated tests.

## 4. Built-in diagram rendering

- [x] 4.1 Detect fenced `mermaid`, `plantuml`, and `drawio` blocks in the Markdown pipeline, enforce the supported input formats, and route them through diagram renderer adapters.
- [x] 4.2 Bundle and integrate the default Mermaid renderer so supported Mermaid blocks display inline in the standard preview, and verify the behavior with fixture-based rendering tests.
- [x] 4.3 Bundle and integrate the default Draw.io renderer so supported raw XML Draw.io blocks display inline in the standard preview, and verify unsupported encodings fall back cleanly.
- [x] 4.4 Implement the bundled local PlantUML rendering path using the packaged `plantuml.jar` and Graphviz `dot` assets to produce preview-safe SVG output without requiring a hosted service, and verify rendering works offline on macOS.
- [x] 4.5 Add graceful fallback states for invalid or failed diagram renders while preserving the rest of the Markdown preview, and validate the error state for each supported diagram type.

## 5. AI and plugin extensibility foundations

- [x] 5.1 Define AI provider request and response traits, provider registration, and disabled-state behavior when no provider is configured, and verify the app remains usable with no provider.
- [x] 5.2 Implement a plugin registry with API version checks and static built-in plugin registration during application startup, and verify incompatible plugins are rejected safely.
- [x] 5.3 Expose typed extension points for renderer enhancements, AI tools, and UI panel contributions while isolating plugin failures from core editing flows, and cover plugin failure isolation with tests.

## 6. Validation assets and quality gates

- [x] 6.1 Create a sample spec-driven workspace fixture that includes Markdown documents and representative Mermaid, PlantUML, and Draw.io blocks using only supported input formats.
- [x] 6.2 Add automated tests for workspace loading, document save flow, preview synchronization, diagram rendering, plugin failure isolation, and renderer failure fallback behavior.
- [x] 6.3 Document the MVP architecture, explicit-save constraints, diagram rendering constraints, bundled runtime assets, and follow-on implementation expectations for later AI and plugin phases.

## 7. OSS repository security baseline

- [x] 7.1 Add a generic `.github/SECURITY.md` that instructs reporters to use GitHub private vulnerability reporting and avoids maintainer-specific personal contact details.
- [x] 7.2 Add repository security automation artifacts such as `.github/dependabot.yml`, and enable dependency graph, Dependabot alerts, Dependabot security updates, and CodeQL default setup where supported.
- [x] 7.3 Configure the public-repository baseline for secret scanning, push protection, default branch rulesets, and GitHub Actions hardening, including restricted `GITHUB_TOKEN` permissions and approval requirements for fork-based workflow runs.
- [x] 7.4 Verify the OSS security baseline before making the repository public, and keep any intentionally deferred controls explicit in the release decision.

## 8. User Feedback & Structural Refinements (Current Priorities)

- [ ] 8.1 Fix Test Compilation: Fix the missing imports and visibility issues caused by separating internal tests to the `tests/` directory at the project root for crates.
- [ ] 8.2 i18n & Settings Menu: Complete the setup of `en.json` and `ja.json` through the system, fix the View Mode texts to use `.t()`, and wire up the Settings -> Language menu properly. Change the binary name to `katana`.
- [ ] 8.3 Lazy Load & Diagram UI: Ensure preview diagrams (PlantUML, Mermaid) do not block rendering and properly show a lazy-load placeholder ("Rendering...") while background processes are running. 
- [ ] 8.4 PlantUML Desktop Focus Bug: Investigate and fix the PlantUML headless mode on macOS so it does not pull focus or move the user to the desktop.
- [ ] 8.5 Mermaid Text Visibility: Fix Mermaid text not rendering properly by adjusting `mmdc` configurations or ensuring a fully configured headless rendering environment.
- [ ] 8.6 Tabbed Interface & View Modes: Implement multiple documents opened via tabs. Add UI buttons to switch between Preview Only (default), Code Only, and Split mode.
