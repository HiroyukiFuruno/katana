# ADR 0001: UI Testing Strategy and egui_kittest Implementation Plan

## Status

Approved (Implementation put on hold until after egui 0.30 upgrade)

## Background and Context

Katana is a desktop application built with `egui`, but currently, there are only manual UI testing (visual confirmation) means available. While we can separate some logic tests (UI-independent functions) by modularizing the code, automating the following points is a pressing need:

1. **Verification of User Scenarios**: Testing use cases end-to-end, such as "open a workspace, click a file, and render the preview."
2. **Detection of UI Regressions**: Discovering pixel-level layout breaking (snapshot testing).
3. **Automated Continuous Execution in CI**: Guaranteeing operations in a headless state without manual intervention.

While Playwright is commonly used for web applications, Katana is a native `egui` app, requiring a dedicated testing solution.

## Considered Options

| Tool/Method | Features | Evaluation |
|---|---|---|
| **Manual Testing** | Low cost but not scalable | Barrier to CI/CD. Rejected. |
| **Logic Separation (Phase 1)** | Unit tests for state transitions without UI components | Already introduced. However, it does not guarantee "how it actually looks" or "whether it's clickable." |
| **`egui_kittest` (Phase 2)** | UI and snapshot testing using egui rendering and AccessKit | Matches project requirements. Adopted. |

## Decision

We formally adopt **`egui_kittest`** as our UI scenario and snapshot testing infrastructure.

However, during the introduction investigation, the following **version compatibility issue** was discovered:

* The Katana project currently uses **egui 0.29**.
* The minimum required version for `egui_kittest` (the one available on crates.io) is **egui 0.30** (initial release is 0.30.0).

Therefore, we will introduce it in the following phases:

1. **Preparation Phase (Current)**
   * Expansion of unit tests through logic separation (Phase 1 completed)
   * Creation of this ADR and identification of test scenarios (`docs/e2e_scenarios.md`)
   * Surrounding infrastructure setup, such as `Makefile` and `.gitignore`

2. **Implementation Phase (Next Phase)**
   * Perform a workspace upgrade to `egui 0.30`
   * Introduce the `egui_kittest` package
   * E2E implementation in `tests/e2e/` based on defined test scenarios and integration into CI runners

## Consequences and Risks

* **Environmental differences in snapshot testing**: There is a risk of unintended image differences occurring due to GPU rendering or font rendering differences between the macOS environment and the Linux CI environment.
  * **Mitigation**: Adjust the snapshot error tolerance settings (difference threshold) or use mock fonts dedicated to UI testing in the next phase.
* **Dependencies of the upgrade**: Since `egui 0.30` is highly likely to be a major update (breaking changes), there will be a corresponding implementation cost.
