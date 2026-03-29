## Why

KatanA currently vendors patched crate snapshots under `vendor/egui_commonmark` and `vendor/egui_commonmark_backend`. These are not passive third-party copies anymore:

- `[patch.crates-io]` in `Cargo.toml` points the workspace directly at both vendored crates.
- `crates/katana-ui/src/svg_loader/mod.rs` includes SVG assets from `vendor/egui_commonmark_backend/src`.
- `crates/katana-ui/tests/underline_rendering.rs` and preview-related code rely on Katana-specific parser behavior inside the vendor fork.
- `vendor/egui_commonmark` already contains Katana-only integration such as the `katana-core` dependency and inline emoji rendering logic.

This means vendor changes recur as product work, but the repository currently has no upstream relationship recorded in git history. Every sync or diff against upstream becomes manual, and `egui_commonmark` onlyを見ても実態を説明しきれません。`egui_commonmark_backend` も同じ upstream repo と同じ runtime path に含まれているため、両方を一体で管理し直す必要があります。

At the same time, this is not a blocking prerequisite for the current roadmap. `v0-8-6-preview-refresh-and-tasklist-fixes` is actively modifying vendored parser behavior, and upstream HEAD has already moved to `egui_commonmark 0.23 / egui 0.34`, while KatanA is still on `egui_commonmark 0.22 / egui 0.33`. The goal is therefore not "upgrade to latest now", but "represent the compatible upstream repository revision as a subtree and keep Katana patches reviewable on top of it".

## What Changes

- Replace ad hoc vendored crate snapshots with a single `git subtree` import of the upstream `lampsitter/egui_commonmark` repository.
- Pin the subtree to a revision in the `0.22.x` line that remains compatible with KatanA's current `egui 0.33` dependency and toolchain assumptions.
- Rewire Cargo patch paths and direct asset/path references to the subtree layout so `egui_commonmark`, `egui_commonmark_backend`, and any upstream sibling crates are resolved from one upstream root.
- Reapply Katana-specific changes as explicit post-subtree commits instead of mixing them into an opaque copied directory.
- Add a documented sync/update workflow so future upstream pulls, local patch review, and regression verification are reproducible.

## Scheduling

- Recommended execution window: after `v0-8-6-preview-refresh-and-tasklist-fixes` is merged and stabilized.
- Do not mix subtree migration into the same branch as active vendor behavior fixes.
- Prefer landing this change immediately before the next planned modification under `vendor/*egui_commonmark*`.

## Capabilities

### Modified Capabilities

- `vendor-dependency-management`: `egui_commonmark` upstream tracking becomes reproducible, reviewable, and compatible with Katana-specific patch layering

## Impact

- `Cargo.toml`
- `crates/katana-ui/src/svg_loader/mod.rs`
- tests and preview/runtime paths that depend on vendor behavior
- `vendor/egui_commonmark*` layout and the maintenance workflow for future upstream syncs
