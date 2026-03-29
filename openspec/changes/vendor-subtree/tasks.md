### Global Definition of Ready (DoR)

- [ ] `v0-8-6-preview-refresh-and-tasklist-fixes` is merged and stabilized, and no concurrent branch is editing `vendor/*egui_commonmark*`
- [ ] The target release version for this maintenance window is assigned
- [ ] A compatible upstream revision in the `0.22.x` line, or an explicit compatible fork revision, is identified before implementation starts

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

## 1. Current Patch Inventory and Assumption Freeze

- [ ] 1.1 Diff `vendor/egui_commonmark` and `vendor/egui_commonmark_backend` against the chosen compatible upstream revision
- [ ] 1.2 Classify every local delta as required Katana patch, removable drift, or path/layout migration work
- [ ] 1.3 Record the required patch inventory explicitly, including `katana-core` integration, parser/rendering overrides, backend UI/layout changes, and vendored SVG assets
- [ ] 1.4 If the upstream layout, compatible revision, or local patch inventory materially differs from this change, update `proposal.md`, `design.md`, `specs/`, and `tasks.md` before continuing

### Definition of Done (DoD)

- [ ] Every current vendor-local delta is classified before any subtree import commit is made
- [ ] The compatible upstream revision is pinned explicitly
- [ ] The artifact has been corrected first if a key assumption proved wrong
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Subtree Import and Runtime Path Rewiring

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 Import the upstream `lampsitter/egui_commonmark` repository root under `vendor/egui_commonmark_upstream/` using `git subtree`
- [ ] 2.2 Keep the raw subtree import separate from Katana-specific changes
- [ ] 2.3 Update `[patch.crates-io]` in `Cargo.toml` so `egui_commonmark` and `egui_commonmark_backend` resolve from crate subdirectories inside the subtree root
- [ ] 2.4 Update direct file consumers such as `crates/katana-ui/src/svg_loader/mod.rs` to the new subtree asset paths
- [ ] 2.5 Remove build/runtime references to the legacy `vendor/egui_commonmark` and `vendor/egui_commonmark_backend` directories

### Definition of Done (DoD)

- [ ] Katana resolves both vendored crates from the subtree root instead of legacy copied directories
- [ ] The raw subtree import commit is reviewable without mixed Katana patch logic
- [ ] No runtime or build path still depends on the removed legacy vendor layout
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Katana Patch Reapplication and Regression Protection

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 Reapply the required Katana-specific manifest, parser, rendering, asset, and UI patches identified in Task 1
- [ ] 3.2 Preserve or add regression coverage for vendor-dependent behaviors already relied on by Katana
- [ ] 3.3 Verify that subtree migration did not silently upgrade KatanA to upstream `0.23.x` / `egui 0.34.x`
- [ ] 3.4 Confirm that vendor-dependent fixes from `v0-8-6-preview-refresh-and-tasklist-fixes` remain intact after the migration

### Definition of Done (DoD)

- [ ] The final commit stack clearly separates subtree base from Katana patch layer
- [ ] Vendor-dependent runtime behavior matches the pre-migration contract
- [ ] Required regression coverage exists for the behaviors preserved by the patch layer
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. Sync Runbook and Maintenance Handoff

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 4.1 Add a maintainer runbook (for example `docs/vendor-egui-commonmark.md`) describing subtree remote, prefix, compatible revision policy, and patch-layer rules
- [ ] 4.2 Document the commands and verification steps required for future subtree pulls
- [ ] 4.3 Document the stop-and-correct rule: if compatibility assumptions change, update the OpenSpec artifact before continuing implementation

### Definition of Done (DoD)

- [ ] Another AI agent or maintainer can update the subtree and rerun verification without relying on this conversation
- [ ] The runbook explains both the steady-state sync flow and the artifact-correction escalation path
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 5. Final Verification & Release Work

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 5.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 5.2 Ensure `make check` passes with exit code 0
- [ ] 5.3 Verify no code path still references legacy `vendor/egui_commonmark*` directories
- [ ] 5.4 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 5.5 Create a PR targeting `master`
- [ ] 5.6 Merge into master (※ `--admin` is permitted)
- [ ] 5.7 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for the target version assigned in the Global DoR
- [ ] 5.8 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
