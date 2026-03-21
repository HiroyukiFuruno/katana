---
description: Routine workflow to execute OpenSpec task delivery (Verification, Commit, PR creation, and Merge) in a single unified process.
---

# OpenSpec Delivery Workflow

This workflow automates the "delivery routine" (from verification to branch compilation) for an individual completed task.

## Execution Requirements

When a task implementation is complete and this workflow is invoked, execute all of the following steps **in order**. If any step encounters an error (test failure, conflict, etc.), immediately pause the process and report to the user.

### Step 0: Pre-Flight — Determine Base Branch

> [!CAUTION]
> **Task PRs MUST NEVER target `master` directly.** Per `/openspec-branching`, task branches merge into the **Base Feature Branch**, not `master`. Only the final release merge (tasks.md Step 5) targets `master`.

Determine the correct base branch by inspecting the current branch name:
- Current branch pattern: `<change-dir>-task<N>` (e.g., `desktop-viewer-polish-v0.4.0-task2`)
- Base Feature Branch: strip the `-task<N>` suffix (e.g., `desktop-viewer-polish-v0.4.0`)
- **Verify** the Base Feature Branch exists: `git branch -a | grep <base-branch>`
- If the Base Feature Branch does not exist, **stop and ask the user**.

### Step 1: Self-Review & Quality Assurance

Read `.agents/skills/self-review/SKILL.md` and execute self-review for code quality, test coverage (ensure `make check-local` or similar returns exit code 0), and project requirements.

### Step 2: Commit & Push

Read `.agents/skills/commit_and_push/SKILL.md`. Separate current changes logically by concern, create appropriate Japanese commit messages per project rules, and push them to the remote repository.

### Step 3: Create Task Pull Request

Read `.agents/skills/create_pull_request/SKILL.md`. Use the GitHub CLI (`gh` command) to create a Pull Request.

> [!IMPORTANT]
> **Always specify `--base <Base-Feature-Branch>`** (determined in Step 0) when running `gh pr create`. Example:
> ```
> gh pr create --base desktop-viewer-polish-v0.4.0 --title "..." --body "..."
> ```
> Never omit `--base`; omitting it defaults to `master`, which violates the branching strategy.

### Step 4: Merge PR & Finalize Synchronization

Using the created PR, execute `gh pr merge` to merge it into the Base Feature Branch. (Use `--admin` option if required, and handle local checkout if git performs a fast-forward).

---

**Strict Instruction for AI Agents**:
- When this routine is specified in `tasks.md`, the AI must NEVER skip or abbreviate the process just because it is condensed into a single checkbox. You must sequentially digest and execute each internal skill phase reliably.
- **NEVER merge a task PR into `master`.** The only exception is the explicit final release merge step defined in `tasks.md`.
