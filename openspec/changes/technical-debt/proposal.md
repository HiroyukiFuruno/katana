# Proposal: Refactor Large UI Rendering Functions

## Goal
Enforce the "30 lines per function" coding standard across the `katana-ui` crate.

## Discovery Context
During the extraction of components from `shell_ui.rs` into the `views/` modules (e.g., `views/panels/workspace.rs`, `views/top_bar.rs`), we observed that several rendering functions inherited directly from `shell_ui.rs` far exceed the 30-line limit defined in `docs/coding-rules.md`. To maintain safety and prevent regressions during the massive 5,000-line modularization, we preserved the internal structure of these functions.

## Technical Debt Item
- File: `crates/katana-ui/src/views/panels/workspace.rs`
  - Violation: `render_workspace_panel()`, `render_tree_node()`, etc. are extremely long.
- File: `crates/katana-ui/src/views/top_bar.rs`
  - Violation: `render_top_bar()` and deeply nested UI elements exceed both size limits and nesting depth limits.
- Other `views/**/*.rs` files contain similar large `egui::Ui` inline rendering blocks.

## Proposed Fix
1. Iteratively target individual `views/*` modules.
2. Decompose large UI blocks into smaller helper functions / internal sub-components.
3. Extract inline closures into private `impl` methods where applicable to reduce nesting.
4. Ensure no new bugs are introduced by confirming layout logic matches visual parity.
