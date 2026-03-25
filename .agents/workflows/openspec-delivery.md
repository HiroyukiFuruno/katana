---
description: Routine workflow to execute OpenSpec task delivery (Verification, Commit, PR creation, and Merge) in a single unified process.
---

# OpenSpec Delivery Workflow

This workflow automates the "delivery routine" (from verification to branch compilation) for an individual completed task.

## Execution Requirements

When a task implementation is complete and this workflow is invoked, execute all of the following steps **in order**. If any step encounters an error (test failure, conflict, etc.), immediately pause the process and report to the user.

### Step 1: Self-Review & Quality Assurance

> **🚨 CRITICAL AI INSTRUCTION: DO NOT SKIP THIS STEP 🚨**
> AI Agents frequently skip self-review and immediately attempt to run `cargo fmt && make check && git commit ...`. This is a strict violation of the delivery process.
> **BEFORE** you type any bash commands to compile, format, or commit:
> 1. You MUST explicitly plan a task and use `view_file` to read `.agents/skills/self-review/SKILL.md`.
> 2. You MUST perform the cognitive self-review described in the skill against the project's coding rules (`docs/coding-rules.ja.md`).
> 3. Only after correcting any logical or formatting violations (like Magic Numbers or `#[allow]` violations) are you permitted to proceed to bash execution.

Read and execute `.agents/skills/self-review/SKILL.md`.

### Step 2: Commit & Push

Read and execute `.agents/skills/commit_and_push/SKILL.md`.

### Step 3: Create Task Pull Request

Read and execute `.agents/skills/create_pull_request/SKILL.md`.

The skill determines the correct `--base` branch automatically. Do not override it.

### Step 4: Merge PR & Synchronization

1. Execute `gh pr merge --merge --delete-branch` (or `--admin` if required) to merge the PR and clean up the remote branch.
2. Switch back to the Base Feature Branch (`git checkout <base-branch>`).
3. Pull the latest changes from remote (`git pull`).
4. Delete the successfully merged local task branch (`git branch -D <task-branch>`).

---

**Strict Instruction for AI Agents**:
- NEVER skip or abbreviate steps just because this workflow is condensed into a single checkbox in `tasks.md`.
- This workflow orchestrates skills. All implementation logic lives in the skills, not here.
