## ADDED Requirements

### Requirement: Compatible upstream repository SHALL be represented as a single subtree

KatanA SHALL represent the upstream `lampsitter/egui_commonmark` repository as a single `git subtree` rooted under `vendor/egui_commonmark_upstream/`, rather than maintaining independent copied crate snapshots.

#### Scenario: Initial subtree import

- **WHEN** the vendor migration is performed
- **THEN** the upstream repository root is imported under `vendor/egui_commonmark_upstream/`
- **AND** `egui_commonmark` and `egui_commonmark_backend` are resolved from crate subdirectories inside that subtree root
- **AND** the chosen revision is pinned to one compatible with KatanA's current `egui 0.33` line instead of upstream HEAD by default

### Requirement: Katana-specific vendor patches SHALL remain isolated from the raw subtree base

KatanA SHALL preserve its local vendor behavior as an explicit patch layer on top of the raw subtree import.

#### Scenario: Reviewing local divergence

- **WHEN** a maintainer inspects the migration history
- **THEN** the raw subtree import is distinguishable from Katana-specific follow-up commits
- **AND** local behaviors such as `katana-core` integration, parser/rendering overrides, backend UI adjustments, and vendored asset additions remain reviewable as Katana-owned deltas

### Requirement: Migration SHALL preserve direct product consumers of the vendor fork

KatanA SHALL migrate runtime, build, and test consumers to the subtree layout in the same change.

#### Scenario: Runtime and test resolution after migration

- **WHEN** the subtree migration lands
- **THEN** Cargo patch resolution succeeds using the subtree paths
- **AND** direct asset consumers and vendor-dependent tests continue to resolve successfully
- **AND** no active code path still depends on the removed legacy `vendor/egui_commonmark*` directories

### Requirement: Subtree migration SHALL be executed in a dedicated maintenance window

KatanA SHALL schedule this migration after active vendor behavior work is stabilized and before the next vendor-touching branch begins.

#### Scenario: Scheduling the change

- **WHEN** the team schedules implementation of `vendor-subtree`
- **THEN** `v0-8-6-preview-refresh-and-tasklist-fixes` is already merged and stable
- **AND** the migration is not bundled into the same branch as another change that edits `vendor/*egui_commonmark*`
