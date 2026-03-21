---
description: Standard branching strategy for implementing OpenSpec changes
---

# OpenSpec Branch Operations Workflow

This workflow ensures standard operational derivation of base and task-specific branches when starting or continuing an OpenSpec implementation session.

## Workflow Rules

All task operations should adhere to the following branch strategy for a single implementation session:

### Step 1: Base Branch Creation (Initial)
Before starting any tasks, create the Base Feature Branch from `master` named exactly after the change directory (e.g., `desktop-viewer-polish-v0.4.0`).
Command: `git switch -c <Change-Directory-Name> master`

### Step 2: Task Branch Creation (Per Task)
For each individual task mapped in `tasks.md`, derive a new task branch from the Base Feature Branch created in Step 1.
Command: `git switch -c <Change-Directory-Name>-task<N> <Change-Directory-Name>`

### Step 3: Implementation and Integration
Perform the task implementation using designated OpenSpec workflow mechanisms (like `/opsx-apply`). Once the task is completed and verified, the Task Branch MUST be merged back into the **Base Feature Branch** (NOT `master`), leveraging the `openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`).

### Step 4: Synchronization
After successfully merging, switch back to the Base Feature Branch and ensure it is mathematically up-to-date with your remote changes before starting the next Task Branch.
Command: `git switch <Change-Directory-Name> && git pull`

---
**Strict Instruction for AI Agents**: Always execute this branching strategy naturally before checking out code or running `/opsx-apply` on an OpenSpec change. Never commit feature development directly to `master`.
