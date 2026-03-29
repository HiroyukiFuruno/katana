## Context

`vendor-subtree` is a maintenance change for KatanA's Markdown rendering fork, not a user-facing feature. The objective is to make ongoing vendor maintenance tractable without mixing repository-structure churn into active parser bug-fix work.

## Current-State Analysis

### 1. Vendor scope is already broader than `vendor/egui_commonmark`

- `Cargo.toml` patches both `egui_commonmark` and `egui_commonmark_backend` to local paths.
- `crates/katana-ui/src/svg_loader/mod.rs` loads `copy.svg` and `check.svg` directly from `vendor/egui_commonmark_backend/src`.
- The upstream repository for both crates is the same (`https://github.com/lampsitter/egui_commonmark`).
- The upstream repository layout contains a workspace root with sibling crates `egui_commonmark`, `egui_commonmark_backend`, and `egui_commonmark_macros`.

Conclusion: a plan that only migrates `vendor/egui_commonmark` is incomplete.

### 2. The current vendor directories already contain Katana-specific behavior

Observed local deltas against upstream `v0.22.0` include:

- `vendor/egui_commonmark/Cargo.toml.orig` adds a `katana-core` dependency.
- `vendor/egui_commonmark/src/parsers/pulldown.rs` contains Katana-specific inline emoji rendering logic.
- `vendor/egui_commonmark/src/lib.rs` exposes additional internal modules.
- `vendor/egui_commonmark_backend/src/alerts.rs` and `src/elements.rs` contain local UI/layout changes.
- `vendor/egui_commonmark_backend/src/check.svg` and `src/copy.svg` are locally consumed assets.

Conclusion: subtree migration cannot be treated as a pure file move. It must inventory, preserve, and isolate an existing patch stack.

### 3. Product code and tests depend on the vendor fork directly

- `crates/katana-core/src/preview/mod.rs` documents behavior that exists because of `egui_commonmark` limitations.
- `crates/katana-ui/tests/underline_rendering.rs` references a vendor parser override as part of the expected rendering path.
- `openspec/changes/v0-8-6-preview-refresh-and-tasklist-fixes/tasks.md` already plans new vendor-side task-list fixes in both `egui_commonmark` and `egui_commonmark_backend`.

Conclusion: subtree conversion is valuable, but doing it while `v0-8-6` is still mutating the same surface area would create unnecessary churn and make review quality worse.

### 4. "Use latest upstream" is the wrong target

- Current KatanA depends on `egui 0.33` and vendors `egui_commonmark 0.22.0`.
- Upstream HEAD has already moved to workspace version `0.23.0`, `egui 0.34.0`, and newer workspace metadata.

Conclusion: this change must pin a compatible upstream revision in the `0.22.x` line or an explicit compatible fork revision. Subtree migration and dependency upgrade are separate concerns.

## Target State

After this change:

1. The upstream `lampsitter/egui_commonmark` repository is represented once under `vendor/egui_commonmark_upstream/`.
2. `Cargo.toml` resolves `egui_commonmark` and `egui_commonmark_backend` from crate subdirectories inside that subtree root.
3. Katana-specific modifications are reintroduced as reviewable commits on top of the raw subtree import instead of being baked into an opaque copied snapshot.
4. Direct product references such as SVG asset includes point at the new subtree layout.
5. A documented sync runbook exists so the next maintainer can pull upstream, reapply/adjust Katana patches, and run the required regression suite without relying on this conversation.

## Non-Goals

- Upgrading KatanA to upstream `egui_commonmark 0.23.x` or `egui 0.34.x`
- Redesigning Katana's Markdown rendering architecture
- Folding active vendor bug-fix work into the subtree migration branch

## Decisions

### 1. Necessity is medium, not low and not blocking

This change is worth doing because vendor-local patches are now recurring product work. However, it does not unblock `v0-8-6` or any currently planned user-facing feature by itself. The right execution window is immediately after `v0-8-6` stabilization and before the next vendor-touching branch starts.

### 2. The subtree unit is the upstream repository root

Because the upstream repository contains `egui_commonmark`, `egui_commonmark_backend`, and `egui_commonmark_macros` together, the subtree must import the upstream workspace root under a single prefix such as `vendor/egui_commonmark_upstream/`.

Why this is required:

- It preserves the real upstream layout.
- It prevents drift between sibling crates that are versioned together upstream.
- It lets Katana track both currently used crates without inventing separate pseudo-upstreams.
- It keeps room for future use of `egui_commonmark_macros` without another repository-structure migration.

### 3. Migration must target a compatible upstream revision

Implementation must select a revision from the `0.22.x` line, or another explicit compatible revision, before any code movement happens. If no compatible upstream revision exists for the current `egui 0.33` stack, the team must either:

- maintain a Katana fork branch as the subtree remote target, or
- explicitly broaden the change into a dependency-upgrade plan by updating the artifacts first.

Do not silently switch this change into an upgrade-to-latest effort.

### 4. Patch layering must be explicit

The migration should produce a commit stack with clear responsibility boundaries:

1. raw subtree import of the chosen upstream revision
2. repository path rewiring in Katana (`Cargo.toml`, asset includes, direct references)
3. Katana-specific vendor patches
4. sync/runbook documentation

This separation is required so a future maintainer can inspect local divergence without reverse-engineering a giant mixed commit.

### 5. Direct consumers must be rewired in the same change

The migration is incomplete unless all direct path consumers move off the legacy directories. At minimum this includes:

- `[patch.crates-io]` entries in `Cargo.toml`
- `crates/katana-ui/src/svg_loader/mod.rs`
- tests and comments that hard-code legacy vendor paths

### 6. Implementation must stop and correct the artifact if a core assumption breaks

Before continuing past the inventory phase, the implementer must update `proposal/design/spec/tasks` first if any of the following prove false:

- the chosen upstream-compatible revision is not actually compatible with KatanA's current dependency graph
- the upstream workspace layout differs from the expected root + crate-subdirectory model
- Katana patch inventory is materially larger than described here
- additional product-critical consumers of the legacy vendor paths are discovered

This change is only "ready to implement" if the artifact stays ahead of reality.

## Implementation Blueprint

### Phase 1: Freeze scope and inventory the current patch set

- Diff current `vendor/egui_commonmark` and `vendor/egui_commonmark_backend` against the chosen compatible upstream revision.
- Classify every delta as one of:
  - required Katana patch to preserve
  - removable drift / generated-file noise
  - path/layout difference caused by the new subtree root
- Record the final patch inventory before the raw subtree import is committed.

### Phase 2: Import the upstream repository root as a subtree

- Add the upstream repository under `vendor/egui_commonmark_upstream/`.
- Pin to the chosen compatible revision instead of upstream HEAD.
- Do not carry local Katana changes inside the raw import commit.

### Phase 3: Rewire Katana to the subtree layout

- Update `[patch.crates-io]` paths in `Cargo.toml` to point into `vendor/egui_commonmark_upstream/...`.
- Update `crates/katana-ui/src/svg_loader/mod.rs` and any other direct file references to the new asset locations.
- Remove runtime/build references to the old `vendor/egui_commonmark` and `vendor/egui_commonmark_backend` paths.

### Phase 4: Reapply Katana-specific patches on top

- Restore the `katana-core` integration and other required crate-manifest changes.
- Reapply required parser/UI patches identified in the Phase 1 inventory.
- Preserve or expand regression coverage for known vendor-dependent behavior such as underline rendering, inline emoji support, alert/code-block UI behavior, and any vendor fixes shipped in `v0-8-6`.

### Phase 5: Document the maintenance workflow

- Add a runbook, e.g. `docs/vendor-egui-commonmark.md`, that documents:
  - subtree remote and prefix
  - how to pull a new compatible upstream revision
  - where Katana-specific patches live conceptually
  - mandatory verification commands before merge

## Verification Strategy

Minimum verification for this change:

- `make check`
- relevant vendor regression tests in `katana-ui` and `katana-core`
- a manual audit that no code path still references removed legacy vendor directories
- confirmation that the subtree revision remains on the compatible `0.22.x` line unless the artifact was explicitly revised to broaden scope

## Recommended Execution Window

- After `v0-8-6-preview-refresh-and-tasklist-fixes` is merged and stable
- Before the next branch that edits `vendor/*egui_commonmark*`
- In a dedicated maintenance branch, not bundled with feature work
