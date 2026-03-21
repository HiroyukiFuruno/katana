---
name: openspec-delivery
description: Routine workflow to execute OpenSpec task delivery (Verification, Commit, PR creation, and Merge) in a single unified process.
---

# OpenSpec Delivery Workflow

This skill automates the "delivery routine" (from verification to branch compilation) for an individual completed task.

## Execution Requirements

When a task implementation is complete and this skill is invoked, execute all of the following steps **in order**. If any step encounters an error (test failure, conflict, etc.), immediately pause the process and report to the user.

### Step 1: Self-Review & Quality Assurance
Read `.agents/skills/self-review/SKILL.md` and execute self-review for code quality, test coverage (ensure `make check-local` or similar returns exit code 0), and project requirements.

### Step 2: Commit & Push
Read `.agents/skills/commit_and_push/SKILL.md`. Separate current changes logically by concern, create appropriate Japanese commit messages per project rules, and push them to the remote repository.

### Step 3: Create Task Pull Request
Read `.agents/skills/create_pull_request/SKILL.md`. Use the GitHub CLI (`gh` command) to create a Pull Request against the base branch.

### Step 4: Merge PR & Finalize Synchronization
Using the created PR, execute `gh pr merge` to merge it into the base branch. (Use `--admin` option if required, and handle local checkout if git performs a fast-forward).

---
**Strict Instruction for AI Agents**: 
When this routine is specified in `tasks.md`, the AI must NEVER skip or abbreviate the process just because it is condensed into a single checkbox. You must sequentially digest and execute each internal skill phase reliably.
