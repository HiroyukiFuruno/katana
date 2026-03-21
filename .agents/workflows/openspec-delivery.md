---
description: Routine workflow to execute OpenSpec task delivery (Verification, Commit, PR creation, and Merge) in a single unified process.
---

# OpenSpec Delivery Workflow

This workflow automates the "delivery routine" (from verification to branch compilation) for an individual completed task.

## Execution Requirements

When a task implementation is complete and this workflow is invoked, execute all of the following steps **in order**. If any step encounters an error (test failure, conflict, etc.), immediately pause the process and report to the user.

### Step 1: Self-Review & Quality Assurance

Read and execute `.agents/skills/self-review/SKILL.md`.

### Step 2: Commit & Push

Read and execute `.agents/skills/commit_and_push/SKILL.md`.

### Step 3: Create Task Pull Request

Read and execute `.agents/skills/create_pull_request/SKILL.md`.

The skill determines the correct `--base` branch automatically. Do not override it.

### Step 4: Merge PR & Synchronization

Execute `gh pr merge` (use `--admin` if required). Then switch to the Base Feature Branch and pull.

---

**Strict Instruction for AI Agents**:
- NEVER skip or abbreviate steps just because this workflow is condensed into a single checkbox in `tasks.md`.
- This workflow orchestrates skills. All implementation logic lives in the skills, not here.
