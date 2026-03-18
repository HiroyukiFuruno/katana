# Katana Rust Coding Rules

> [!IMPORTANT]
> **Bi-directional Update Required / 双方向アップデート必須**
> When updating this file (`coding-rules.md`), you MUST also update the Japanese version (`coding-rules.ja.md`) simultaneously to keep them in sync.
> 本ファイル（`coding-rules.md`）を更新する際は、必ず日本語版（`coding-rules.ja.md`）も同時に同期更新してください。

This document defines the Rust coding conventions to be followed throughout the project.
Rules that can be checked automatically by linters are enforced via `.clippy.toml` and `#![deny(...)]` in each crate.

---

## 1. Structure and Responsibility

### 1.1 `struct` + `impl` Based Design

**Domain logic must always be implemented within `struct` + `impl` blocks.**
Free functions are only allowed for internal processing that is module-private (no `pub`).

```rust
// ✅ Good — struct + impl
pub struct DocumentLoader { ... }
impl DocumentLoader {
    pub fn load(&self, path: &Path) -> Result<Document, DocumentError> { ... }
}

// ❌ Bad — pub external processing implemented as a free function
pub fn load_document(path: &Path) -> Result<Document, DocumentError> { ... }

// ✅ Good — Module-internal auxiliary processing is fine as a free function
fn html_escape(s: &str) -> String { ... }
```

### 1.2 SOLID Principles

| Principle | Application in Rust |
|------|-------------|
| S (Single Responsibility) | One `struct` / `impl` has one responsibility. Functions > 30 lines are a sign to separate responsibilities. |
| O (Open-Closed) | Define extension points with `trait` and avoid direct additions to `struct`. |
| L (Liskov Substitution) | `trait` implementations do not break the contract (pre/post-conditions of the documentation). |
| I (Interface Segregation) | Keep `trait`s small and avoid clumping unnecessary methods together. |
| D (Dependency Inversion) | Upper layers depend on `trait` rather than concrete types. |

### 1.3 State Separation Architecture (Global, Session, Tab)

State must be explicitly separated and designed based on its scope and lifecycle.
Fusing states into a single massive struct or using ad-hoc `HashMap`s is considered technical debt.

| State Layer | Description | Example |
|-------------|-------------|---------|
| **Global**  | Overall settings requiring persistence. Always treated as the source of truth. | `SettingsService` (e.g., default split direction) |
| **Session** | Temporary app execution state. Volatile. | `pending_action`, window resize state |
| **Tab (Tab-Specific)** | Independent, isolated cache per document tab. | `SplitViewState` (per-tab split state overrides) |

**Implementation Principles:**
- The moment a tab is opened, copy/cache the Initial Value from the Global settings to generate and isolate the Tab-specific state (e.g., `SplitViewState`).
- Individual UI elements (like toggle buttons) mutate **ONLY the Tab-Specific** state.
- When Global settings change, re-initialize existing Tab states (overwrite and sync with the latest Global value) to prevent state mismatch.

---

## 2. Function Size

**A single function (method or free function) is limited to 30 lines.**
If it exceeds 30 lines, apply the SOLID 'S' principle and separate the responsibilities.

- Linter: Automatically detected by `clippy::too_many_lines` (`too-many-lines-threshold = 30`)
- The line count of the `impl` block itself is not targeted (evaluation is per method)

---

## 3. Nesting Depth

**Code nesting is allowed up to 3 levels. The target is 2 levels.**

```rust
// ✅ Good — Early return with let-else, nesting 2
fn handle_save(&mut self) {
    let Some(doc) = &mut self.state.active_document else {
        return; // ← Keep nesting shallow with error-first
    };
    match self.fs.save_document(doc) {
        Ok(()) => self.state.status_message = Some("Saved.".to_string()),
        Err(e) => self.state.status_message = Some(format!("Save failed: {e}")),
    }
}

// ❌ Bad — Nesting 4
fn handle_save(&mut self) {
    if let Some(doc) = &mut self.state.active_document {
        if doc.is_dirty {
            match self.fs.save_document(doc) {
                Ok(()) => {
                    if let Some(msg) = &mut self.state.status_message { ... }
                }
                ...
            }
        }
    }
}
```

- Linter: Automatically detected by `clippy::cognitive_complexity` (`cognitive-complexity-threshold = 10`)
- Proactively use the `?` operator on `Result` to reduce the nesting of `match` / `if let`

---

## 4. Error First

**If values required for subsequent processing cannot be obtained, return immediately/return an error.**
The `?` operator and `let...else` are the primary choices.

```rust
// ✅ Good
fn process(&self, path: &Path) -> Result<Output, MyError> {
    let content = std::fs::read_to_string(path)?;      // Early return with ?
    let parsed = parse(&content)?;
    Ok(transform(parsed))
}

// ❌ Bad — Nesting that defers errors
fn process(&self, path: &Path) -> Result<Output, MyError> {
    if let Ok(content) = std::fs::read_to_string(path) {
        if let Ok(parsed) = parse(&content) {
            return Ok(transform(parsed));
        }
    }
    Err(MyError::Failed)
}
```

---

## 5. Type Safety and Non-null Design

### 5.1 Prohibited Types

The use of the following types equivalent to TypeScript's `any` / `unknown` / `Record<string, unknown>` is prohibited:

| Prohibited | Reason | Alternative |
|------|------|------|
| `Box<dyn std::any::Any>` | Type erasure | Define a dedicated `trait` / `enum` |
| `HashMap<String, serde_json::Value>` | Unstructured data | Define a typed `struct` |
| `serde_json::Value` (excluding external API boundaries) | Loss of type safety | Corresponding `struct` + `#[derive(Deserialize)]` |

- Linter: `use foo::*` is prohibited by `clippy::wildcard_imports`

### 5.2 Non-null Design

**Never wrap values that are guaranteed to exist in `Option`.**
For places that need `Option`, review the design and change the structure so that `Option` is no longer necessary.

```rust
// ✅ Good — Values guaranteed to exist are held directly
pub struct ActiveDocument {
    pub path: PathBuf,  // Always exists
    pub buffer: String, // Always exists
}

// ❌ Bad — Wrapped in Option even though it's always initialized
pub struct AppState {
    pub active_path: Option<PathBuf>, // Is it really Optional?
}
```

- `unwrap()` is prohibited by `deny(clippy::unwrap_used)`
  - Permitted only as `expect("Explicit reason")` in test code
- `panic!` is prohibited by `deny(clippy::panic)` (outside of tests)
- `todo!` / `unimplemented!` are prohibited by `deny` (except on WIP branches)

---

## 6. Comment Rules

**Comments should only describe the "WHY". The "WHAT" should be expressed through code.**
Comments must be written in **English** (following the English First Policy).

```rust
// ✅ Good — Only WHY, written in English
// comrak disables GFM by default, so we explicitly enable the extension here.
opts.extension.table = true;

// ❌ Bad — Commenting on the WHAT (obvious from reading the code)
// Enable the table extension
opts.extension.table = true;
```

Documentation comments (`///`) must be written in English for public APIs (following crates.io / rustdoc conventions).

---

## 7. Testing Rules

### 7.1 Test Naming

**Test `fn` names should represent the meaning in English snake_case.**
Grouping equivalent to `describe` is done with `mod`.

```rust
// ✅ Good
#[test]
fn unsaved_buffer_does_not_write_to_disk() { ... }
```

### 7.2 Test File Placement (Updated 2026-03-19)

Aligned with Rust community best practices, tests are separated **by kind**.

| Kind | Placement | Description |
|------|-----------|-------------|
| **Unit Test (UT)** | `#[cfg(test)] mod tests { ... }` inside `src/` | Can directly test private APIs within the same crate. Rust standard style. |
| **Integration Test (IT)** | `tests/` directory | Only public APIs are accessible. Verifies behavior from an external-user perspective. |

```text
crates/katana-core/
  src/
    document.rs           # Implementation + #[cfg(test)] mod tests { ... }  (UT)
    workspace.rs          # Implementation + #[cfg(test)] mod tests { ... }  (UT)
  tests/
    integration.rs        # Integration tests via public API (IT)
    markdown_renderer.rs  # Renderer integration tests (IT)
```

**UT Guidelines**

- Write in `#[cfg(test)] mod tests { ... }` within the same file as the `src/` module.
- May test private functions and internal state.
- `#[allow(clippy::unwrap_used)]` is permitted only within the `mod tests` block.

**IT Guidelines**

- Place in the `tests/` directory.
- Only use `pub` APIs of the crate (no private access).
- Import via `use crate_name::...;` and test from the same perspective as an external user.

### 7.3 Test Pyramid

| Type | Placement | Coverage Target |
|------|-----------|----------------|
| Unit Test (UT) | Inline `#[cfg(test)]` inside `src/` | **100% (No Exceptions)** |
| Integration Test (IT) | `tests/` directory | Core public API flow coverage |
| UI Integration Test | `tests/integration/` (egui_kittest) | All MVP scenarios, and assertion of actual responses (Node, Rect, etc.). **Snapshots are deprecated** |

Coverage measurement: `cargo llvm-cov --workspace --fail-under-lines 100` (Forced in CI)

### 7.4 TDD Enforcement (Test-Driven Development)

**All new features and bugfixes MUST follow the TDD cycle: Red → Green → Refactor.**
The convention "a defined rule without enforcement is meaningless" applies here as well.

#### Mandatory Process

1. **Red**: Write a failing test that defines the expected behavior BEFORE writing any implementation code.
2. **Green**: Write the minimum implementation to make the test pass.
3. **Refactor**: Clean up the implementation while keeping all tests green.

#### What This Means in Practice

| Scenario | Required Action |
|---|---|
| New feature (e.g., centering HTML elements) | Write a test that asserts the expected rendering position. **Verify the test FAILS** before implementing. |
| Bugfix (e.g., elements not centered) | Write a test that reproduces the bug (assert the correct behavior). **Verify the test FAILS** to confirm the bug is captured. Then fix the code. |
| Refactoring | Ensure all existing tests pass before and after the refactor. No test modifications unless the interface changes. |

#### Anti-patterns (Strictly Prohibited)

```text
❌ Implement first, write tests later (or never)
❌ Modify implementation code and ask the user to "verify visually"
❌ Make multiple implementation attempts without automated verification
❌ Remove or weaken a test to make the implementation pass
```

#### UI Testing with egui_kittest (Abolishing Snapshots and Validating Actual Responses)

**Best Practice: UI verification must validate the "actual response (output)".**
Reliance on "Snapshot testing (comparing `.png` files) which can only detect visual differences" is now formally deprecated as technical debt.
Snapshot tests are slow to execute, and when they fail, developers blindly run `UPDATE_SNAPSHOTS=1` assuming "the image just shifted slightly", which renders them completely ineffective as a safeguard against regressions.

**Ironclad Rule for Bugs/Regressions:**
When a bug or regression occurs, **before modifying any code, you have an absolute obligation to first reproduce and bound that failure (the bug's behavior) using TDD (RED)**.
For this verification, you MUST **programmatically and directly assert the "actual response of the UI engine (output such as `Rect` width/height, `Z-index` rendering layer order, etc.)"**, and do not escape to Snapshots.

```rust
// ✅ Good — Assertion-based UI actual response validation (Reproduce/prove the bug with TDD/RED)
#[test]
fn new_horizontal_split_starts_at_half_width() {
    let mut harness = Harness::new_ui(|ui| { /* ... */ });
    harness.run();
    
    // Get the Panel's Rect directly and assert it exactly matches 50% of the screen width in pixels
    let panel_rect = egui::containers::panel::PanelState::load(&ctx, id).unwrap().rect;
    assert!((panel_rect.width() - 600.0).abs() <= 4.0, "must start at 50%");
}

// ❌ Bad — Thoughtless Snapshot image comparison (Technical Debt)
#[test]
fn split_looks_correct() {
    let mut harness = Harness::new_ui(|ui| { /* ... */ });
    harness.step();
    // Meaningless and causes CI delays. Becomes technical debt.
    harness.snapshot_options("split_looks_correct", ...);
}
```

---

## 8. Variable and Type Naming

**Abbreviations are prohibited. Use full names considering future readers.**

```rust
// ✅ Good
let workspace_root = Path::new("/home/user/project");
let active_document = Document::new(path, content);

// ❌ Bad
let ws = Path::new("/home/user/project");
let doc = Document::new(path, content);  // ← Context is lost
```

**Closure parameters**: Emulating Kotlin's `it` idiom, use `it` for single-argument closures.

```rust
// ✅ Good — Single-argument closure
entries.iter().filter(|it| it.is_markdown())

// ✅ Good — for expression with naming focused on readability
for entry in &ws.tree { ... }
for plugin_meta in registry.active_plugins_for(&point) { ... }
```

---

## 9. Linter Setting Summary

Add the following to the top of `lib.rs` / `main.rs` of each crate:

```rust
#![deny(
    clippy::too_many_lines,         // Prohibit functions over 30 lines
    clippy::cognitive_complexity,   // Nesting depth proxy
    clippy::wildcard_imports,       // Prohibit use foo::*
    clippy::unwrap_used,            // Prohibit unwrap()
    clippy::panic,                  // Prohibit panic!
    clippy::todo,                   // Prohibit todo!
    clippy::unimplemented,          // Prohibit unimplemented!
    clippy::exhaustive_structs,     // Non-exhaustive pattern warning on public struct (may be set to warn)
)]
#![warn(
    clippy::expect_used,            // Warn on expect() (permitted with a reason)
    clippy::indexing_slicing,       // Warn on index access
    clippy::missing_errors_doc,     // pub fn's Result requires doc
    missing_docs,                   // pub items require doc comments
)]
```

Set thresholds in `.clippy.toml`:

```toml
too-many-lines-threshold = 30
cognitive-complexity-threshold = 10
```

### 9.2 Quality Gates (Definition of Done)

Prerequisites for allowing a PR to be merged:

1. **Format**: Passes `cargo fmt --all -- --check`
2. **Clippy**: Passes `cargo clippy --workspace -- -D warnings` (Zero warnings)
3. **Tests (Logic)**: Passes all `cargo test --workspace`
4. **Tests (Integration)**: Passes `make test-integration` (No UI Snapshot Regressions)
5. **Test Placement**: New logic has accompanying UT in `src/` inline `#[cfg(test)]`; IT for cross-boundary scenarios in the `tests/` directory
6. **Coverage**: Passes `cargo llvm-cov --workspace --fail-under-lines 100`

Batch check: `make check` (equivalent to the pre-push hook)

---

## 10. Exception Request Process

`#[allow(...)]` is only permitted if any of the following apply:

1. It cannot be split due to framework constraints, such as `update()` in egui
2. Generated code or macro expansion results
3. There is a design reason that obtained agreement during PR review

You must **always state the reason in an English comment** for `#[allow(...)]`:

```rust
// App::update in egui is a single entry point and cannot be split due to framework constraints.
#[allow(clippy::too_many_lines)]
fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) { ... }
```

---

## 11. i18n (Internationalization) Rules [Highest Priority - Maintain Zero Violations]

**All strings displayed in the UI must go through `i18n::t()` or `i18n::tf()`.**
Hardcoding is **strictly prohibited** regardless of the language (everything except English, Japanese, and symbols).

Based on the project's philosophy, "A convention that is defined but has no guarantee of being followed is meaningless," this rule is **mechanically enforced by a Custom AST Linter** defined in Chapter 12.

### 11.1 Target Calls and AST Detection

Passing string literals directly (hardcoded such as `"..."` or `String::from("...")`) in the arguments of the following method calls will be rejected as errors by AST analysis.

| Method Name / Target for Detection | Correct Pattern (Passes Linter) | ❌ Prohibited Pattern (Rejected by Linter) |
|---|---|---|
| `ui.label(...)` | `ui.label(i18n::t("status_ready"))` | `ui.label("Ready")` |
| `ui.heading(...)` | `ui.heading(i18n::t("preview_title"))` | `ui.heading("Preview")` |
| `ui.button(...)` | `ui.button(i18n::t("menu_save"))` | `ui.button("Save")` |
| `RichText::new(...)` | `RichText::new(i18n::t("alert"))` | `RichText::new("Alert")` |
| `.on_hover_text(...)` | `.on_hover_text(i18n::t("expand"))` | `.on_hover_text("Expand all")` |
| String interpolation (e.g., `format!`) | `i18n::tf("saved", &[("key", val)])` | `format!("Saved: {}", val)` |

### 11.2 i18n Exceptions (Automatic Allowlist)

Strings consisting purely of symbols are automatically bypassed as the "AST parser itself evaluates them as an Allowlist".

- **Single Symbols**: `"*"`, `"x"`, `"+"`, `"-"`, `"▼"`, `"▶"`, etc.
- **Icon Emojis**: Standalone symbols for UI controls such as `"🔄"`
- **Path Separators / Layout Whitespace**: `"/"`, `" "`, `"\n"`, etc.
- **Debug Strings**: Inside `tracing::info!`, output independent of egui

### 11.3 Locale File Management

- New keys must be **added simultaneously to en.json and ja.json** (adding to only one is prohibited).
- Missing keys are automatically detected by integration tests (such as `tests/i18n.rs`).

---

## 12. Enforcement via Custom Static Analysis (AST Linter)

To automatically enforce these coding conventions (including i18n rules and prohibited type constraints) in CI rather than manually, we operate a Custom Linter using AST (Abstract Syntax Tree) traversal with Rust's `syn` crate.

> * For specification details, see `docs/ast-linter-plan.md`.

### 12.1 Flow of Enforcement (`pre-commit` / `pre-push`)

When a developer changes code and commits, it goes through the following hard gate. If there is a convention violation, the commit itself will be rejected by `lefthook`.

```
[Code Change] → [lefthook Inspection] → [cargo test (ast_linter.rs)] → [AST Analysis / Pass or Fail]
```
