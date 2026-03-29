---
name: openspec-tasks-template
description: Add Katana-specific "Branch Rule" and "Release Process" boilerplate to OpenSpec `tasks.md` documents. Use this when instructed to create, extend, or organize `tasks.md`.
---
# Katana OpenSpec Tasks Template

When creating, formatting, or updating a `tasks.md` file for an OpenSpec change in the Katana project, ALWAYS use the following structures for the **Branch Rule** and the **Final Checklist**.
Throughout the development lifecycle, proactively utilize OpenSpec skills (`/opsx-propose`, `/opsx-explore`, `/opsx-apply`, `/opsx-archive`) to streamline the workflow.

## 0. Change Directory Naming Rule

When creating a new change directory under `openspec/changes/`, the directory name MUST follow the format `v${x}-${y}-${z}-xxxx` (where `${x}-${y}-${z}` is the target version with hyphens, and `xxxx` is a slug for the feature).

- **Correct**: `v0-5-0-desktop-viewer-polish`
- **Incorrect**: `v0.5.0-desktop-viewer-polish` (No dots allowed in the directory name)

## 1. Branch Rule (Initial Phase)

Insert this at the top of the `tasks.md` file, immediately after the DoR (Definition of Ready):

```markdown
## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.
```

*(Replace `[Change Directory Name]` with the actual change directory name, e.g., `v0-5-0-desktop-viewer-polish`)*

## 2. Final Verification & Release Work (Final Phase)

Insert this as the very last task group at the bottom of the `tasks.md` file.

```markdown
---

## x. Final Verification & Release Work

- [ ] x.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] x.2 Ensure `make check` passes with exit code 0
- [ ] x.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] x.4 Create a PR targeting `master`
- [ ] x.5 Merge into master (※ `--admin` is permitted)
- [ ] x.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `[Target Version]`
- [ ] x.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
```

*(Replace `x` with the next sequential task number. Replace `[Target Version]` with the actual target release version like `0.5.0`)*

## 3. Mandatory Definition of Done (DoD) per Task

To prevent AI workflow skipping and ensure delivery consistency, you MUST INCLUDE the following unified checklist item in the Definition of Done for any individual task. Do NOT split it into separate manual steps.

```markdown
### Definition of Done (DoD)
- [ ] (Other task-specific verifiable conditions...)
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).
```

## 4. Mandatory Definition of Ready (DoR) for Subsequent Tasks

When there are multiple major task groups (e.g., Task 1, Task 2, Task 3...), you MUST INCLUDE the following Definition of Ready (DoR) immediately below the heading of every subsequent task (Task 2 and onwards). This prevents the AI from rushing into the next task without properly delivering the previous one.

```markdown
### Definition of Ready (DoR)
- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.
```

## 5. Mandatory UI Verification & Feedback (When UI is involved)

Because this project does not use rigid design mockups (e.g., Figma), any change involving the creation or modification of User Interfaces (UI) MUST explicitly include the following items as dedicated tasks within the respective UI implementation group:

```markdown
- [ ] x.x ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] x.y ユーザーからのフィードバックに基づくUIの微調整および改善実装
```

*(Ensure these task items are placed *before* the Definition of Done for any UI-related group)*
