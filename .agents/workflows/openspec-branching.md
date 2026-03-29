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

Before creating a new task branch (for Task 2, Task 3, etc.), you MUST explicitly verify its **Definition of Ready (DoR)**:

- **pr**: The PR for the *previous* task branch was created.
- **review**: The *previous* task branch was self-reviewed.
- **recovery**: Any violations found during review were fixed and committed.
- **merge**: The PR for the *previous* task branch was successfully merged.
- **branch delete**: The local and remote branches for the *previous* task were deleted.

If the DoR is satisfied, base branch is synced (`git switch <base> && git pull`), then for each **Major Task Group** (e.g., `## 1. Core Logic`) in `tasks.md`, derive a *single* task branch from the Base Feature Branch.

```bash
git switch -c <Change-Directory-Name>-task<N> <Change-Directory-Name>
```

#### ⚠️ MANDATORY RULE: NO SUBTASK BRANCHING

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

### Step 5: Merge Base Feature Branch into `master`

After **all** Major Task Groups are complete and merged into the Base Feature Branch, create a PR from the Base Feature Branch into `master`.

```bash
gh pr create --base master --head <Change-Directory-Name> \
  --title "<PR title summarizing the change>" \
  --body "<summary of all completed tasks>"
```

Merge the PR once approved (or self-merge if the project policy allows):

```bash
gh pr merge --squash  # or --merge, per project convention
```

### Step 6: Branch Cleanup (Local and Remote)

After the Base Feature Branch is merged into `master`, delete both the **local** and **remote** branches to keep the repository clean.

```bash
# Delete local branch
git branch -d <Change-Directory-Name>

# Delete remote branch
git push origin --delete <Change-Directory-Name>
```

**⚠️ MANDATORY**: Both local AND remote cleanup MUST be performed. Leaving either as an orphan branch is a workflow violation. Verify with:

```bash
git branch -a | grep <Change-Directory-Name>
# Expected: no output
```

---

**Strict Instruction for AI Agents**: Always execute this branching strategy before starting task implementation. Never commit feature work directly to `master`. Never delete a branch (local or remote) without first ensuring it is fully merged into its target.
