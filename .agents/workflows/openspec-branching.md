---
description: Standard branching strategy for implementing OpenSpec changes
---

# OpenSpec Branch Operations Workflow

This workflow defines the branching **strategy** (what branches to create and where to merge). The mechanics of PR creation, base branch determination, and merging are handled by the skills this workflow depends on.

## Dependencies

- `.agents/skills/create_pull_request/SKILL.md` — owns base branch determination and PR creation
- `.agents/skills/commit_and_push/SKILL.md` — owns commit and push mechanics

## Workflow Steps

### Step 1: Base Branch Creation (Initial)

Before starting any tasks, create the Base Feature Branch from `master` named exactly after the change directory.

```bash
git switch -c <Change-Directory-Name> master
```

### Step 2: Task Branch Creation (Per MAJOR Task Group)

For each **Major Task Group** (e.g., `## 1. Core Logic`, `## 2. UI implementation`) in `tasks.md`, derive a *single* task branch from the Base Feature Branch.

```bash
git switch -c <Change-Directory-Name>-task<N> <Change-Directory-Name>
```

**⚠️ MANDATORY RULE: NO SUBTASK BRANCHING**
- DO NOT branch for subtasks (`1.1`, `1.2`, `1.4`, etc.).
- A branch named `-task4` for subtask `1.4` is logically incorrect and strictly prohibited. Subtasks `1.1` to `1.5` MUST all be implemented and committed into the same `...-task1` branch.
- You should proceed sequentially on the `-task<N>` branch through all its subtasks, committing regularly, and DO NOT open a Pull Request until ALL subtasks for that Major Task Group are completed.

### Step 3: Implementation and Delivery

Implement the task, then execute the `/openspec-delivery` workflow to deliver.

The delivery workflow calls `create_pull_request` skill, which automatically determines that the `--base` is the Base Feature Branch (by stripping `-task<N>` from the current branch name). This ensures task PRs never target `master`.

### Step 4: Synchronization

After merge, return to the Base Feature Branch and sync before starting the next task.

```bash
git switch <Change-Directory-Name> && git pull
```

---

**Strict Instruction for AI Agents**: Always execute this branching strategy before starting task implementation. Never commit feature work directly to `master`.
