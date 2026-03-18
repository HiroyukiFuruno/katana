---
name: self-review
description: Pre-commit self-review skill. Autonomously checks coding standards, verification integrity, breaking change detection, and language rules.
---

# Self-Review Skill

A mandatory self-review before every commit. Check **all** of the following
criteria. If any violation is found, fix it before committing.

## Review Scope

The review targets **the current diff only**, with one exception:

| Criteria | Scope | Rationale |
|---|---|---|
| Coding standards | **Diff only** | Existing violations are separate tasks. Goal: don't introduce new ones. |
| Verification integrity | **Diff only** | Only added/modified tests are in scope. |
| Breaking changes & bugs | **Diff + call sites** | Type/API changes require tracing consumers to verify correctness. |
| Language rules | **Diff only** | Fixing all existing Japanese comments is a separate task. |

> **Important:** If large-scale refactoring is needed for existing code,
> file it as a separate task. Do not mix unrelated fixes into the current
> commit — it makes review harder and blocks progress.

### Recording Out-of-Scope Issues

When the review discovers issues **outside the current diff scope**
(existing violations, design improvements, etc.), they must NOT be silently
ignored. Per the `technical_debt_and_separation` skill:

> **Fix it now, or record it now. Never leave it untracked.**

Record out-of-scope issues in `openspec/changes/technical-debt/proposal.md`
with:
- A description of the issue
- The discovery context (what triggered you to notice it)

---

## 1. Coding Standards Check

Verify compliance with the rules defined in `docs/coding-rules.ja.md`.

### Automated (enforced by `make check`)
- `cargo fmt --check` — formatting
- `cargo clippy` — lint (`#![deny(warnings)]` enforced)
- `cargo test -p katana-linter` — AST Linter (magic numbers, function size, etc.)
- `cargo-llvm-cov` — coverage gate

### Manual (AI visual inspection)
- [ ] Function size ≤ 30 lines
- [ ] Nesting depth ≤ 3 levels
- [ ] Error-first pattern (`?` / `let...else`)
- [ ] `struct + impl` based design (no pub free functions)
- [ ] No magic numbers (extracted to `const`)
- [ ] No banned types (`Box<dyn Any>`, `HashMap<String, Value>`, etc.)
- [ ] Appropriate derives on new structs (`Debug`, `Clone`, `PartialEq`, `Serialize`, etc.)
- [ ] Unit tests follow Rust conventions: inline `#[cfg(test)]` in `src/`; integration tests live in `tests/`
- [ ] No private function testing hacks (redesign instead)

### AI Quality is Speed Check (No Sloppy Shortcuts)
- [ ] Are there ANY lazy shortcuts? (e.g., `todo!()`, `unimplemented!()`, `dbg!()`, `unwrap()` used as a temporary bypass). **Fix them now.**
- [ ] Does this design gracefully support future extension, avoiding hardcoded boolean flags or spaghetti state?
- [ ] Is it built with the absolute principle of "丁寧＝早い (Careful = Fast)" preventing bugs later?

---

## 2. OpenSpec State Check

If working under an OpenSpec change, verify the task state is updated.

### Checklist
- [ ] Completed tasks in `tasks.md` are marked as done (`[x]`)
- [ ] In-progress tasks are clearly indicated
- [ ] Task completion aligns with actual code changes

---

## 2. Verification Integrity Check (Cheating Detection)

Ensure tests are not distorted just to make them pass.

### Checklist
- [ ] Tests verify real logic (no empty assertions like `assert!(true)`)
- [ ] Assertions compare against concrete expected values
- [ ] No unjustified relaxation of linter rules or allowed-list expansion. If done, is the reason legitimate?
- [ ] No intentional test disabling via `#[ignore]` or `#[cfg(not(test))]`
- [ ] Snapshot updates are expected diffs caused by the change (not silent regressions)
- [ ] No unnatural code contortions to bypass the coverage gate

---

## 3. Breaking Change & Bug Detection Check

Confirm the change does not break existing functionality.

### Checklist
- [ ] `make check` passes all gates (fmt / lint / test / coverage)
- [ ] All call sites are updated for changed public APIs (`pub fn`, `pub struct` fields)
- [ ] Serde compatibility: existing JSON settings files can deserialize into the new structs (proper use of `#[serde(default)]`)
- [ ] Type signature changes are propagated across all dependent crates

### Handling User-Reported Bugs
When a user reports a bug that was not caught:
- [ ] A unit test or integration test reproducing the reported issue was added **first**
- [ ] The test FAILs before the fix and PASSes after
- [ ] Similar edge-case test scenarios were also added

---

## 4. Language Rule Check

Verify the correct language is used in Git-managed files.

### Rules
| File type | Language |
|---|---|
| Source code (`.rs`, `.toml`, `.yml`, etc.) | **English** |
| Comments and doc comments in source code | **English** |
| Test function names (`fn test_*`) | **English** |
| Files with `_ja` / `.ja` suffix (e.g. `*.ja.md`) | Japanese OK |
| Other `.md`, `.txt` documents without `_ja` suffix | **English** |
| Git commit messages | **Japanese** (project convention) |

### Checklist
- [ ] Comments in new/modified files are written in English
- [ ] Any remaining Japanese comments are translated to English
- [ ] Test function `///` doc comments and `assert!` messages are in English
- [ ] Exception: files with `_ja` suffix may use Japanese

---

## Execution Steps

```bash
# 1. Automated checks
make check

# 2. Manual review (walk through the checklists above)

# 3. Final diff inspection
git diff --stat HEAD
git diff HEAD              # review all diffs

# 4. Fix issues → re-run make check → re-review

# 5. Once all checks pass, proceed to commit
```

---

## Output Format

Output review results as an artifact in the following format:

```markdown
# Self-Review: [Change Title]

## ✅ No Issues
- [Checked item and result]

## ⚠️ Findings
- [Description and remediation plan]

## Conclusion
[PASS / FAIL — reason]
```
